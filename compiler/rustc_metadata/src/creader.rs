//! Validates all used crates and extern libraries and loads their metadata

#[cfg(any(unix, windows))]
use std::error::Error;

use std::ops::Fn;
use std::path::Path;
use std::str::FromStr;
use std::{cmp, env, iter};

use proc_macro::bridge::client::ProcMacro;
use rustc_ast::expand::allocator::{AllocatorKind, alloc_error_handler_name, global_fn_name};
use rustc_ast::{self as ast, *};
use rustc_data_structures::fx::FxHashSet;
use rustc_data_structures::owned_slice::OwnedSlice;
use rustc_data_structures::svh::Svh;
use rustc_data_structures::sync::{self, FreezeReadGuard, FreezeWriteGuard, Lrc};
use rustc_errors::DiagCtxtHandle;
use rustc_expand::base::{SyntaxExtension, SyntaxExtensionKind};

#[cfg(any(unix, windows))]
use rustc_fs_util::try_canonicalize;

use rustc_hir::def_id::{CrateNum, LOCAL_CRATE, LocalDefId, StableCrateId};
use rustc_hir::definitions::Definitions;
use rustc_index::IndexVec;
use rustc_middle::bug;
use rustc_middle::ty::{TyCtxt, TyCtxtFeed};
use rustc_session::config::{self, CrateType, ExternLocation};
use rustc_session::cstore::{CrateDepKind, CrateSource, ExternCrate, ExternCrateSource};
use rustc_session::lint::{self, BuiltinLintDiag};
use rustc_session::output::validate_crate_name;
use rustc_session::search_paths::PathKind;
use rustc_span::edition::Edition;
use rustc_span::symbol::{Ident, Symbol, sym};
use rustc_span::{DUMMY_SP, Span};
use rustc_target::spec::{PanicStrategy, Target, TargetTuple};
use tracing::{debug, info, trace};

use crate::errors;
use crate::locator::{CrateError, CrateLocator, CratePaths};
use crate::rmeta::{CrateDep, CrateMetadata, CrateNumMap, CrateRoot, MetadataBlob};

/// The backend's way to give the crate store access to the metadata in a library.
/// Note that it returns the raw metadata bytes stored in the library file, whether
/// it is compressed, uncompressed, some weird mix, etc.
/// rmeta files are backend independent and not handled here.
pub trait MetadataLoader {
    fn get_rlib_metadata(&self, target: &Target, filename: &Path) -> Result<OwnedSlice, String>;
    fn get_dylib_metadata(&self, target: &Target, filename: &Path) -> Result<OwnedSlice, String>;
}

pub type MetadataLoaderDyn = dyn MetadataLoader + Send + Sync + sync::DynSend + sync::DynSync;

pub struct CStore {
    metadata_loader: Box<MetadataLoaderDyn>,

    metas: IndexVec<CrateNum, Option<Box<CrateMetadata>>>,
    injected_panic_runtime: Option<CrateNum>,
    /// This crate needs an allocator and either provides it itself, or finds it in a dependency.
    /// If the above is true, then this field denotes the kind of the found allocator.
    allocator_kind: Option<AllocatorKind>,
    /// This crate needs an allocation error handler and either provides it itself, or finds it in a dependency.
    /// If the above is true, then this field denotes the kind of the found allocator.
    alloc_error_handler_kind: Option<AllocatorKind>,
    /// This crate has a `#[global_allocator]` item.
    has_global_allocator: bool,
    /// This crate has a `#[alloc_error_handler]` item.
    has_alloc_error_handler: bool,

    /// Unused externs of the crate
    unused_externs: Vec<Symbol>,
}

impl std::fmt::Debug for CStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CStore").finish_non_exhaustive()
    }
}

pub struct CrateLoader<'a, 'tcx: 'a> {
    // Immutable configuration.
    tcx: TyCtxt<'tcx>,
    // Mutable output.
    cstore: &'a mut CStore,
    used_extern_options: &'a mut FxHashSet<Symbol>,
}

impl<'a, 'tcx> std::ops::Deref for CrateLoader<'a, 'tcx> {
    type Target = TyCtxt<'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.tcx
    }
}

impl<'a, 'tcx> CrateLoader<'a, 'tcx> {
    fn dcx(&self) -> DiagCtxtHandle<'tcx> {
        self.tcx.dcx()
    }
}

pub enum LoadedMacro {
    MacroDef { def: MacroDef, ident: Ident, attrs: AttrVec, span: Span, edition: Edition },
    ProcMacro(SyntaxExtension),
}

pub(crate) struct Library {
    pub source: CrateSource,
    pub metadata: MetadataBlob,
}

enum LoadResult {
    Previous(CrateNum),
    Loaded(Library),
}

/// A reference to `CrateMetadata` that can also give access to whole crate store when necessary.
#[derive(Clone, Copy)]
pub(crate) struct CrateMetadataRef<'a> {
    pub cdata: &'a CrateMetadata,
    pub cstore: &'a CStore,
}

impl std::ops::Deref for CrateMetadataRef<'_> {
    type Target = CrateMetadata;

    fn deref(&self) -> &Self::Target {
        self.cdata
    }
}

struct CrateDump<'a>(&'a CStore);

impl<'a> std::fmt::Debug for CrateDump<'a> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(fmt, "resolved crates:")?;
        for (cnum, data) in self.0.iter_crate_data() {
            writeln!(fmt, "  name: {}", data.name())?;
            writeln!(fmt, "  cnum: {cnum}")?;
            writeln!(fmt, "  hash: {}", data.hash())?;
            writeln!(fmt, "  reqd: {:?}", data.dep_kind())?;
            let CrateSource { dylib, rlib, rmeta } = data.source();
            if let Some(dylib) = dylib {
                writeln!(fmt, "  dylib: {}", dylib.0.display())?;
            }
            if let Some(rlib) = rlib {
                writeln!(fmt, "   rlib: {}", rlib.0.display())?;
            }
            if let Some(rmeta) = rmeta {
                writeln!(fmt, "   rmeta: {}", rmeta.0.display())?;
            }
        }
        Ok(())
    }
}

impl CStore {
    pub fn from_tcx(tcx: TyCtxt<'_>) -> FreezeReadGuard<'_, CStore> {
        FreezeReadGuard::map(tcx.untracked().cstore.read(), |cstore| {
            cstore.as_any().downcast_ref::<CStore>().expect("`tcx.cstore` is not a `CStore`")
        })
    }

    pub fn from_tcx_mut(tcx: TyCtxt<'_>) -> FreezeWriteGuard<'_, CStore> {
        FreezeWriteGuard::map(tcx.untracked().cstore.write(), |cstore| {
            cstore.untracked_as_any().downcast_mut().expect("`tcx.cstore` is not a `CStore`")
        })
    }

    fn intern_stable_crate_id<'tcx>(
        &mut self,
        root: &CrateRoot,
        tcx: TyCtxt<'tcx>,
    ) -> Result<TyCtxtFeed<'tcx, CrateNum>, CrateError> {
        assert_eq!(self.metas.len(), tcx.untracked().stable_crate_ids.read().len());
        let num = tcx.create_crate_num(root.stable_crate_id()).map_err(|existing| {
            // Check for (potential) conflicts with the local crate
            if existing == LOCAL_CRATE {
                CrateError::SymbolConflictsCurrent(root.name())
            } else if let Some(crate_name1) = self.metas[existing].as_ref().map(|data| data.name())
            {
                let crate_name0 = root.name();
                CrateError::StableCrateIdCollision(crate_name0, crate_name1)
            } else {
                CrateError::NotFound(root.name())
            }
        })?;

        self.metas.push(None);
        Ok(num)
    }

    pub fn has_crate_data(&self, cnum: CrateNum) -> bool {
        self.metas[cnum].is_some()
    }

    pub(crate) fn get_crate_data(&self, cnum: CrateNum) -> CrateMetadataRef<'_> {
        let cdata = self.metas[cnum]
            .as_ref()
            .unwrap_or_else(|| panic!("Failed to get crate data for {cnum:?}"));
        CrateMetadataRef { cdata, cstore: self }
    }

    pub(crate) fn get_crate_data_mut(&mut self, cnum: CrateNum) -> &mut CrateMetadata {
        self.metas[cnum].as_mut().unwrap_or_else(|| panic!("Failed to get crate data for {cnum:?}"))
    }

    fn set_crate_data(&mut self, cnum: CrateNum, data: CrateMetadata) {
        assert!(self.metas[cnum].is_none(), "Overwriting crate metadata entry");
        self.metas[cnum] = Some(Box::new(data));
    }

    pub(crate) fn iter_crate_data(&self) -> impl Iterator<Item = (CrateNum, &CrateMetadata)> {
        self.metas
            .iter_enumerated()
            .filter_map(|(cnum, data)| data.as_deref().map(|data| (cnum, data)))
    }

    fn iter_crate_data_mut(&mut self) -> impl Iterator<Item = (CrateNum, &mut CrateMetadata)> {
        self.metas
            .iter_enumerated_mut()
            .filter_map(|(cnum, data)| data.as_deref_mut().map(|data| (cnum, data)))
    }

    fn push_dependencies_in_postorder(&self, deps: &mut Vec<CrateNum>, cnum: CrateNum) {
        if !deps.contains(&cnum) {
            let data = self.get_crate_data(cnum);
            for dep in data.dependencies() {
                if dep != cnum {
                    self.push_dependencies_in_postorder(deps, dep);
                }
            }

            deps.push(cnum);
        }
    }

    pub(crate) fn crate_dependencies_in_postorder(&self, cnum: CrateNum) -> Vec<CrateNum> {
        let mut deps = Vec::new();
        if cnum == LOCAL_CRATE {
            for (cnum, _) in self.iter_crate_data() {
                self.push_dependencies_in_postorder(&mut deps, cnum);
            }
        } else {
            self.push_dependencies_in_postorder(&mut deps, cnum);
        }
        deps
    }

    fn crate_dependencies_in_reverse_postorder(&self, cnum: CrateNum) -> Vec<CrateNum> {
        let mut deps = self.crate_dependencies_in_postorder(cnum);
        deps.reverse();
        deps
    }

    pub(crate) fn injected_panic_runtime(&self) -> Option<CrateNum> {
        self.injected_panic_runtime
    }

    pub(crate) fn allocator_kind(&self) -> Option<AllocatorKind> {
        self.allocator_kind
    }

    pub(crate) fn alloc_error_handler_kind(&self) -> Option<AllocatorKind> {
        self.alloc_error_handler_kind
    }

    pub(crate) fn has_global_allocator(&self) -> bool {
        self.has_global_allocator
    }

    pub(crate) fn has_alloc_error_handler(&self) -> bool {
        self.has_alloc_error_handler
    }

    pub fn report_unused_deps(&self, tcx: TyCtxt<'_>) {
        let json_unused_externs = tcx.sess.opts.json_unused_externs;

        // We put the check for the option before the lint_level_at_node call
        // because the call mutates internal state and introducing it
        // leads to some ui tests failing.
        if !json_unused_externs.is_enabled() {
            return;
        }
        let level = tcx
            .lint_level_at_node(lint::builtin::UNUSED_CRATE_DEPENDENCIES, rustc_hir::CRATE_HIR_ID)
            .0;
        if level != lint::Level::Allow {
            let unused_externs =
                self.unused_externs.iter().map(|ident| ident.to_ident_string()).collect::<Vec<_>>();
            let unused_externs = unused_externs.iter().map(String::as_str).collect::<Vec<&str>>();
            tcx.dcx().emit_unused_externs(level, json_unused_externs.is_loud(), &unused_externs);
        }
    }

    pub fn new(metadata_loader: Box<MetadataLoaderDyn>) -> CStore {
        CStore {
            metadata_loader,
            // We add an empty entry for LOCAL_CRATE (which maps to zero) in
            // order to make array indices in `metas` match with the
            // corresponding `CrateNum`. This first entry will always remain
            // `None`.
            metas: IndexVec::from_iter(iter::once(None)),
            injected_panic_runtime: None,
            allocator_kind: None,
            alloc_error_handler_kind: None,
            has_global_allocator: false,
            has_alloc_error_handler: false,
            unused_externs: Vec::new(),
        }
    }
}

impl<'a, 'tcx> CrateLoader<'a, 'tcx> {
    pub fn new(
        tcx: TyCtxt<'tcx>,
        cstore: &'a mut CStore,
        used_extern_options: &'a mut FxHashSet<Symbol>,
    ) -> Self {
        CrateLoader { tcx, cstore, used_extern_options }
    }

    /// Load WASM proc macros specified via `--wasm-proc-macro` flags
    /// Returns a vector of (macro_name, SyntaxExtension) tuples for the resolver to register
    /// This bypasses the normal metadata/CStore system entirely
    pub fn load_wasm_proc_macros(&mut self) -> Vec<(Symbol, Lrc<SyntaxExtension>)> {
        // Only compile this code when building rustc for WASM
        #[cfg(target_family = "wasm")]
        {
            use std::fs;
            use rustc_watt_runtime::WasmMacro;

            let mut result = Vec::new();

            eprintln!("[CREADER] load_wasm_proc_macros called with {} entries",
                      self.sess.opts.wasm_proc_macros.len());

            for (name, path) in &self.sess.opts.wasm_proc_macros {
                eprintln!("[CREADER] Loading WASM proc macro: {} from {:?}", name, path);

                // Read the WASM file
                let wasm_bytes = match fs::read(path) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        self.dcx().fatal(format!(
                            "Failed to read WASM proc macro file {}: {}",
                            path.display(),
                            e
                        ));
                    }
                };

                eprintln!("[CREADER] Read {} bytes from {}", wasm_bytes.len(), path.display());

                // Create WasmMacro instance
                let wasm_macro = WasmMacro::new_owned(wasm_bytes);

                // Extract proc macros from WASM
                let proc_macros = create_wasm_proc_macros(wasm_macro);

                eprintln!("[CREADER] Extracted {} proc macros from WASM file", proc_macros.len());

                // Convert ProcMacro to SyntaxExtension before passing to resolver
                // This avoids needing proc_macro crate dependency in rustc_resolve
                for pm in proc_macros {
                    let (name, kind, helper_attrs) = match pm {
                        ProcMacro::CustomDerive { trait_name, attributes, client } => {
                            let helper_attrs = attributes.iter()
                                .map(|attr| Symbol::intern(attr))
                                .collect::<Vec<_>>();
                            (
                                trait_name,
                                SyntaxExtensionKind::Derive(Box::new(rustc_expand::proc_macro::DeriveProcMacro { client })),
                                helper_attrs,
                            )
                        }
                        ProcMacro::Attr { name, client } => {
                            (
                                name,
                                SyntaxExtensionKind::Attr(Box::new(rustc_expand::proc_macro::AttrProcMacro { client })),
                                Vec::new(),
                            )
                        }
                        ProcMacro::Bang { name, client } => {
                            (
                                name,
                                SyntaxExtensionKind::Bang(Box::new(rustc_expand::proc_macro::BangProcMacro { client })),
                                Vec::new(),
                            )
                        }
                    };

                    // Create a minimal SyntaxExtension for WASM proc macros
                    // We use dummy/minimal values since we don't have full metadata
                    let ext = SyntaxExtension {
                        kind,
                        span: DUMMY_SP,
                        allow_internal_unstable: None,
                        stability: None,
                        deprecation: None,
                        helper_attrs,
                        edition: Edition::Edition2015,
                        builtin_name: None,
                        allow_internal_unsafe: false,
                        local_inner_macros: false,
                        collapse_debuginfo: false,
                    };

                    result.push((Symbol::intern(name), Lrc::new(ext)));
                }
            }

            result
        }

        #[cfg(not(target_family = "wasm"))]
        {
            // When building rustc for non-WASM platforms, return empty vector
            // The flag will just be ignored
            let _ = &self.sess.opts.wasm_proc_macros;
            Vec::new()
        }
    }

    fn existing_match(&self, name: Symbol, hash: Option<Svh>, kind: PathKind) -> Option<CrateNum> {
        for (cnum, data) in self.cstore.iter_crate_data() {
            if data.name() != name {
                trace!("{} did not match {}", data.name(), name);
                continue;
            }

            match hash {
                Some(hash) if hash == data.hash() => return Some(cnum),
                Some(hash) => {
                    debug!("actual hash {} did not match expected {}", hash, data.hash());
                    continue;
                }
                None => {}
            }

            // When the hash is None we're dealing with a top-level dependency
            // in which case we may have a specification on the command line for
            // this library. Even though an upstream library may have loaded
            // something of the same name, we have to make sure it was loaded
            // from the exact same location as well.
            //
            // We're also sure to compare *paths*, not actual byte slices. The
            // `source` stores paths which are normalized which may be different
            // from the strings on the command line.
            let source = self.cstore.get_crate_data(cnum).cdata.source();
            if let Some(entry) = self.sess.opts.externs.get(name.as_str()) {
                // Only use `--extern crate_name=path` here, not `--extern crate_name`.
                if let Some(mut files) = entry.files() {
                    if files.any(|l| {
                        let l = l.canonicalized();
                        source.dylib.as_ref().map(|(p, _)| p) == Some(l)
                            || source.rlib.as_ref().map(|(p, _)| p) == Some(l)
                            || source.rmeta.as_ref().map(|(p, _)| p) == Some(l)
                    }) {
                        return Some(cnum);
                    }
                }
                continue;
            }

            // Alright, so we've gotten this far which means that `data` has the
            // right name, we don't have a hash, and we don't have a --extern
            // pointing for ourselves. We're still not quite yet done because we
            // have to make sure that this crate was found in the crate lookup
            // path (this is a top-level dependency) as we don't want to
            // implicitly load anything inside the dependency lookup path.
            let prev_kind = source
                .dylib
                .as_ref()
                .or(source.rlib.as_ref())
                .or(source.rmeta.as_ref())
                .expect("No sources for crate")
                .1;
            if kind.matches(prev_kind) {
                return Some(cnum);
            } else {
                debug!(
                    "failed to load existing crate {}; kind {:?} did not match prev_kind {:?}",
                    name, kind, prev_kind
                );
            }
        }

        None
    }

    // The `dependency` type is determined by the command line arguments(`--extern`) and
    // `private_dep`. However, sometimes the directly dependent crate is not specified by
    // `--extern`, in this case, `private-dep` is none during loading. This is equivalent to the
    // scenario where the command parameter is set to `public-dependency`
    fn is_private_dep(&self, name: &str, private_dep: Option<bool>) -> bool {
        self.sess.opts.externs.get(name).map_or(private_dep.unwrap_or(false), |e| e.is_private_dep)
            && private_dep.unwrap_or(true)
    }

    fn register_crate(
        &mut self,
        host_lib: Option<Library>,
        root: Option<&CratePaths>,
        lib: Library,
        dep_kind: CrateDepKind,
        name: Symbol,
        private_dep: Option<bool>,
        pre_loaded_proc_macros: Option<&'static [ProcMacro]>,
    ) -> Result<CrateNum, CrateError> {
        let _prof_timer =
            self.sess.prof.generic_activity_with_arg("metadata_register_crate", name.as_str());

        let Library { source, metadata } = lib;
        let crate_root = metadata.get_root();
        let host_hash = host_lib.as_ref().map(|lib| lib.metadata.get_root().hash());
        let private_dep = self.is_private_dep(name.as_str(), private_dep);

        // Claim this crate number and cache it
        let feed = self.cstore.intern_stable_crate_id(&crate_root, self.tcx)?;
        let cnum = feed.key();

        info!(
            "register crate `{}` (cnum = {}. private_dep = {})",
            crate_root.name(),
            cnum,
            private_dep
        );

        // Maintain a reference to the top most crate.
        // Stash paths for top-most crate locally if necessary.
        let crate_paths;
        let root = if let Some(root) = root {
            root
        } else {
            crate_paths = CratePaths::new(crate_root.name(), source.clone());
            &crate_paths
        };

        let cnum_map = self.resolve_crate_deps(root, &crate_root, &metadata, cnum, dep_kind)?;

        let raw_proc_macros = if let Some(pre_loaded) = pre_loaded_proc_macros {
            // Use pre-loaded proc macros (e.g., from WASM)
            eprintln!("[CREADER] Using {} pre-loaded proc macros", pre_loaded.len());
            Some(pre_loaded)
        } else if crate_root.is_proc_macro_crate() {
            // Load proc macros from dylib using dlsym
            let temp_root;
            let (dlsym_source, dlsym_root) = match &host_lib {
                Some(host_lib) => (&host_lib.source, {
                    temp_root = host_lib.metadata.get_root();
                    &temp_root
                }),
                None => (&source, &crate_root),
            };
            let dlsym_dylib = dlsym_source.dylib.as_ref().expect("no dylib for a proc-macro crate");
            Some(self.dlsym_proc_macros(&dlsym_dylib.0, dlsym_root.stable_crate_id())?)
        } else {
            None
        };

        let crate_metadata = CrateMetadata::new(
            self.sess,
            self.cstore,
            metadata,
            crate_root,
            raw_proc_macros,
            cnum,
            cnum_map,
            dep_kind,
            source,
            private_dep,
            host_hash,
        );

        self.cstore.set_crate_data(cnum, crate_metadata);

        Ok(cnum)
    }

    fn load_proc_macro<'b>(
        &self,
        locator: &mut CrateLocator<'b>,
        path_kind: PathKind,
        host_hash: Option<Svh>,
    ) -> Result<Option<(LoadResult, Option<Library>)>, CrateError>
    where
        'a: 'b,
    {
        // Use a new crate locator so trying to load a proc macro doesn't affect the error
        // message we emit
        let mut proc_macro_locator = locator.clone();

        // Try to load a proc macro
        proc_macro_locator.is_proc_macro = true;

        // Load the proc macro crate for the target
        let (locator, target_result) = if self.sess.opts.unstable_opts.dual_proc_macros {
            proc_macro_locator.reset();
            let result = match self.load(&mut proc_macro_locator)? {
                Some(LoadResult::Previous(cnum)) => {
                    return Ok(Some((LoadResult::Previous(cnum), None)));
                }
                Some(LoadResult::Loaded(library)) => Some(LoadResult::Loaded(library)),
                None => return Ok(None),
            };
            locator.hash = host_hash;
            // Use the locator when looking for the host proc macro crate, as that is required
            // so we want it to affect the error message
            (locator, result)
        } else {
            (&mut proc_macro_locator, None)
        };

        // Load the proc macro crate for the host

        locator.reset();
        locator.is_proc_macro = true;
        locator.target = &self.sess.host;
        locator.tuple = TargetTuple::from_tuple(config::host_tuple());
        locator.filesearch = self.sess.host_filesearch();
        locator.path_kind = path_kind;

        let Some(host_result) = self.load(locator)? else {
            return Ok(None);
        };

        Ok(Some(if self.sess.opts.unstable_opts.dual_proc_macros {
            let host_result = match host_result {
                LoadResult::Previous(..) => {
                    panic!("host and target proc macros must be loaded in lock-step")
                }
                LoadResult::Loaded(library) => library,
            };
            (target_result.unwrap(), Some(host_result))
        } else {
            (host_result, None)
        }))
    }

    fn resolve_crate(
        &mut self,
        name: Symbol,
        span: Span,
        dep_kind: CrateDepKind,
    ) -> Option<CrateNum> {
        self.used_extern_options.insert(name);
        match self.maybe_resolve_crate(name, dep_kind, None) {
            Ok(cnum) => {
                self.cstore.set_used_recursively(cnum);
                Some(cnum)
            }
            Err(err) => {
                debug!("failed to resolve crate {} {:?}", name, dep_kind);
                let missing_core =
                    self.maybe_resolve_crate(sym::core, CrateDepKind::Explicit, None).is_err();
                err.report(self.sess, span, missing_core);
                None
            }
        }
    }

    fn maybe_resolve_crate<'b>(
        &'b mut self,
        name: Symbol,
        mut dep_kind: CrateDepKind,
        dep: Option<(&'b CratePaths, &'b CrateDep)>,
    ) -> Result<CrateNum, CrateError> {
        info!("resolving crate `{}`", name);
        if !name.as_str().is_ascii() {
            return Err(CrateError::NonAsciiName(name));
        }
        let (root, hash, host_hash, extra_filename, path_kind, private_dep) = match dep {
            Some((root, dep)) => (
                Some(root),
                Some(dep.hash),
                dep.host_hash,
                Some(&dep.extra_filename[..]),
                PathKind::Dependency,
                Some(dep.is_private),
            ),
            None => (None, None, None, None, PathKind::Crate, None),
        };
        let result = if let Some(cnum) = self.existing_match(name, hash, path_kind) {
            (LoadResult::Previous(cnum), None)
        } else {
            info!("falling back to a load");
            let mut locator = CrateLocator::new(
                self.sess,
                &*self.cstore.metadata_loader,
                name,
                // The all loop is because `--crate-type=rlib --crate-type=rlib` is
                // legal and produces both inside this type.
                self.tcx.crate_types().iter().all(|c| *c == CrateType::Rlib),
                hash,
                extra_filename,
                path_kind,
            );

            match self.load(&mut locator)? {
                Some(res) => (res, None),
                None => {
                    info!("falling back to loading proc_macro");
                    dep_kind = CrateDepKind::MacrosOnly;
                    match self.load_proc_macro(&mut locator, path_kind, host_hash)? {
                        Some(res) => res,
                        None => return Err(locator.into_error(root.cloned())),
                    }
                }
            }
        };

        match result {
            (LoadResult::Previous(cnum), None) => {
                info!("library for `{}` was loaded previously", name);
                // When `private_dep` is none, it indicates the directly dependent crate. If it is
                // not specified by `--extern` on command line parameters, it may be
                // `private-dependency` when `register_crate` is called for the first time. Then it must be updated to
                // `public-dependency` here.
                let private_dep = self.is_private_dep(name.as_str(), private_dep);
                let data = self.cstore.get_crate_data_mut(cnum);
                if data.is_proc_macro_crate() {
                    dep_kind = CrateDepKind::MacrosOnly;
                }
                data.set_dep_kind(cmp::max(data.dep_kind(), dep_kind));
                data.update_and_private_dep(private_dep);
                Ok(cnum)
            }
            (LoadResult::Loaded(library), host_library) => {
                info!("register newly loaded library for `{}`", name);
                self.register_crate(host_library, root, library, dep_kind, name, private_dep, None)
            }
            _ => panic!(),
        }
    }

    fn load(&self, locator: &mut CrateLocator<'_>) -> Result<Option<LoadResult>, CrateError> {
        let Some(library) = locator.maybe_load_library_crate()? else {
            return Ok(None);
        };

        // In the case that we're loading a crate, but not matching
        // against a hash, we could load a crate which has the same hash
        // as an already loaded crate. If this is the case prevent
        // duplicates by just using the first crate.
        //
        // Note that we only do this for target triple crates, though, as we
        // don't want to match a host crate against an equivalent target one
        // already loaded.
        let root = library.metadata.get_root();
        // FIXME: why is this condition necessary? It was adding in #33625 but I
        // don't know why and the original author doesn't remember ...
        let can_reuse_cratenum =
            locator.tuple == self.sess.opts.target_triple || locator.is_proc_macro;
        Ok(Some(if can_reuse_cratenum {
            let mut result = LoadResult::Loaded(library);
            for (cnum, data) in self.cstore.iter_crate_data() {
                if data.name() == root.name() && root.hash() == data.hash() {
                    assert!(locator.hash.is_none());
                    info!("load success, going to previous cnum: {}", cnum);
                    result = LoadResult::Previous(cnum);
                    break;
                }
            }
            result
        } else {
            LoadResult::Loaded(library)
        }))
    }

    // Go through the crate metadata and load any crates that it references
    fn resolve_crate_deps(
        &mut self,
        root: &CratePaths,
        crate_root: &CrateRoot,
        metadata: &MetadataBlob,
        krate: CrateNum,
        dep_kind: CrateDepKind,
    ) -> Result<CrateNumMap, CrateError> {
        debug!("resolving deps of external crate");
        if crate_root.is_proc_macro_crate() {
            return Ok(CrateNumMap::new());
        }

        // The map from crate numbers in the crate we're resolving to local crate numbers.
        // We map 0 and all other holes in the map to our parent crate. The "additional"
        // self-dependencies should be harmless.
        let deps = crate_root.decode_crate_deps(metadata);
        let mut crate_num_map = CrateNumMap::with_capacity(1 + deps.len());
        crate_num_map.push(krate);
        for dep in deps {
            info!(
                "resolving dep crate {} hash: `{}` extra filename: `{}`",
                dep.name, dep.hash, dep.extra_filename
            );
            let dep_kind = match dep_kind {
                CrateDepKind::MacrosOnly => CrateDepKind::MacrosOnly,
                _ => dep.kind,
            };
            let cnum = self.maybe_resolve_crate(dep.name, dep_kind, Some((root, &dep)))?;
            crate_num_map.push(cnum);
        }

        debug!("resolve_crate_deps: cnum_map for {:?} is {:?}", krate, crate_num_map);
        Ok(crate_num_map)
    }

    fn dlsym_proc_macros(
        &self,
        path: &Path,
        stable_crate_id: StableCrateId,
    ) -> Result<&'static [ProcMacro], CrateError> {
        // Check if the file is a WASM module
        #[cfg(target_family = "wasm")]
        {
            if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                return self.dlsym_proc_macros_wasm(path, stable_crate_id);
            }
        }

        // Otherwise use native dylib loading
        let sym_name = self.sess.generate_proc_macro_decls_symbol(stable_crate_id);
        debug!("trying to dlsym proc_macros {} for symbol `{}`", path.display(), sym_name);

        unsafe {
            let result = load_symbol_from_dylib::<*const &[ProcMacro]>(path, &sym_name);
            match result {
                Ok(result) => {
                    debug!("loaded dlsym proc_macros {} for symbol `{}`", path.display(), sym_name);
                    Ok(*result)
                }
                Err(err) => {
                    debug!(
                        "failed to dlsym proc_macros {} for symbol `{}`",
                        path.display(),
                        sym_name
                    );
                    Err(err.into())
                }
            }
        }
    }

    #[cfg(target_family = "wasm")]
    fn dlsym_proc_macros_wasm(
        &self,
        path: &Path,
        _stable_crate_id: StableCrateId,
    ) -> Result<&'static [ProcMacro], CrateError> {
        eprintln!("[CREADER DEBUG] dlsym_proc_macros_wasm called for: {:?}", path);
        use rustc_watt_runtime::WasmMacro;
        use std::fs;

        debug!("loading WASM proc_macros from {}", path.display());

        // Read the .wasm file
        let wasm_bytes = fs::read(path).map_err(|err| {
            CrateError::DlOpen(
                path.display().to_string(),
                format!("failed to read WASM file: {}", err),
            )
        })?;

        // Create WasmMacro instance
        let wasm_macro = WasmMacro::new_owned(wasm_bytes);

        // For now, create a simple test proc macro
        // TODO: Extract actual proc macro metadata from WASM module
        // This will be implemented in Phase 1.3
        let proc_macros = create_wasm_proc_macros(wasm_macro);

        debug!("loaded {} WASM proc_macros from {}", proc_macros.len(), path.display());

        Ok(Box::leak(proc_macros))
    }

    fn inject_panic_runtime(&mut self, krate: &ast::Crate) {
        // If we're only compiling an rlib, then there's no need to select a
        // panic runtime, so we just skip this section entirely.
        let any_non_rlib = self.tcx.crate_types().iter().any(|ct| *ct != CrateType::Rlib);
        if !any_non_rlib {
            info!("panic runtime injection skipped, only generating rlib");
            return;
        }

        // If we need a panic runtime, we try to find an existing one here. At
        // the same time we perform some general validation of the DAG we've got
        // going such as ensuring everything has a compatible panic strategy.
        //
        // The logic for finding the panic runtime here is pretty much the same
        // as the allocator case with the only addition that the panic strategy
        // compilation mode also comes into play.
        let desired_strategy = self.sess.panic_strategy();
        let mut runtime_found = false;
        let mut needs_panic_runtime = attr::contains_name(&krate.attrs, sym::needs_panic_runtime);

        let mut panic_runtimes = Vec::new();
        for (cnum, data) in self.cstore.iter_crate_data() {
            needs_panic_runtime = needs_panic_runtime || data.needs_panic_runtime();
            if data.is_panic_runtime() {
                // Inject a dependency from all #![needs_panic_runtime] to this
                // #![panic_runtime] crate.
                panic_runtimes.push(cnum);
                runtime_found = runtime_found || data.dep_kind() == CrateDepKind::Explicit;
            }
        }
        for cnum in panic_runtimes {
            self.inject_dependency_if(cnum, "a panic runtime", &|data| data.needs_panic_runtime());
        }

        // If an explicitly linked and matching panic runtime was found, or if
        // we just don't need one at all, then we're done here and there's
        // nothing else to do.
        if !needs_panic_runtime || runtime_found {
            return;
        }

        // By this point we know that we (a) need a panic runtime and (b) no
        // panic runtime was explicitly linked. Here we just load an appropriate
        // default runtime for our panic strategy and then inject the
        // dependencies.
        //
        // We may resolve to an already loaded crate (as the crate may not have
        // been explicitly linked prior to this) and we may re-inject
        // dependencies again, but both of those situations are fine.
        //
        // Also note that we have yet to perform validation of the crate graph
        // in terms of everyone has a compatible panic runtime format, that's
        // performed later as part of the `dependency_format` module.
        let name = match desired_strategy {
            PanicStrategy::Unwind => sym::panic_unwind,
            PanicStrategy::Abort => sym::panic_abort,
        };
        info!("panic runtime not found -- loading {}", name);

        let Some(cnum) = self.resolve_crate(name, DUMMY_SP, CrateDepKind::Implicit) else {
            return;
        };
        let data = self.cstore.get_crate_data(cnum);

        // Sanity check the loaded crate to ensure it is indeed a panic runtime
        // and the panic strategy is indeed what we thought it was.
        if !data.is_panic_runtime() {
            self.dcx().emit_err(errors::CrateNotPanicRuntime { crate_name: name });
        }
        if data.required_panic_strategy() != Some(desired_strategy) {
            self.dcx()
                .emit_err(errors::NoPanicStrategy { crate_name: name, strategy: desired_strategy });
        }

        self.cstore.injected_panic_runtime = Some(cnum);
        self.inject_dependency_if(cnum, "a panic runtime", &|data| data.needs_panic_runtime());
    }

    fn inject_profiler_runtime(&mut self, krate: &ast::Crate) {
        if self.sess.opts.unstable_opts.no_profiler_runtime
            || !(self.sess.instrument_coverage() || self.sess.opts.cg.profile_generate.enabled())
        {
            return;
        }

        info!("loading profiler");

        let name = Symbol::intern(&self.sess.opts.unstable_opts.profiler_runtime);
        if name == sym::profiler_builtins && attr::contains_name(&krate.attrs, sym::no_core) {
            self.dcx().emit_err(errors::ProfilerBuiltinsNeedsCore);
        }

        let Some(cnum) = self.resolve_crate(name, DUMMY_SP, CrateDepKind::Implicit) else {
            return;
        };
        let data = self.cstore.get_crate_data(cnum);

        // Sanity check the loaded crate to ensure it is indeed a profiler runtime
        if !data.is_profiler_runtime() {
            self.dcx().emit_err(errors::NotProfilerRuntime { crate_name: name });
        }
    }

    fn inject_allocator_crate(&mut self, krate: &ast::Crate) {
        self.cstore.has_global_allocator = match &*global_allocator_spans(krate) {
            [span1, span2, ..] => {
                self.dcx().emit_err(errors::NoMultipleGlobalAlloc { span2: *span2, span1: *span1 });
                true
            }
            spans => !spans.is_empty(),
        };
        self.cstore.has_alloc_error_handler = match &*alloc_error_handler_spans(krate) {
            [span1, span2, ..] => {
                self.dcx()
                    .emit_err(errors::NoMultipleAllocErrorHandler { span2: *span2, span1: *span1 });
                true
            }
            spans => !spans.is_empty(),
        };

        // Check to see if we actually need an allocator. This desire comes
        // about through the `#![needs_allocator]` attribute and is typically
        // written down in liballoc.
        if !attr::contains_name(&krate.attrs, sym::needs_allocator)
            && !self.cstore.iter_crate_data().any(|(_, data)| data.needs_allocator())
        {
            return;
        }

        // At this point we've determined that we need an allocator. Let's see
        // if our compilation session actually needs an allocator based on what
        // we're emitting.
        let all_rlib = self.tcx.crate_types().iter().all(|ct| matches!(*ct, CrateType::Rlib));
        if all_rlib {
            return;
        }

        // Ok, we need an allocator. Not only that but we're actually going to
        // create an artifact that needs one linked in. Let's go find the one
        // that we're going to link in.
        //
        // First up we check for global allocators. Look at the crate graph here
        // and see what's a global allocator, including if we ourselves are a
        // global allocator.
        let mut global_allocator =
            self.cstore.has_global_allocator.then(|| Symbol::intern("this crate"));
        for (_, data) in self.cstore.iter_crate_data() {
            if data.has_global_allocator() {
                match global_allocator {
                    Some(other_crate) => {
                        self.dcx().emit_err(errors::ConflictingGlobalAlloc {
                            crate_name: data.name(),
                            other_crate_name: other_crate,
                        });
                    }
                    None => global_allocator = Some(data.name()),
                }
            }
        }
        let mut alloc_error_handler =
            self.cstore.has_alloc_error_handler.then(|| Symbol::intern("this crate"));
        for (_, data) in self.cstore.iter_crate_data() {
            if data.has_alloc_error_handler() {
                match alloc_error_handler {
                    Some(other_crate) => {
                        self.dcx().emit_err(errors::ConflictingAllocErrorHandler {
                            crate_name: data.name(),
                            other_crate_name: other_crate,
                        });
                    }
                    None => alloc_error_handler = Some(data.name()),
                }
            }
        }

        if global_allocator.is_some() {
            self.cstore.allocator_kind = Some(AllocatorKind::Global);
        } else {
            // Ok we haven't found a global allocator but we still need an
            // allocator. At this point our allocator request is typically fulfilled
            // by the standard library, denoted by the `#![default_lib_allocator]`
            // attribute.
            if !attr::contains_name(&krate.attrs, sym::default_lib_allocator)
                && !self.cstore.iter_crate_data().any(|(_, data)| data.has_default_lib_allocator())
            {
                self.dcx().emit_err(errors::GlobalAllocRequired);
            }
            self.cstore.allocator_kind = Some(AllocatorKind::Default);
        }

        if alloc_error_handler.is_some() {
            self.cstore.alloc_error_handler_kind = Some(AllocatorKind::Global);
        } else {
            // The alloc crate provides a default allocation error handler if
            // one isn't specified.
            self.cstore.alloc_error_handler_kind = Some(AllocatorKind::Default);
        }
    }

    fn inject_forced_externs(&mut self) {
        for (name, entry) in self.sess.opts.externs.iter() {
            if entry.force {
                let name_interned = Symbol::intern(name);
                if !self.used_extern_options.contains(&name_interned) {
                    self.resolve_crate(name_interned, DUMMY_SP, CrateDepKind::Explicit);
                }
            }
        }
    }

    fn inject_dependency_if(
        &mut self,
        krate: CrateNum,
        what: &str,
        needs_dep: &dyn Fn(&CrateMetadata) -> bool,
    ) {
        // Don't perform this validation if the session has errors, as one of
        // those errors may indicate a circular dependency which could cause
        // this to stack overflow.
        if self.dcx().has_errors().is_some() {
            return;
        }

        // Before we inject any dependencies, make sure we don't inject a
        // circular dependency by validating that this crate doesn't
        // transitively depend on any crates satisfying `needs_dep`.
        for dep in self.cstore.crate_dependencies_in_reverse_postorder(krate) {
            let data = self.cstore.get_crate_data(dep);
            if needs_dep(&data) {
                self.dcx().emit_err(errors::NoTransitiveNeedsDep {
                    crate_name: self.cstore.get_crate_data(krate).name(),
                    needs_crate_name: what,
                    deps_crate_name: data.name(),
                });
            }
        }

        // All crates satisfying `needs_dep` do not explicitly depend on the
        // crate provided for this compile, but in order for this compilation to
        // be successfully linked we need to inject a dependency (to order the
        // crates on the command line correctly).
        for (cnum, data) in self.cstore.iter_crate_data_mut() {
            if needs_dep(data) {
                info!("injecting a dep from {} to {}", cnum, krate);
                data.add_dependency(krate);
            }
        }
    }

    fn report_unused_deps(&mut self, krate: &ast::Crate) {
        // Make a point span rather than covering the whole file
        let span = krate.spans.inner_span.shrink_to_lo();
        // Complain about anything left over
        for (name, entry) in self.sess.opts.externs.iter() {
            if let ExternLocation::FoundInLibrarySearchDirectories = entry.location {
                // Don't worry about pathless `--extern foo` sysroot references
                continue;
            }
            if entry.nounused_dep || entry.force {
                // We're not worried about this one
                continue;
            }
            let name_interned = Symbol::intern(name);
            if self.used_extern_options.contains(&name_interned) {
                continue;
            }

            // Got a real unused --extern
            if self.sess.opts.json_unused_externs.is_enabled() {
                self.cstore.unused_externs.push(name_interned);
                continue;
            }

            self.sess.psess.buffer_lint(
                lint::builtin::UNUSED_CRATE_DEPENDENCIES,
                span,
                ast::CRATE_NODE_ID,
                BuiltinLintDiag::UnusedCrateDependency {
                    extern_crate: name_interned,
                    local_crate: self.tcx.crate_name(LOCAL_CRATE),
                },
            );
        }
    }

    fn report_future_incompatible_deps(&self, krate: &ast::Crate) {
        let name = self.tcx.crate_name(LOCAL_CRATE);

        if name.as_str() == "wasm_bindgen" {
            let major = env::var("CARGO_PKG_VERSION_MAJOR")
                .ok()
                .and_then(|major| u64::from_str(&major).ok());
            let minor = env::var("CARGO_PKG_VERSION_MINOR")
                .ok()
                .and_then(|minor| u64::from_str(&minor).ok());
            let patch = env::var("CARGO_PKG_VERSION_PATCH")
                .ok()
                .and_then(|patch| u64::from_str(&patch).ok());

            match (major, minor, patch) {
                // v1 or bigger is valid.
                (Some(1..), _, _) => return,
                // v0.3 or bigger is valid.
                (Some(0), Some(3..), _) => return,
                // v0.2.88 or bigger is valid.
                (Some(0), Some(2), Some(88..)) => return,
                // Not using Cargo.
                (None, None, None) => return,
                _ => (),
            }

            // Make a point span rather than covering the whole file
            let span = krate.spans.inner_span.shrink_to_lo();

            self.sess.psess.buffer_lint(
                lint::builtin::WASM_C_ABI,
                span,
                ast::CRATE_NODE_ID,
                BuiltinLintDiag::WasmCAbi,
            );
        }
    }

    pub fn postprocess(&mut self, krate: &ast::Crate) {
        self.inject_forced_externs();
        self.inject_profiler_runtime(krate);
        self.inject_allocator_crate(krate);
        self.inject_panic_runtime(krate);

        self.report_unused_deps(krate);
        self.report_future_incompatible_deps(krate);

        info!("{:?}", CrateDump(self.cstore));
    }

    pub fn process_extern_crate(
        &mut self,
        item: &ast::Item,
        def_id: LocalDefId,
        definitions: &Definitions,
    ) -> Option<CrateNum> {
        match item.kind {
            ast::ItemKind::ExternCrate(orig_name) => {
                debug!(
                    "resolving extern crate stmt. ident: {} orig_name: {:?}",
                    item.ident, orig_name
                );
                let name = match orig_name {
                    Some(orig_name) => {
                        validate_crate_name(self.sess, orig_name, Some(item.span));
                        orig_name
                    }
                    None => item.ident.name,
                };
                let dep_kind = if attr::contains_name(&item.attrs, sym::no_link) {
                    CrateDepKind::MacrosOnly
                } else {
                    CrateDepKind::Explicit
                };

                let cnum = self.resolve_crate(name, item.span, dep_kind)?;

                let path_len = definitions.def_path(def_id).data.len();
                self.cstore.update_extern_crate(cnum, ExternCrate {
                    src: ExternCrateSource::Extern(def_id.to_def_id()),
                    span: item.span,
                    path_len,
                    dependency_of: LOCAL_CRATE,
                });
                Some(cnum)
            }
            _ => bug!(),
        }
    }

    pub fn process_path_extern(&mut self, name: Symbol, span: Span) -> Option<CrateNum> {
        let cnum = self.resolve_crate(name, span, CrateDepKind::Explicit)?;

        self.cstore.update_extern_crate(cnum, ExternCrate {
            src: ExternCrateSource::Path,
            span,
            // to have the least priority in `update_extern_crate`
            path_len: usize::MAX,
            dependency_of: LOCAL_CRATE,
        });

        Some(cnum)
    }

    pub fn maybe_process_path_extern(&mut self, name: Symbol) -> Option<CrateNum> {
        self.maybe_resolve_crate(name, CrateDepKind::Explicit, None).ok()
    }
}

fn global_allocator_spans(krate: &ast::Crate) -> Vec<Span> {
    struct Finder {
        name: Symbol,
        spans: Vec<Span>,
    }
    impl<'ast> visit::Visitor<'ast> for Finder {
        fn visit_item(&mut self, item: &'ast ast::Item) {
            if item.ident.name == self.name
                && attr::contains_name(&item.attrs, sym::rustc_std_internal_symbol)
            {
                self.spans.push(item.span);
            }
            visit::walk_item(self, item)
        }
    }

    let name = Symbol::intern(&global_fn_name(sym::alloc));
    let mut f = Finder { name, spans: Vec::new() };
    visit::walk_crate(&mut f, krate);
    f.spans
}

fn alloc_error_handler_spans(krate: &ast::Crate) -> Vec<Span> {
    struct Finder {
        name: Symbol,
        spans: Vec<Span>,
    }
    impl<'ast> visit::Visitor<'ast> for Finder {
        fn visit_item(&mut self, item: &'ast ast::Item) {
            if item.ident.name == self.name
                && attr::contains_name(&item.attrs, sym::rustc_std_internal_symbol)
            {
                self.spans.push(item.span);
            }
            visit::walk_item(self, item)
        }
    }

    let name = Symbol::intern(alloc_error_handler_name(AllocatorKind::Global));
    let mut f = Finder { name, spans: Vec::new() };
    visit::walk_crate(&mut f, krate);
    f.spans
}

#[cfg(any(unix, windows))]
fn format_dlopen_err(e: &(dyn std::error::Error + 'static)) -> String {
    e.sources().map(|e| format!(": {e}")).collect()
}

#[cfg(any(unix, windows))]
fn attempt_load_dylib(path: &Path) -> Result<libloading::Library, libloading::Error> {
    #[cfg(target_os = "aix")]
    if let Some(ext) = path.extension()
        && ext.eq("a")
    {
        // On AIX, we ship all libraries as .a big_af archive
        // the expected format is lib<name>.a(libname.so) for the actual
        // dynamic library
        let library_name = path.file_stem().expect("expect a library name");
        let mut archive_member = OsString::from("a(");
        archive_member.push(library_name);
        archive_member.push(".so)");
        let new_path = path.with_extension(archive_member);

        // On AIX, we need RTLD_MEMBER to dlopen an archived shared
        let flags = libc::RTLD_LAZY | libc::RTLD_LOCAL | libc::RTLD_MEMBER;
        return unsafe { libloading::os::unix::Library::open(Some(&new_path), flags) }
            .map(|lib| lib.into());
    }

    unsafe { libloading::Library::new(&path) }
}

// On Windows the compiler would sometimes intermittently fail to open the
// proc-macro DLL with `Error::LoadLibraryExW`. It is suspected that something in the
// system still holds a lock on the file, so we retry a few times before calling it
// an error.
#[cfg(any(unix, windows))]
fn load_dylib(path: &Path, max_attempts: usize) -> Result<libloading::Library, String> {
    assert!(max_attempts > 0);

    let mut last_error = None;

    for attempt in 0..max_attempts {
        debug!("Attempt to load proc-macro `{}`.", path.display());
        match attempt_load_dylib(path) {
            Ok(lib) => {
                if attempt > 0 {
                    debug!(
                        "Loaded proc-macro `{}` after {} attempts.",
                        path.display(),
                        attempt + 1
                    );
                }
                return Ok(lib);
            }
            Err(err) => {
                // Only try to recover from this specific error.
                if !matches!(err, libloading::Error::LoadLibraryExW { .. }) {
                    debug!("Failed to load proc-macro `{}`. Not retrying", path.display());
                    let err = format_dlopen_err(&err);
                    // We include the path of the dylib in the error ourselves, so
                    // if it's in the error, we strip it.
                    if let Some(err) = err.strip_prefix(&format!(": {}", path.display())) {
                        return Err(err.to_string());
                    }
                    return Err(err);
                }

                last_error = Some(err);
                std::thread::sleep(std::time::Duration::from_millis(100));
                debug!("Failed to load proc-macro `{}`. Retrying.", path.display());
            }
        }
    }

    debug!("Failed to load proc-macro `{}` even after {} attempts.", path.display(), max_attempts);

    let last_error = last_error.unwrap();
    let message = if let Some(src) = last_error.source() {
        format!("{} ({src}) (retried {max_attempts} times)", format_dlopen_err(&last_error))
    } else {
        format!("{} (retried {max_attempts} times)", format_dlopen_err(&last_error))
    };
    Err(message)
}

/// Helper function to create ProcMacro instances from a WASM module
///
/// This function extracts proc macro metadata from the WASM module and creates
/// the appropriate ProcMacro enum variants that bridge to the watt runtime.
#[cfg(target_family = "wasm")]
fn create_wasm_proc_macros(
    wasm_macro: rustc_watt_runtime::WasmMacro,
) -> Box<[ProcMacro]> {
    eprintln!("[CREADER DEBUG] create_wasm_proc_macros called");
    use proc_macro::bridge::client::{Client, ProcMacro};
    use proc_macro::TokenStream;
    use rustc_watt_runtime::metadata::{ProcMacroMetadata, extract_proc_macro_metadata};
    use std::sync::{Mutex, OnceLock};

    // Slot-based registry for WASM proc macros
    // This allows us to use zero-sized function items instead of closures
    #[derive(Copy, Clone)]
    struct SlotData {
        wasm_macro: &'static rustc_watt_runtime::WasmMacro,
        function_name: &'static str,
        slot_type: SlotType,
    }

    #[derive(Copy, Clone)]
    enum SlotType {
        Derive,
        Attr,
        Bang,
    }

    static SLOTS: OnceLock<Mutex<Vec<Option<SlotData>>>> = OnceLock::new();

    fn get_slots() -> &'static Mutex<Vec<Option<SlotData>>> {
        SLOTS.get_or_init(|| Mutex::new(vec![None; 256]))
    }

    fn allocate_slot(data: SlotData) -> usize {
        let mut slots = get_slots().lock().unwrap();
        for (i, slot) in slots.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(data);
                return i;
            }
        }
        panic!("Ran out of proc macro slots (max 256)");
    }

    fn slot_0_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[0].as_ref().expect("Slot 0 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_0_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[0].as_ref().expect("Slot 0 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_0_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[0].as_ref().expect("Slot 0 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_1_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[1].as_ref().expect("Slot 1 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_1_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[1].as_ref().expect("Slot 1 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_1_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[1].as_ref().expect("Slot 1 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_2_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[2].as_ref().expect("Slot 2 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_2_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[2].as_ref().expect("Slot 2 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_2_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[2].as_ref().expect("Slot 2 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_3_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[3].as_ref().expect("Slot 3 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_3_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[3].as_ref().expect("Slot 3 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_3_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[3].as_ref().expect("Slot 3 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_4_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[4].as_ref().expect("Slot 4 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_4_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[4].as_ref().expect("Slot 4 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_4_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[4].as_ref().expect("Slot 4 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_5_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[5].as_ref().expect("Slot 5 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_5_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[5].as_ref().expect("Slot 5 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_5_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[5].as_ref().expect("Slot 5 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_6_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[6].as_ref().expect("Slot 6 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_6_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[6].as_ref().expect("Slot 6 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_6_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[6].as_ref().expect("Slot 6 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_7_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[7].as_ref().expect("Slot 7 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_7_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[7].as_ref().expect("Slot 7 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_7_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[7].as_ref().expect("Slot 7 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_8_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[8].as_ref().expect("Slot 8 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_8_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[8].as_ref().expect("Slot 8 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_8_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[8].as_ref().expect("Slot 8 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_9_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[9].as_ref().expect("Slot 9 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_9_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[9].as_ref().expect("Slot 9 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_9_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[9].as_ref().expect("Slot 9 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_10_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[10].as_ref().expect("Slot 10 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_10_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[10].as_ref().expect("Slot 10 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_10_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[10].as_ref().expect("Slot 10 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_11_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[11].as_ref().expect("Slot 11 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_11_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[11].as_ref().expect("Slot 11 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_11_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[11].as_ref().expect("Slot 11 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_12_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[12].as_ref().expect("Slot 12 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_12_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[12].as_ref().expect("Slot 12 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_12_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[12].as_ref().expect("Slot 12 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_13_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[13].as_ref().expect("Slot 13 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_13_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[13].as_ref().expect("Slot 13 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_13_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[13].as_ref().expect("Slot 13 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_14_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[14].as_ref().expect("Slot 14 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_14_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[14].as_ref().expect("Slot 14 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_14_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[14].as_ref().expect("Slot 14 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_15_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[15].as_ref().expect("Slot 15 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_15_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[15].as_ref().expect("Slot 15 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_15_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[15].as_ref().expect("Slot 15 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_16_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[16].as_ref().expect("Slot 16 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_16_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[16].as_ref().expect("Slot 16 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_16_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[16].as_ref().expect("Slot 16 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_17_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[17].as_ref().expect("Slot 17 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_17_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[17].as_ref().expect("Slot 17 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_17_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[17].as_ref().expect("Slot 17 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_18_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[18].as_ref().expect("Slot 18 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_18_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[18].as_ref().expect("Slot 18 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_18_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[18].as_ref().expect("Slot 18 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_19_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[19].as_ref().expect("Slot 19 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_19_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[19].as_ref().expect("Slot 19 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_19_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[19].as_ref().expect("Slot 19 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_20_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[20].as_ref().expect("Slot 20 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_20_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[20].as_ref().expect("Slot 20 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_20_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[20].as_ref().expect("Slot 20 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_21_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[21].as_ref().expect("Slot 21 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_21_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[21].as_ref().expect("Slot 21 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_21_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[21].as_ref().expect("Slot 21 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_22_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[22].as_ref().expect("Slot 22 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_22_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[22].as_ref().expect("Slot 22 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_22_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[22].as_ref().expect("Slot 22 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_23_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[23].as_ref().expect("Slot 23 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_23_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[23].as_ref().expect("Slot 23 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_23_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[23].as_ref().expect("Slot 23 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_24_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[24].as_ref().expect("Slot 24 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_24_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[24].as_ref().expect("Slot 24 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_24_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[24].as_ref().expect("Slot 24 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_25_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[25].as_ref().expect("Slot 25 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_25_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[25].as_ref().expect("Slot 25 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_25_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[25].as_ref().expect("Slot 25 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_26_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[26].as_ref().expect("Slot 26 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_26_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[26].as_ref().expect("Slot 26 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_26_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[26].as_ref().expect("Slot 26 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_27_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[27].as_ref().expect("Slot 27 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_27_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[27].as_ref().expect("Slot 27 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_27_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[27].as_ref().expect("Slot 27 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_28_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[28].as_ref().expect("Slot 28 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_28_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[28].as_ref().expect("Slot 28 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_28_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[28].as_ref().expect("Slot 28 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_29_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[29].as_ref().expect("Slot 29 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_29_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[29].as_ref().expect("Slot 29 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_29_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[29].as_ref().expect("Slot 29 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_30_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[30].as_ref().expect("Slot 30 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_30_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[30].as_ref().expect("Slot 30 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_30_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[30].as_ref().expect("Slot 30 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_31_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[31].as_ref().expect("Slot 31 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_31_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[31].as_ref().expect("Slot 31 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_31_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[31].as_ref().expect("Slot 31 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_32_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[32].as_ref().expect("Slot 32 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_32_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[32].as_ref().expect("Slot 32 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_32_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[32].as_ref().expect("Slot 32 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_33_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[33].as_ref().expect("Slot 33 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_33_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[33].as_ref().expect("Slot 33 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_33_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[33].as_ref().expect("Slot 33 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_34_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[34].as_ref().expect("Slot 34 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_34_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[34].as_ref().expect("Slot 34 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_34_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[34].as_ref().expect("Slot 34 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_35_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[35].as_ref().expect("Slot 35 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_35_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[35].as_ref().expect("Slot 35 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_35_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[35].as_ref().expect("Slot 35 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_36_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[36].as_ref().expect("Slot 36 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_36_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[36].as_ref().expect("Slot 36 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_36_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[36].as_ref().expect("Slot 36 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_37_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[37].as_ref().expect("Slot 37 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_37_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[37].as_ref().expect("Slot 37 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_37_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[37].as_ref().expect("Slot 37 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_38_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[38].as_ref().expect("Slot 38 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_38_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[38].as_ref().expect("Slot 38 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_38_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[38].as_ref().expect("Slot 38 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_39_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[39].as_ref().expect("Slot 39 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_39_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[39].as_ref().expect("Slot 39 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_39_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[39].as_ref().expect("Slot 39 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_40_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[40].as_ref().expect("Slot 40 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_40_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[40].as_ref().expect("Slot 40 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_40_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[40].as_ref().expect("Slot 40 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_41_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[41].as_ref().expect("Slot 41 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_41_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[41].as_ref().expect("Slot 41 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_41_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[41].as_ref().expect("Slot 41 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_42_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[42].as_ref().expect("Slot 42 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_42_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[42].as_ref().expect("Slot 42 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_42_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[42].as_ref().expect("Slot 42 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_43_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[43].as_ref().expect("Slot 43 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_43_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[43].as_ref().expect("Slot 43 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_43_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[43].as_ref().expect("Slot 43 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_44_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[44].as_ref().expect("Slot 44 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_44_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[44].as_ref().expect("Slot 44 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_44_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[44].as_ref().expect("Slot 44 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_45_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[45].as_ref().expect("Slot 45 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_45_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[45].as_ref().expect("Slot 45 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_45_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[45].as_ref().expect("Slot 45 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_46_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[46].as_ref().expect("Slot 46 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_46_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[46].as_ref().expect("Slot 46 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_46_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[46].as_ref().expect("Slot 46 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_47_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[47].as_ref().expect("Slot 47 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_47_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[47].as_ref().expect("Slot 47 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_47_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[47].as_ref().expect("Slot 47 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_48_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[48].as_ref().expect("Slot 48 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_48_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[48].as_ref().expect("Slot 48 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_48_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[48].as_ref().expect("Slot 48 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_49_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[49].as_ref().expect("Slot 49 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_49_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[49].as_ref().expect("Slot 49 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_49_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[49].as_ref().expect("Slot 49 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_50_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[50].as_ref().expect("Slot 50 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_50_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[50].as_ref().expect("Slot 50 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_50_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[50].as_ref().expect("Slot 50 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_51_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[51].as_ref().expect("Slot 51 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_51_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[51].as_ref().expect("Slot 51 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_51_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[51].as_ref().expect("Slot 51 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_52_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[52].as_ref().expect("Slot 52 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_52_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[52].as_ref().expect("Slot 52 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_52_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[52].as_ref().expect("Slot 52 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_53_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[53].as_ref().expect("Slot 53 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_53_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[53].as_ref().expect("Slot 53 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_53_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[53].as_ref().expect("Slot 53 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_54_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[54].as_ref().expect("Slot 54 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_54_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[54].as_ref().expect("Slot 54 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_54_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[54].as_ref().expect("Slot 54 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_55_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[55].as_ref().expect("Slot 55 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_55_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[55].as_ref().expect("Slot 55 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_55_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[55].as_ref().expect("Slot 55 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_56_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[56].as_ref().expect("Slot 56 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_56_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[56].as_ref().expect("Slot 56 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_56_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[56].as_ref().expect("Slot 56 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_57_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[57].as_ref().expect("Slot 57 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_57_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[57].as_ref().expect("Slot 57 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_57_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[57].as_ref().expect("Slot 57 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_58_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[58].as_ref().expect("Slot 58 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_58_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[58].as_ref().expect("Slot 58 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_58_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[58].as_ref().expect("Slot 58 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_59_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[59].as_ref().expect("Slot 59 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_59_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[59].as_ref().expect("Slot 59 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_59_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[59].as_ref().expect("Slot 59 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_60_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[60].as_ref().expect("Slot 60 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_60_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[60].as_ref().expect("Slot 60 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_60_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[60].as_ref().expect("Slot 60 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_61_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[61].as_ref().expect("Slot 61 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_61_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[61].as_ref().expect("Slot 61 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_61_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[61].as_ref().expect("Slot 61 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_62_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[62].as_ref().expect("Slot 62 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_62_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[62].as_ref().expect("Slot 62 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_62_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[62].as_ref().expect("Slot 62 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }
    fn slot_63_derive(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[63].as_ref().expect("Slot 63 not initialized");
        data.wasm_macro.proc_macro_derive(data.function_name, input)
    }
    fn slot_63_attr(args: TokenStream, input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[63].as_ref().expect("Slot 63 not initialized");
        data.wasm_macro.proc_macro_attribute(data.function_name, args, input)
    }
    fn slot_63_bang(input: TokenStream) -> TokenStream {
        let slots = get_slots().lock().unwrap();
        let data = slots[63].as_ref().expect("Slot 63 not initialized");
        data.wasm_macro.proc_macro(data.function_name, input)
    }

    fn make_derive_client(slot: usize) -> Client<TokenStream, TokenStream> {
        match slot {
            0 => Client::expand1(slot_0_derive),
            1 => Client::expand1(slot_1_derive),
            2 => Client::expand1(slot_2_derive),
            3 => Client::expand1(slot_3_derive),
            4 => Client::expand1(slot_4_derive),
            5 => Client::expand1(slot_5_derive),
            6 => Client::expand1(slot_6_derive),
            7 => Client::expand1(slot_7_derive),
            8 => Client::expand1(slot_8_derive),
            9 => Client::expand1(slot_9_derive),
            10 => Client::expand1(slot_10_derive),
            11 => Client::expand1(slot_11_derive),
            12 => Client::expand1(slot_12_derive),
            13 => Client::expand1(slot_13_derive),
            14 => Client::expand1(slot_14_derive),
            15 => Client::expand1(slot_15_derive),
            16 => Client::expand1(slot_16_derive),
            17 => Client::expand1(slot_17_derive),
            18 => Client::expand1(slot_18_derive),
            19 => Client::expand1(slot_19_derive),
            20 => Client::expand1(slot_20_derive),
            21 => Client::expand1(slot_21_derive),
            22 => Client::expand1(slot_22_derive),
            23 => Client::expand1(slot_23_derive),
            24 => Client::expand1(slot_24_derive),
            25 => Client::expand1(slot_25_derive),
            26 => Client::expand1(slot_26_derive),
            27 => Client::expand1(slot_27_derive),
            28 => Client::expand1(slot_28_derive),
            29 => Client::expand1(slot_29_derive),
            30 => Client::expand1(slot_30_derive),
            31 => Client::expand1(slot_31_derive),
            32 => Client::expand1(slot_32_derive),
            33 => Client::expand1(slot_33_derive),
            34 => Client::expand1(slot_34_derive),
            35 => Client::expand1(slot_35_derive),
            36 => Client::expand1(slot_36_derive),
            37 => Client::expand1(slot_37_derive),
            38 => Client::expand1(slot_38_derive),
            39 => Client::expand1(slot_39_derive),
            40 => Client::expand1(slot_40_derive),
            41 => Client::expand1(slot_41_derive),
            42 => Client::expand1(slot_42_derive),
            43 => Client::expand1(slot_43_derive),
            44 => Client::expand1(slot_44_derive),
            45 => Client::expand1(slot_45_derive),
            46 => Client::expand1(slot_46_derive),
            47 => Client::expand1(slot_47_derive),
            48 => Client::expand1(slot_48_derive),
            49 => Client::expand1(slot_49_derive),
            50 => Client::expand1(slot_50_derive),
            51 => Client::expand1(slot_51_derive),
            52 => Client::expand1(slot_52_derive),
            53 => Client::expand1(slot_53_derive),
            54 => Client::expand1(slot_54_derive),
            55 => Client::expand1(slot_55_derive),
            56 => Client::expand1(slot_56_derive),
            57 => Client::expand1(slot_57_derive),
            58 => Client::expand1(slot_58_derive),
            59 => Client::expand1(slot_59_derive),
            60 => Client::expand1(slot_60_derive),
            61 => Client::expand1(slot_61_derive),
            62 => Client::expand1(slot_62_derive),
            63 => Client::expand1(slot_63_derive),
            _ => panic!("Invalid slot: {}", slot),
        }
    }

    fn make_attr_client(slot: usize) -> Client<(TokenStream, TokenStream), TokenStream> {
        match slot {
            0 => Client::expand2(slot_0_attr),
            1 => Client::expand2(slot_1_attr),
            2 => Client::expand2(slot_2_attr),
            3 => Client::expand2(slot_3_attr),
            4 => Client::expand2(slot_4_attr),
            5 => Client::expand2(slot_5_attr),
            6 => Client::expand2(slot_6_attr),
            7 => Client::expand2(slot_7_attr),
            8 => Client::expand2(slot_8_attr),
            9 => Client::expand2(slot_9_attr),
            10 => Client::expand2(slot_10_attr),
            11 => Client::expand2(slot_11_attr),
            12 => Client::expand2(slot_12_attr),
            13 => Client::expand2(slot_13_attr),
            14 => Client::expand2(slot_14_attr),
            15 => Client::expand2(slot_15_attr),
            16 => Client::expand2(slot_16_attr),
            17 => Client::expand2(slot_17_attr),
            18 => Client::expand2(slot_18_attr),
            19 => Client::expand2(slot_19_attr),
            20 => Client::expand2(slot_20_attr),
            21 => Client::expand2(slot_21_attr),
            22 => Client::expand2(slot_22_attr),
            23 => Client::expand2(slot_23_attr),
            24 => Client::expand2(slot_24_attr),
            25 => Client::expand2(slot_25_attr),
            26 => Client::expand2(slot_26_attr),
            27 => Client::expand2(slot_27_attr),
            28 => Client::expand2(slot_28_attr),
            29 => Client::expand2(slot_29_attr),
            30 => Client::expand2(slot_30_attr),
            31 => Client::expand2(slot_31_attr),
            32 => Client::expand2(slot_32_attr),
            33 => Client::expand2(slot_33_attr),
            34 => Client::expand2(slot_34_attr),
            35 => Client::expand2(slot_35_attr),
            36 => Client::expand2(slot_36_attr),
            37 => Client::expand2(slot_37_attr),
            38 => Client::expand2(slot_38_attr),
            39 => Client::expand2(slot_39_attr),
            40 => Client::expand2(slot_40_attr),
            41 => Client::expand2(slot_41_attr),
            42 => Client::expand2(slot_42_attr),
            43 => Client::expand2(slot_43_attr),
            44 => Client::expand2(slot_44_attr),
            45 => Client::expand2(slot_45_attr),
            46 => Client::expand2(slot_46_attr),
            47 => Client::expand2(slot_47_attr),
            48 => Client::expand2(slot_48_attr),
            49 => Client::expand2(slot_49_attr),
            50 => Client::expand2(slot_50_attr),
            51 => Client::expand2(slot_51_attr),
            52 => Client::expand2(slot_52_attr),
            53 => Client::expand2(slot_53_attr),
            54 => Client::expand2(slot_54_attr),
            55 => Client::expand2(slot_55_attr),
            56 => Client::expand2(slot_56_attr),
            57 => Client::expand2(slot_57_attr),
            58 => Client::expand2(slot_58_attr),
            59 => Client::expand2(slot_59_attr),
            60 => Client::expand2(slot_60_attr),
            61 => Client::expand2(slot_61_attr),
            62 => Client::expand2(slot_62_attr),
            63 => Client::expand2(slot_63_attr),
            _ => panic!("Invalid slot: {}", slot),
        }
    }

    fn make_bang_client(slot: usize) -> Client<TokenStream, TokenStream> {
        match slot {
            0 => Client::expand1(slot_0_bang),
            1 => Client::expand1(slot_1_bang),
            2 => Client::expand1(slot_2_bang),
            3 => Client::expand1(slot_3_bang),
            4 => Client::expand1(slot_4_bang),
            5 => Client::expand1(slot_5_bang),
            6 => Client::expand1(slot_6_bang),
            7 => Client::expand1(slot_7_bang),
            8 => Client::expand1(slot_8_bang),
            9 => Client::expand1(slot_9_bang),
            10 => Client::expand1(slot_10_bang),
            11 => Client::expand1(slot_11_bang),
            12 => Client::expand1(slot_12_bang),
            13 => Client::expand1(slot_13_bang),
            14 => Client::expand1(slot_14_bang),
            15 => Client::expand1(slot_15_bang),
            16 => Client::expand1(slot_16_bang),
            17 => Client::expand1(slot_17_bang),
            18 => Client::expand1(slot_18_bang),
            19 => Client::expand1(slot_19_bang),
            20 => Client::expand1(slot_20_bang),
            21 => Client::expand1(slot_21_bang),
            22 => Client::expand1(slot_22_bang),
            23 => Client::expand1(slot_23_bang),
            24 => Client::expand1(slot_24_bang),
            25 => Client::expand1(slot_25_bang),
            26 => Client::expand1(slot_26_bang),
            27 => Client::expand1(slot_27_bang),
            28 => Client::expand1(slot_28_bang),
            29 => Client::expand1(slot_29_bang),
            30 => Client::expand1(slot_30_bang),
            31 => Client::expand1(slot_31_bang),
            32 => Client::expand1(slot_32_bang),
            33 => Client::expand1(slot_33_bang),
            34 => Client::expand1(slot_34_bang),
            35 => Client::expand1(slot_35_bang),
            36 => Client::expand1(slot_36_bang),
            37 => Client::expand1(slot_37_bang),
            38 => Client::expand1(slot_38_bang),
            39 => Client::expand1(slot_39_bang),
            40 => Client::expand1(slot_40_bang),
            41 => Client::expand1(slot_41_bang),
            42 => Client::expand1(slot_42_bang),
            43 => Client::expand1(slot_43_bang),
            44 => Client::expand1(slot_44_bang),
            45 => Client::expand1(slot_45_bang),
            46 => Client::expand1(slot_46_bang),
            47 => Client::expand1(slot_47_bang),
            48 => Client::expand1(slot_48_bang),
            49 => Client::expand1(slot_49_bang),
            50 => Client::expand1(slot_50_bang),
            51 => Client::expand1(slot_51_bang),
            52 => Client::expand1(slot_52_bang),
            53 => Client::expand1(slot_53_bang),
            54 => Client::expand1(slot_54_bang),
            55 => Client::expand1(slot_55_bang),
            56 => Client::expand1(slot_56_bang),
            57 => Client::expand1(slot_57_bang),
            58 => Client::expand1(slot_58_bang),
            59 => Client::expand1(slot_59_bang),
            60 => Client::expand1(slot_60_bang),
            61 => Client::expand1(slot_61_bang),
            62 => Client::expand1(slot_62_bang),
            63 => Client::expand1(slot_63_bang),
            _ => panic!("Invalid slot: {}", slot),
        }
    }

    // Extract metadata from the WASM module's custom section
    eprintln!("[CREADER DEBUG] Extracting proc macro metadata from WASM...");
    let metadata = extract_proc_macro_metadata(wasm_macro.wasm_bytes());
    eprintln!("[CREADER DEBUG] Found {} metadata entries", metadata.len());

    if metadata.is_empty() {
        eprintln!("[CREADER DEBUG] No proc macro metadata found - returning empty");
        debug!(
            "No proc macro metadata found in WASM module. \
             Make sure the proc macro crate includes the .rustc_proc_macro_decls custom section."
        );
        return Box::new([]);
    }

    // Leak the WasmMacro to get a 'static reference
    let wasm_macro: &'static rustc_watt_runtime::WasmMacro = Box::leak(Box::new(wasm_macro));

    // Create ProcMacro instances for each metadata entry
    let proc_macros: Vec<ProcMacro> = metadata
        .into_iter()
        .map(|meta| {
            let function_name: &'static str = Box::leak(meta.function_name().to_string().into_boxed_str());

            match meta {
                ProcMacroMetadata::CustomDerive { trait_name, attributes, .. } => {
                    let slot = allocate_slot(SlotData {
                        wasm_macro,
                        function_name,
                        slot_type: SlotType::Derive,
                    });

                    let static_attrs: &'static [&'static str] = {
                        let attrs: Vec<&'static str> = attributes
                            .into_iter()
                            .map(|s| Box::leak(s.into_boxed_str()) as &'static str)
                            .collect();
                        Box::leak(attrs.into_boxed_slice())
                    };

                    let static_trait_name: &'static str = Box::leak(trait_name.into_boxed_str());

                    ProcMacro::CustomDerive {
                        trait_name: static_trait_name,
                        attributes: static_attrs,
                        client: make_derive_client(slot),
                    }
                }
                ProcMacroMetadata::Attr { name, .. } => {
                    let slot = allocate_slot(SlotData {
                        wasm_macro,
                        function_name,
                        slot_type: SlotType::Attr,
                    });

                    let static_name: &'static str = Box::leak(name.into_boxed_str());

                    ProcMacro::Attr {
                        name: static_name,
                        client: make_attr_client(slot),
                    }
                }
                ProcMacroMetadata::Bang { name, .. } => {
                    let slot = allocate_slot(SlotData {
                        wasm_macro,
                        function_name,
                        slot_type: SlotType::Bang,
                    });

                    let static_name: &'static str = Box::leak(name.into_boxed_str());

                    ProcMacro::Bang {
                        name: static_name,
                        client: make_bang_client(slot),
                    }
                }
            }
        })
        .collect();

    debug!("Created {} proc macro instances from WASM module", proc_macros.len());

    proc_macros.into_boxed_slice()
}

pub enum DylibError {
    DlOpen(String, String),
    DlSym(String, String),
}

impl From<DylibError> for CrateError {
    fn from(err: DylibError) -> CrateError {
        match err {
            DylibError::DlOpen(path, err) => CrateError::DlOpen(path, err),
            DylibError::DlSym(path, err) => CrateError::DlSym(path, err),
        }
    }
}

#[cfg(any(unix, windows))]
pub unsafe fn load_symbol_from_dylib<T: Copy>(
    path: &Path,
    sym_name: &str,
) -> Result<T, DylibError> {
    // Make sure the path contains a / or the linker will search for it.
    let path = try_canonicalize(path).unwrap();
    let lib =
        load_dylib(&path, 5).map_err(|err| DylibError::DlOpen(path.display().to_string(), err))?;

    let sym = unsafe { lib.get::<T>(sym_name.as_bytes()) }
        .map_err(|err| DylibError::DlSym(path.display().to_string(), format_dlopen_err(&err)))?;

    // Intentionally leak the dynamic library. We can't ever unload it
    // since the library can make things that will live arbitrarily long.
    let sym = unsafe { sym.into_raw() };
    std::mem::forget(lib);

    Ok(*sym)
}

#[cfg(not(any(unix, windows)))]
pub unsafe fn load_symbol_from_dylib<T: Copy>(
    path: &Path,
    _sym_name: &str,
) -> Result<T, DylibError> {
    Err(DylibError::DlOpen(
        path.display().to_string(),
        "dlopen not supported on this platform".to_owned(),
    ))
}
