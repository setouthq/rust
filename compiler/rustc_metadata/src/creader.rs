//! Validates all used crates and extern libraries and loads their metadata

#[cfg(any(unix, windows))]
use std::error::Error;

use std::path::Path;
use std::str::FromStr;
use std::num::NonZero;
use std::{cmp, env, iter};

use rustc_ast::expand::allocator::{ALLOC_ERROR_HANDLER, AllocatorKind, global_fn_name};
use rustc_ast::{self as ast, *};
use rustc_data_structures::fx::FxHashSet;
use rustc_data_structures::owned_slice::OwnedSlice;
use rustc_data_structures::svh::Svh;
use rustc_data_structures::sync::{self, FreezeReadGuard, FreezeWriteGuard, Lrc};
use rustc_data_structures::unord::UnordMap;
use rustc_expand::base::SyntaxExtension;

#[cfg(any(unix, windows))]
use rustc_fs_util::try_canonicalize;

use rustc_hir as hir;
use rustc_hir::def_id::{CrateNum, DefId, DefIndex, LOCAL_CRATE, LocalDefId, StableCrateId};
use rustc_hir::definitions::Definitions;
use rustc_index::IndexVec;
use rustc_middle::bug;
use rustc_middle::ty::data_structures::IndexSet;
use rustc_middle::ty::{TyCtxt, TyCtxtFeed};
use rustc_proc_macro::bridge::client::ProcMacro;
use rustc_session::Session;
use rustc_session::config::{
    CrateType, ExtendedTargetModifierInfo, ExternLocation, Externs, OptionsTargetModifiers,
    TargetModifier,
};
use rustc_session::cstore::{CrateDepKind, CrateSource, ExternCrate, ExternCrateSource};
use rustc_session::lint::{self, BuiltinLintDiag};
use rustc_session::output::validate_crate_name;
use rustc_session::search_paths::PathKind;
use rustc_span::def_id::DefId;
use rustc_span::edition::Edition;
use rustc_span::{DUMMY_SP, Ident, Span, Symbol, sym};
use rustc_target::spec::{PanicStrategy, Target};
use tracing::{debug, info, trace};

use crate::errors;
use crate::locator::{CrateError, CrateLocator, CratePaths, CrateRejections};
use crate::rmeta::{
    CrateDep, CrateMetadata, CrateNumMap, CrateRoot, MetadataBlob, TargetModifiers,
};

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

    /// Names that were used to load the crates via `extern crate` or paths.
    resolved_externs: UnordMap<Symbol, CrateNum>,

    /// Unused externs of the crate
    unused_externs: Vec<Symbol>,

    used_extern_options: FxHashSet<Symbol>,
}

impl std::fmt::Debug for CStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CStore").finish_non_exhaustive()
    }
}

pub enum LoadedMacro {
    MacroDef {
        def: MacroDef,
        ident: Ident,
        attrs: Vec<hir::Attribute>,
        span: Span,
        edition: Edition,
    },
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
            writeln!(fmt, "  priv: {:?}", data.is_private_dep())?;
            let CrateSource { dylib, rlib, rmeta, sdylib_interface } = data.source();
            if let Some(dylib) = dylib {
                writeln!(fmt, "  dylib: {}", dylib.0.display())?;
            }
            if let Some(rlib) = rlib {
                writeln!(fmt, "   rlib: {}", rlib.0.display())?;
            }
            if let Some(rmeta) = rmeta {
                writeln!(fmt, "   rmeta: {}", rmeta.0.display())?;
            }
            if let Some(sdylib_interface) = sdylib_interface {
                writeln!(fmt, "   sdylib interface: {}", sdylib_interface.0.display())?;
            }
        }
        Ok(())
    }
}

/// Reason that a crate is being sourced as a dependency.
#[derive(Clone, Copy)]
enum CrateOrigin<'a> {
    /// This crate was a dependency of another crate.
    IndirectDependency {
        /// Where this dependency was included from.
        dep_root: &'a CratePaths,
        /// True if the parent is private, meaning the dependent should also be private.
        parent_private: bool,
        /// Dependency info about this crate.
        dep: &'a CrateDep,
    },
    /// Injected by `rustc`.
    Injected,
    /// Provided by `extern crate foo` or as part of the extern prelude.
    Extern,
}

impl<'a> CrateOrigin<'a> {
    /// Return the dependency root, if any.
    fn dep_root(&self) -> Option<&'a CratePaths> {
        match self {
            CrateOrigin::IndirectDependency { dep_root, .. } => Some(dep_root),
            _ => None,
        }
    }

    /// Return dependency information, if any.
    fn dep(&self) -> Option<&'a CrateDep> {
        match self {
            CrateOrigin::IndirectDependency { dep, .. } => Some(dep),
            _ => None,
        }
    }

    /// `Some(true)` if the dependency is private or its parent is private, `Some(false)` if the
    /// dependency is not private, `None` if it could not be determined.
    fn private_dep(&self) -> Option<bool> {
        match self {
            CrateOrigin::IndirectDependency { parent_private, dep, .. } => {
                Some(dep.is_private || *parent_private)
            }
            _ => None,
        }
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
        tcx: TyCtxt<'tcx>,
        root: &CrateRoot,
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

    /// Save the name used to resolve the extern crate in the local crate
    ///
    /// The name isn't always the crate's own name, because `sess.opts.externs` can assign it another name.
    /// It's also not always the same as the `DefId`'s symbol due to renames `extern crate resolved_name as defid_name`.
    pub(crate) fn set_resolved_extern_crate_name(&mut self, name: Symbol, extern_crate: CrateNum) {
        self.resolved_externs.insert(name, extern_crate);
    }

    /// Crate resolved and loaded via the given extern name
    /// (corresponds to names in `sess.opts.externs`)
    ///
    /// May be `None` if the crate wasn't used
    pub fn resolved_extern_crate(&self, externs_name: Symbol) -> Option<CrateNum> {
        self.resolved_externs.get(&externs_name).copied()
    }

    pub(crate) fn iter_crate_data(&self) -> impl Iterator<Item = (CrateNum, &CrateMetadata)> {
        self.metas
            .iter_enumerated()
            .filter_map(|(cnum, data)| data.as_deref().map(|data| (cnum, data)))
    }

    pub fn all_proc_macro_def_ids(&self) -> impl Iterator<Item = DefId> {
        self.iter_crate_data().flat_map(|(krate, data)| data.proc_macros_for_crate(krate, self))
    }

    fn push_dependencies_in_postorder(&self, deps: &mut IndexSet<CrateNum>, cnum: CrateNum) {
        if !deps.contains(&cnum) {
            let data = self.get_crate_data(cnum);
            for dep in data.dependencies() {
                if dep != cnum {
                    self.push_dependencies_in_postorder(deps, dep);
                }
            }

            deps.insert(cnum);
        }
    }

    pub(crate) fn crate_dependencies_in_postorder(&self, cnum: CrateNum) -> IndexSet<CrateNum> {
        let mut deps = IndexSet::default();
        if cnum == LOCAL_CRATE {
            for (cnum, _) in self.iter_crate_data() {
                self.push_dependencies_in_postorder(&mut deps, cnum);
            }
        } else {
            self.push_dependencies_in_postorder(&mut deps, cnum);
        }
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
            .level;
        if level != lint::Level::Allow {
            let unused_externs =
                self.unused_externs.iter().map(|ident| ident.to_ident_string()).collect::<Vec<_>>();
            let unused_externs = unused_externs.iter().map(String::as_str).collect::<Vec<&str>>();
            tcx.dcx().emit_unused_externs(level, json_unused_externs.is_loud(), &unused_externs);
        }
    }

    fn report_target_modifiers_extended(
        tcx: TyCtxt<'_>,
        krate: &Crate,
        mods: &TargetModifiers,
        dep_mods: &TargetModifiers,
        data: &CrateMetadata,
    ) {
        let span = krate.spans.inner_span.shrink_to_lo();
        let allowed_flag_mismatches = &tcx.sess.opts.cg.unsafe_allow_abi_mismatch;
        let local_crate = tcx.crate_name(LOCAL_CRATE);
        let tmod_extender = |tmod: &TargetModifier| (tmod.extend(), tmod.clone());
        let report_diff = |prefix: &String,
                           opt_name: &String,
                           flag_local_value: Option<&String>,
                           flag_extern_value: Option<&String>| {
            if allowed_flag_mismatches.contains(&opt_name) {
                return;
            }
            let extern_crate = data.name();
            let flag_name = opt_name.clone();
            let flag_name_prefixed = format!("-{}{}", prefix, opt_name);

            match (flag_local_value, flag_extern_value) {
                (Some(local_value), Some(extern_value)) => {
                    tcx.dcx().emit_err(errors::IncompatibleTargetModifiers {
                        span,
                        extern_crate,
                        local_crate,
                        flag_name,
                        flag_name_prefixed,
                        local_value: local_value.to_string(),
                        extern_value: extern_value.to_string(),
                    })
                }
                (None, Some(extern_value)) => {
                    tcx.dcx().emit_err(errors::IncompatibleTargetModifiersLMissed {
                        span,
                        extern_crate,
                        local_crate,
                        flag_name,
                        flag_name_prefixed,
                        extern_value: extern_value.to_string(),
                    })
                }
                (Some(local_value), None) => {
                    tcx.dcx().emit_err(errors::IncompatibleTargetModifiersRMissed {
                        span,
                        extern_crate,
                        local_crate,
                        flag_name,
                        flag_name_prefixed,
                        local_value: local_value.to_string(),
                    })
                }
                (None, None) => panic!("Incorrect target modifiers report_diff(None, None)"),
            };
        };
        let mut it1 = mods.iter().map(tmod_extender);
        let mut it2 = dep_mods.iter().map(tmod_extender);
        let mut left_name_val: Option<(ExtendedTargetModifierInfo, TargetModifier)> = None;
        let mut right_name_val: Option<(ExtendedTargetModifierInfo, TargetModifier)> = None;
        loop {
            left_name_val = left_name_val.or_else(|| it1.next());
            right_name_val = right_name_val.or_else(|| it2.next());
            match (&left_name_val, &right_name_val) {
                (Some(l), Some(r)) => match l.1.opt.cmp(&r.1.opt) {
                    cmp::Ordering::Equal => {
                        if !l.1.consistent(&tcx.sess.opts, Some(&r.1)) {
                            report_diff(
                                &l.0.prefix,
                                &l.0.name,
                                Some(&l.1.value_name),
                                Some(&r.1.value_name),
                            );
                        }
                        left_name_val = None;
                        right_name_val = None;
                    }
                    cmp::Ordering::Greater => {
                        if !r.1.consistent(&tcx.sess.opts, None) {
                            report_diff(&r.0.prefix, &r.0.name, None, Some(&r.1.value_name));
                        }
                        right_name_val = None;
                    }
                    cmp::Ordering::Less => {
                        if !l.1.consistent(&tcx.sess.opts, None) {
                            report_diff(&l.0.prefix, &l.0.name, Some(&l.1.value_name), None);
                        }
                        left_name_val = None;
                    }
                },
                (Some(l), None) => {
                    if !l.1.consistent(&tcx.sess.opts, None) {
                        report_diff(&l.0.prefix, &l.0.name, Some(&l.1.value_name), None);
                    }
                    left_name_val = None;
                }
                (None, Some(r)) => {
                    if !r.1.consistent(&tcx.sess.opts, None) {
                        report_diff(&r.0.prefix, &r.0.name, None, Some(&r.1.value_name));
                    }
                    right_name_val = None;
                }
                (None, None) => break,
            }
        }
    }

    pub fn report_incompatible_target_modifiers(&self, tcx: TyCtxt<'_>, krate: &Crate) {
        for flag_name in &tcx.sess.opts.cg.unsafe_allow_abi_mismatch {
            if !OptionsTargetModifiers::is_target_modifier(flag_name) {
                tcx.dcx().emit_err(errors::UnknownTargetModifierUnsafeAllowed {
                    span: krate.spans.inner_span.shrink_to_lo(),
                    flag_name: flag_name.clone(),
                });
            }
        }
        let mods = tcx.sess.opts.gather_target_modifiers();
        for (_cnum, data) in self.iter_crate_data() {
            if data.is_proc_macro_crate() {
                continue;
            }
            let dep_mods = data.target_modifiers();
            if mods != dep_mods {
                Self::report_target_modifiers_extended(tcx, krate, &mods, &dep_mods, data);
            }
        }
    }

    // Report about async drop types in dependency if async drop feature is disabled
    pub fn report_incompatible_async_drop_feature(&self, tcx: TyCtxt<'_>, krate: &Crate) {
        if tcx.features().async_drop() {
            return;
        }
        for (_cnum, data) in self.iter_crate_data() {
            if data.is_proc_macro_crate() {
                continue;
            }
            if data.has_async_drops() {
                let extern_crate = data.name();
                let local_crate = tcx.crate_name(LOCAL_CRATE);
                tcx.dcx().emit_warn(errors::AsyncDropTypesInDependency {
                    span: krate.spans.inner_span.shrink_to_lo(),
                    extern_crate,
                    local_crate,
                });
            }
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
            resolved_externs: UnordMap::default(),
            unused_externs: Vec::new(),
            used_extern_options: Default::default(),
        }
    }
        
    /// Load WASM proc macros specified via `--wasm-proc-macro` flags
    /// Returns a vector of (macro_name, SyntaxExtension) tuples for the resolver to register
    /// This bypasses the normal metadata/CStore system entirely
    pub fn load_wasm_proc_macros(&mut self) -> Vec<(Symbol, Lrc<SyntaxExtension>, DefId)> {
        // Only compile this code when building rustc for WASM
        #[cfg(target_family = "wasm")]
        {
            use std::fs;
            use rustc_watt_runtime::WasmMacro;
            use rustc_span::def_id::DefId;

            let mut result = Vec::new();

            eprintln!("[CREADER] load_wasm_proc_macros called with {} entries",
                      self.sess.opts.wasm_proc_macros.len());

            for (file_name, path) in &self.sess.opts.wasm_proc_macros {
                eprintln!("[CREADER] Loading WASM proc macro: {} from {:?}", file_name, path);

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

                // Allocate a CrateNum for this WASM proc macro library
                // Use a synthetic stable crate ID based on the file name
                let crate_name_symbol = Symbol::intern(file_name);
                let stable_crate_id = rustc_span::def_id::StableCrateId::new(
                    crate_name_symbol,
                    false, // is_exe
                    vec![format!("wasm_proc_macro_{}", file_name)], // metadata
                    env!("CFG_VERSION"), // cfg_version
                );

                // Allocate the CrateNum
                let cnum = match self.tcx.create_crate_num(stable_crate_id) {
                    Ok(feed) => {
                        self.cstore.metas.push(None); // Reserve slot - will be filled below
                        feed.key()
                    }
                    Err(existing) => {
                        // If it already exists, use the existing cnum
                        existing
                    }
                };

                eprintln!("[CREADER] Allocated CrateNum {:?} for WASM proc macro library", cnum);

                // Create stub CrateMetadata for this WASM proc macro crate
                // This is necessary because other parts of the compiler may try to query
                // information about this crate (e.g., dependencies during lowering)
                let stub_metadata = create_wasm_proc_macro_stub_metadata(
                    self.sess,
                    self.cstore,
                    &proc_macros,
                    cnum,
                    crate_name_symbol,
                    stable_crate_id,
                    path,
                );
                self.cstore.set_crate_data(cnum, stub_metadata);

                // Convert ProcMacro to SyntaxExtension before passing to resolver
                // This avoids needing proc_macro crate dependency in rustc_resolve
                // Assign sequential DefIndex values starting from 1 (0 is crate root)
                let proc_macros_vec = proc_macros.into_vec();
                for (idx, pm) in proc_macros_vec.into_iter().enumerate() {
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

                    // Create a DefId using the allocated CrateNum
                    // Use sequential DefIndex values starting from 1 (0 is reserved for crate root)
                    let def_id = DefId {
                        krate: cnum,
                        index: rustc_span::def_id::DefIndex::from_u32((idx + 1) as u32),
                    };

                    eprintln!("[CREADER] About to intern symbol for WASM proc macro: {}", name);
                    let name_symbol = Symbol::intern(name);
                    eprintln!("[CREADER] Symbol interned successfully");

                    result.push((name_symbol, Lrc::new(ext), def_id));
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


    fn existing_match(
        &self,
        externs: &Externs,
        name: Symbol,
        hash: Option<Svh>,
        kind: PathKind,
    ) -> Option<CrateNum> {
        for (cnum, data) in self.iter_crate_data() {
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
            let source = data.source();
            if let Some(entry) = externs.get(name.as_str()) {
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

    /// Determine whether a dependency should be considered private.
    ///
    /// Dependencies are private if they get extern option specified, e.g. `--extern priv:mycrate`.
    /// This is stored in metadata, so `private_dep`  can be correctly set during load. A `Some`
    /// value for `private_dep` indicates that the crate is known to be private or public (note
    /// that any `None` or `Some(false)` use of the same crate will make it public).
    ///
    /// Sometimes the directly dependent crate is not specified by `--extern`, in this case,
    /// `private-dep` is none during loading. This is equivalent to the scenario where the
    /// command parameter is set to `public-dependency`
    fn is_private_dep(
        &self,
        externs: &Externs,
        name: Symbol,
        private_dep: Option<bool>,
        origin: CrateOrigin<'_>,
    ) -> bool {
        if matches!(origin, CrateOrigin::Injected) {
            return true;
        }

        let extern_private = externs.get(name.as_str()).map(|e| e.is_private_dep);
        match (extern_private, private_dep) {
            // Explicit non-private via `--extern`, explicit non-private from metadata, or
            // unspecified with default to public.
            (Some(false), _) | (_, Some(false)) | (None, None) => false,
            // Marked private via `--extern priv:mycrate` or in metadata.
            (Some(true) | None, Some(true) | None) => true,
        }
    }

    fn register_crate<'tcx>(
        &mut self,
        tcx: TyCtxt<'tcx>,
        host_lib: Option<Library>,
        origin: CrateOrigin<'_>,
        lib: Library,
        dep_kind: CrateDepKind,
        name: Symbol,
        private_dep: Option<bool>,
        pre_loaded_proc_macros: Option<&'static [ProcMacro]>,
    ) -> Result<CrateNum, CrateError> {
        let _prof_timer =
            tcx.sess.prof.generic_activity_with_arg("metadata_register_crate", name.as_str());

        let Library { source, metadata } = lib;
        let crate_root = metadata.get_root();
        let host_hash = host_lib.as_ref().map(|lib| lib.metadata.get_root().hash());
        let private_dep = self.is_private_dep(&tcx.sess.opts.externs, name, private_dep, origin);

        // Claim this crate number and cache it
        let feed = self.intern_stable_crate_id(tcx, &crate_root)?;
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
        let dep_root = if let Some(dep_root) = origin.dep_root() {
            dep_root
        } else {
            crate_paths = CratePaths::new(crate_root.name(), source.clone());
            &crate_paths
        };

        let cnum_map = self.resolve_crate_deps(
            tcx,
            dep_root,
            &crate_root,
            &metadata,
            cnum,
            dep_kind,
            private_dep,
        )?;

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
            Some(self.dlsym_proc_macros(tcx.sess, &dlsym_dylib.0, dlsym_root.stable_crate_id())?)
        } else {
            None
        };

        let crate_metadata = CrateMetadata::new(
            tcx.sess,
            self,
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

        self.set_crate_data(cnum, crate_metadata);

        Ok(cnum)
    }

    fn load_proc_macro<'a, 'b>(
        &self,
        sess: &'a Session,
        locator: &mut CrateLocator<'b>,
        crate_rejections: &mut CrateRejections,
        path_kind: PathKind,
        host_hash: Option<Svh>,
    ) -> Result<Option<(LoadResult, Option<Library>)>, CrateError>
    where
        'a: 'b,
    {
        if sess.opts.unstable_opts.dual_proc_macros {
            // Use a new crate locator and crate rejections so trying to load a proc macro doesn't
            // affect the error message we emit
            let mut proc_macro_locator = locator.clone();

            // Try to load a proc macro
            proc_macro_locator.for_target_proc_macro(sess, path_kind);

            // Load the proc macro crate for the target
            let target_result =
                match self.load(&mut proc_macro_locator, &mut CrateRejections::default())? {
                    Some(LoadResult::Previous(cnum)) => {
                        return Ok(Some((LoadResult::Previous(cnum), None)));
                    }
                    Some(LoadResult::Loaded(library)) => Some(LoadResult::Loaded(library)),
                    None => return Ok(None),
                };

            // Use the existing crate_rejections as we want the error message to be affected by
            // loading the host proc macro.
            *crate_rejections = CrateRejections::default();

            // Load the proc macro crate for the host
            locator.for_proc_macro(sess, path_kind);

            locator.hash = host_hash;

            let Some(host_result) = self.load(locator, crate_rejections)? else {
                return Ok(None);
            };

            let host_result = match host_result {
                LoadResult::Previous(..) => {
                    panic!("host and target proc macros must be loaded in lock-step")
                }
                LoadResult::Loaded(library) => library,
            };
            Ok(Some((target_result.unwrap(), Some(host_result))))
        } else {
            // Use a new crate locator and crate rejections so trying to load a proc macro doesn't
            // affect the error message we emit
            let mut proc_macro_locator = locator.clone();

            // Load the proc macro crate for the host
            proc_macro_locator.for_proc_macro(sess, path_kind);

            let Some(host_result) =
                self.load(&mut proc_macro_locator, &mut CrateRejections::default())?
            else {
                return Ok(None);
            };

            Ok(Some((host_result, None)))
        }
    }

    fn resolve_crate<'tcx>(
        &mut self,
        tcx: TyCtxt<'tcx>,
        name: Symbol,
        span: Span,
        dep_kind: CrateDepKind,
        origin: CrateOrigin<'_>,
    ) -> Option<CrateNum> {
        self.used_extern_options.insert(name);
        match self.maybe_resolve_crate(tcx, name, dep_kind, origin) {
            Ok(cnum) => {
                self.set_used_recursively(cnum);
                Some(cnum)
            }
            Err(err) => {
                debug!("failed to resolve crate {} {:?}", name, dep_kind);
                let missing_core = self
                    .maybe_resolve_crate(
                        tcx,
                        sym::core,
                        CrateDepKind::Explicit,
                        CrateOrigin::Extern,
                    )
                    .is_err();
                err.report(tcx.sess, span, missing_core);
                None
            }
        }
    }

    fn maybe_resolve_crate<'b, 'tcx>(
        &'b mut self,
        tcx: TyCtxt<'tcx>,
        name: Symbol,
        mut dep_kind: CrateDepKind,
        origin: CrateOrigin<'b>,
    ) -> Result<CrateNum, CrateError> {
        info!("resolving crate `{}`", name);
        if !name.as_str().is_ascii() {
            return Err(CrateError::NonAsciiName(name));
        }

        let dep_root = origin.dep_root();
        let dep = origin.dep();
        let hash = dep.map(|d| d.hash);
        let host_hash = dep.map(|d| d.host_hash).flatten();
        let extra_filename = dep.map(|d| &d.extra_filename[..]);
        let path_kind = if dep.is_some() { PathKind::Dependency } else { PathKind::Crate };
        let private_dep = origin.private_dep();

        let result = if let Some(cnum) =
            self.existing_match(&tcx.sess.opts.externs, name, hash, path_kind)
        {
            (LoadResult::Previous(cnum), None)
        } else {
            info!("falling back to a load");
            let mut locator = CrateLocator::new(
                tcx.sess,
                &*self.metadata_loader,
                name,
                // The all loop is because `--crate-type=rlib --crate-type=rlib` is
                // legal and produces both inside this type.
                tcx.crate_types().iter().all(|c| *c == CrateType::Rlib),
                hash,
                extra_filename,
                path_kind,
            );
            let mut crate_rejections = CrateRejections::default();

            match self.load(&mut locator, &mut crate_rejections)? {
                Some(res) => (res, None),
                None => {
                    info!("falling back to loading proc_macro");
                    dep_kind = CrateDepKind::MacrosOnly;
                    match self.load_proc_macro(
                        tcx.sess,
                        &mut locator,
                        &mut crate_rejections,
                        path_kind,
                        host_hash,
                    )? {
                        Some(res) => res,
                        None => return Err(locator.into_error(crate_rejections, dep_root.cloned())),
                    }
                }
            }
        };

        match result {
            (LoadResult::Previous(cnum), None) => {
                info!("library for `{}` was loaded previously, cnum {cnum}", name);
                // When `private_dep` is none, it indicates the directly dependent crate. If it is
                // not specified by `--extern` on command line parameters, it may be
                // `private-dependency` when `register_crate` is called for the first time. Then it must be updated to
                // `public-dependency` here.
                let private_dep =
                    self.is_private_dep(&tcx.sess.opts.externs, name, private_dep, origin);
                let data = self.get_crate_data_mut(cnum);
                if data.is_proc_macro_crate() {
                    dep_kind = CrateDepKind::MacrosOnly;
                }
                data.set_dep_kind(cmp::max(data.dep_kind(), dep_kind));
                data.update_and_private_dep(private_dep);
                Ok(cnum)
            }
            (LoadResult::Loaded(library), host_library) => {
                info!("register newly loaded library for `{}`", name);
                self.register_crate(tcx, host_library, origin, library, dep_kind, name, private_dep, None)
            }
            _ => panic!(),
        }
    }

    fn load(
        &self,
        locator: &CrateLocator<'_>,
        crate_rejections: &mut CrateRejections,
    ) -> Result<Option<LoadResult>, CrateError> {
        let Some(library) = locator.maybe_load_library_crate(crate_rejections)? else {
            return Ok(None);
        };

        // In the case that we're loading a crate, but not matching
        // against a hash, we could load a crate which has the same hash
        // as an already loaded crate. If this is the case prevent
        // duplicates by just using the first crate.
        let root = library.metadata.get_root();
        let mut result = LoadResult::Loaded(library);
        for (cnum, data) in self.iter_crate_data() {
            if data.name() == root.name() && root.hash() == data.hash() {
                assert!(locator.hash.is_none());
                info!("load success, going to previous cnum: {}", cnum);
                result = LoadResult::Previous(cnum);
                break;
            }
        }
        Ok(Some(result))
    }

    /// Go through the crate metadata and load any crates that it references.
    fn resolve_crate_deps(
        &mut self,
        tcx: TyCtxt<'_>,
        dep_root: &CratePaths,
        crate_root: &CrateRoot,
        metadata: &MetadataBlob,
        krate: CrateNum,
        dep_kind: CrateDepKind,
        parent_is_private: bool,
    ) -> Result<CrateNumMap, CrateError> {
        debug!(
            "resolving deps of external crate `{}` with dep root `{}`",
            crate_root.name(),
            dep_root.name
        );
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
                "resolving dep `{}`->`{}` hash: `{}` extra filename: `{}` private {}",
                crate_root.name(),
                dep.name,
                dep.hash,
                dep.extra_filename,
                dep.is_private,
            );
            let dep_kind = match dep_kind {
                CrateDepKind::MacrosOnly => CrateDepKind::MacrosOnly,
                _ => dep.kind,
            };
            let cnum = self.maybe_resolve_crate(
                tcx,
                dep.name,
                dep_kind,
                CrateOrigin::IndirectDependency {
                    dep_root,
                    parent_private: parent_is_private,
                    dep: &dep,
                },
            )?;
            crate_num_map.push(cnum);
        }

        debug!("resolve_crate_deps: cnum_map for {:?} is {:?}", krate, crate_num_map);
        Ok(crate_num_map)
    }

    fn dlsym_proc_macros(
        &self,
        sess: &Session,
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
        let sym_name = sess.generate_proc_macro_decls_symbol(stable_crate_id);
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

    fn inject_panic_runtime(&mut self, tcx: TyCtxt<'_>, krate: &ast::Crate) {
        // If we're only compiling an rlib, then there's no need to select a
        // panic runtime, so we just skip this section entirely.
        let only_rlib = tcx.crate_types().iter().all(|ct| *ct == CrateType::Rlib);
        if only_rlib {
            info!("panic runtime injection skipped, only generating rlib");
            return;
        }

        // If we need a panic runtime, we try to find an existing one here. At
        // the same time we perform some general validation of the DAG we've got
        // going such as ensuring everything has a compatible panic strategy.
        let mut needs_panic_runtime = attr::contains_name(&krate.attrs, sym::needs_panic_runtime);
        for (_cnum, data) in self.iter_crate_data() {
            needs_panic_runtime |= data.needs_panic_runtime();
        }

        // If we just don't need a panic runtime at all, then we're done here
        // and there's nothing else to do.
        if !needs_panic_runtime {
            return;
        }

        // By this point we know that we need a panic runtime. Here we just load
        // an appropriate default runtime for our panic strategy.
        //
        // We may resolve to an already loaded crate (as the crate may not have
        // been explicitly linked prior to this), but this is fine.
        //
        // Also note that we have yet to perform validation of the crate graph
        // in terms of everyone has a compatible panic runtime format, that's
        // performed later as part of the `dependency_format` module.
        let desired_strategy = tcx.sess.panic_strategy();
        let name = match desired_strategy {
            PanicStrategy::Unwind => sym::panic_unwind,
            PanicStrategy::Abort => sym::panic_abort,
            PanicStrategy::ImmediateAbort => {
                // Immediate-aborting panics don't use a runtime.
                return;
            }
        };
        info!("panic runtime not found -- loading {}", name);

        let Some(cnum) =
            self.resolve_crate(tcx, name, DUMMY_SP, CrateDepKind::Implicit, CrateOrigin::Injected)
        else {
            return;
        };
        let data = self.get_crate_data(cnum);

        // Sanity check the loaded crate to ensure it is indeed a panic runtime
        // and the panic strategy is indeed what we thought it was.
        if !data.is_panic_runtime() {
            tcx.dcx().emit_err(errors::CrateNotPanicRuntime { crate_name: name });
        }
        if data.required_panic_strategy() != Some(desired_strategy) {
            tcx.dcx()
                .emit_err(errors::NoPanicStrategy { crate_name: name, strategy: desired_strategy });
        }

        self.injected_panic_runtime = Some(cnum);
    }

    fn inject_profiler_runtime(&mut self, tcx: TyCtxt<'_>) {
        let needs_profiler_runtime =
            tcx.sess.instrument_coverage() || tcx.sess.opts.cg.profile_generate.enabled();
        if !needs_profiler_runtime || tcx.sess.opts.unstable_opts.no_profiler_runtime {
            return;
        }

        info!("loading profiler");

        let name = Symbol::intern(&tcx.sess.opts.unstable_opts.profiler_runtime);
        let Some(cnum) =
            self.resolve_crate(tcx, name, DUMMY_SP, CrateDepKind::Implicit, CrateOrigin::Injected)
        else {
            return;
        };
        let data = self.get_crate_data(cnum);

        // Sanity check the loaded crate to ensure it is indeed a profiler runtime
        if !data.is_profiler_runtime() {
            tcx.dcx().emit_err(errors::NotProfilerRuntime { crate_name: name });
        }
    }

    fn inject_allocator_crate(&mut self, tcx: TyCtxt<'_>, krate: &ast::Crate) {
        self.has_global_allocator =
            match &*fn_spans(krate, Symbol::intern(&global_fn_name(sym::alloc))) {
                [span1, span2, ..] => {
                    tcx.dcx()
                        .emit_err(errors::NoMultipleGlobalAlloc { span2: *span2, span1: *span1 });
                    true
                }
                spans => !spans.is_empty(),
            };
        let alloc_error_handler = Symbol::intern(&global_fn_name(ALLOC_ERROR_HANDLER));
        self.has_alloc_error_handler = match &*fn_spans(krate, alloc_error_handler) {
            [span1, span2, ..] => {
                tcx.dcx()
                    .emit_err(errors::NoMultipleAllocErrorHandler { span2: *span2, span1: *span1 });
                true
            }
            spans => !spans.is_empty(),
        };

        // Check to see if we actually need an allocator. This desire comes
        // about through the `#![needs_allocator]` attribute and is typically
        // written down in liballoc.
        if !attr::contains_name(&krate.attrs, sym::needs_allocator)
            && !self.iter_crate_data().any(|(_, data)| data.needs_allocator())
        {
            return;
        }

        // At this point we've determined that we need an allocator. Let's see
        // if our compilation session actually needs an allocator based on what
        // we're emitting.
        let all_rlib = tcx.crate_types().iter().all(|ct| matches!(*ct, CrateType::Rlib));
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
        #[allow(rustc::symbol_intern_string_literal)]
        let this_crate = Symbol::intern("this crate");

        let mut global_allocator = self.has_global_allocator.then_some(this_crate);
        for (_, data) in self.iter_crate_data() {
            if data.has_global_allocator() {
                match global_allocator {
                    Some(other_crate) => {
                        tcx.dcx().emit_err(errors::ConflictingGlobalAlloc {
                            crate_name: data.name(),
                            other_crate_name: other_crate,
                        });
                    }
                    None => global_allocator = Some(data.name()),
                }
            }
        }
        let mut alloc_error_handler = self.has_alloc_error_handler.then_some(this_crate);
        for (_, data) in self.iter_crate_data() {
            if data.has_alloc_error_handler() {
                match alloc_error_handler {
                    Some(other_crate) => {
                        tcx.dcx().emit_err(errors::ConflictingAllocErrorHandler {
                            crate_name: data.name(),
                            other_crate_name: other_crate,
                        });
                    }
                    None => alloc_error_handler = Some(data.name()),
                }
            }
        }

        if global_allocator.is_some() {
            self.allocator_kind = Some(AllocatorKind::Global);
        } else {
            // Ok we haven't found a global allocator but we still need an
            // allocator. At this point our allocator request is typically fulfilled
            // by the standard library, denoted by the `#![default_lib_allocator]`
            // attribute.
            if !attr::contains_name(&krate.attrs, sym::default_lib_allocator)
                && !self.iter_crate_data().any(|(_, data)| data.has_default_lib_allocator())
            {
                tcx.dcx().emit_err(errors::GlobalAllocRequired);
            }
            self.allocator_kind = Some(AllocatorKind::Default);
        }

        if alloc_error_handler.is_some() {
            self.alloc_error_handler_kind = Some(AllocatorKind::Global);
        } else {
            // The alloc crate provides a default allocation error handler if
            // one isn't specified.
            self.alloc_error_handler_kind = Some(AllocatorKind::Default);
        }
    }

    fn inject_forced_externs(&mut self, tcx: TyCtxt<'_>) {
        for (name, entry) in tcx.sess.opts.externs.iter() {
            if entry.force {
                let name_interned = Symbol::intern(name);
                if !self.used_extern_options.contains(&name_interned) {
                    self.resolve_crate(
                        tcx,
                        name_interned,
                        DUMMY_SP,
                        CrateDepKind::Explicit,
                        CrateOrigin::Extern,
                    );
                }
            }
        }
    }

    /// Inject the `compiler_builtins` crate if it is not already in the graph.
    fn inject_compiler_builtins(&mut self, tcx: TyCtxt<'_>, krate: &ast::Crate) {
        // `compiler_builtins` does not get extern builtins, nor do `#![no_core]` crates
        if attr::contains_name(&krate.attrs, sym::compiler_builtins)
            || attr::contains_name(&krate.attrs, sym::no_core)
        {
            info!("`compiler_builtins` unneeded");
            return;
        }

        // If a `#![compiler_builtins]` crate already exists, avoid injecting it twice. This is
        // the common case since usually it appears as a dependency of `std` or `alloc`.
        for (cnum, cmeta) in self.iter_crate_data() {
            if cmeta.is_compiler_builtins() {
                info!("`compiler_builtins` already exists (cnum = {cnum}); skipping injection");
                return;
            }
        }

        // `compiler_builtins` is not yet in the graph; inject it. Error on resolution failure.
        let Some(cnum) = self.resolve_crate(
            tcx,
            sym::compiler_builtins,
            krate.spans.inner_span.shrink_to_lo(),
            CrateDepKind::Explicit,
            CrateOrigin::Injected,
        ) else {
            info!("`compiler_builtins` not resolved");
            return;
        };

        // Sanity check that the loaded crate is `#![compiler_builtins]`
        let cmeta = self.get_crate_data(cnum);
        if !cmeta.is_compiler_builtins() {
            tcx.dcx().emit_err(errors::CrateNotCompilerBuiltins { crate_name: cmeta.name() });
        }
    }

    fn report_unused_deps_in_crate(&mut self, tcx: TyCtxt<'_>, krate: &ast::Crate) {
        // Make a point span rather than covering the whole file
        let span = krate.spans.inner_span.shrink_to_lo();
        // Complain about anything left over
        for (name, entry) in tcx.sess.opts.externs.iter() {
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
            if tcx.sess.opts.json_unused_externs.is_enabled() {
                self.unused_externs.push(name_interned);
                continue;
            }

            tcx.sess.psess.buffer_lint(
                lint::builtin::UNUSED_CRATE_DEPENDENCIES,
                span,
                ast::CRATE_NODE_ID,
                BuiltinLintDiag::UnusedCrateDependency {
                    extern_crate: name_interned,
                    local_crate: tcx.crate_name(LOCAL_CRATE),
                },
            );
        }
    }

    fn report_future_incompatible_deps(&self, tcx: TyCtxt<'_>, krate: &ast::Crate) {
        let name = tcx.crate_name(LOCAL_CRATE);

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

            tcx.sess.dcx().emit_err(errors::WasmCAbi { span });
        }
    }

    pub fn postprocess(&mut self, tcx: TyCtxt<'_>, krate: &ast::Crate) {
        self.inject_compiler_builtins(tcx, krate);
        self.inject_forced_externs(tcx);
        self.inject_profiler_runtime(tcx);
        self.inject_allocator_crate(tcx, krate);
        self.inject_panic_runtime(tcx, krate);

        self.report_unused_deps_in_crate(tcx, krate);
        self.report_future_incompatible_deps(tcx, krate);

        info!("{:?}", CrateDump(self));
    }

    /// Process an `extern crate foo` AST node.
    pub fn process_extern_crate(
        &mut self,
        tcx: TyCtxt<'_>,
        item: &ast::Item,
        def_id: LocalDefId,
        definitions: &Definitions,
    ) -> Option<CrateNum> {
        match item.kind {
            ast::ItemKind::ExternCrate(orig_name, ident) => {
                debug!("resolving extern crate stmt. ident: {} orig_name: {:?}", ident, orig_name);
                let name = match orig_name {
                    Some(orig_name) => {
                        validate_crate_name(tcx.sess, orig_name, Some(item.span));
                        orig_name
                    }
                    None => ident.name,
                };
                let dep_kind = if attr::contains_name(&item.attrs, sym::no_link) {
                    CrateDepKind::MacrosOnly
                } else {
                    CrateDepKind::Explicit
                };

                let cnum =
                    self.resolve_crate(tcx, name, item.span, dep_kind, CrateOrigin::Extern)?;

                let path_len = definitions.def_path(def_id).data.len();
                self.update_extern_crate(
                    cnum,
                    name,
                    ExternCrate {
                        src: ExternCrateSource::Extern(def_id.to_def_id()),
                        span: item.span,
                        path_len,
                        dependency_of: LOCAL_CRATE,
                    },
                );
                Some(cnum)
            }
            _ => bug!(),
        }
    }

    pub fn process_path_extern(
        &mut self,
        tcx: TyCtxt<'_>,
        name: Symbol,
        span: Span,
    ) -> Option<CrateNum> {
        let cnum =
            self.resolve_crate(tcx, name, span, CrateDepKind::Explicit, CrateOrigin::Extern)?;

        self.update_extern_crate(
            cnum,
            name,
            ExternCrate {
                src: ExternCrateSource::Path,
                span,
                // to have the least priority in `update_extern_crate`
                path_len: usize::MAX,
                dependency_of: LOCAL_CRATE,
            },
        );

        Some(cnum)
    }

    pub fn maybe_process_path_extern(&mut self, tcx: TyCtxt<'_>, name: Symbol) -> Option<CrateNum> {
        self.maybe_resolve_crate(tcx, name, CrateDepKind::Explicit, CrateOrigin::Extern).ok()
    }
}

/// Creates a minimal stub CrateMetadata for WASM proc macro crates
///
/// WASM proc macros don't have real .rmeta files, so we need to create
/// synthetic metadata so that the compiler can handle queries about them.
#[cfg(target_family = "wasm")]
fn create_wasm_proc_macro_stub_metadata(
    sess: &rustc_session::Session,
    _cstore: &CStore,
    proc_macros: &[ProcMacro],
    cnum: CrateNum,
    crate_name: Symbol,
    stable_crate_id: StableCrateId,
    wasm_path: &std::path::Path,
) -> CrateMetadata {
    use rustc_data_structures::owned_slice::slice_owned;

    // Create a stub CrateRoot with all empty/default fields using the helper
    let stub_root = CrateRoot::new_wasm_proc_macro_stub(
        TargetTuple::from_tuple(&sess.opts.target_triple.tuple()),
        crate_name,
        stable_crate_id,
    );

    // Create a minimal empty blob without full encoding
    // For WASM proc macros, we don't actually need most of the metadata
    // since queries won't be made against these crates - we only use raw_proc_macros

    // Create minimal placeholder bytes (make it large enough for any decoder attempts)
    // Must end with the magic bytes that MemDecoder expects
    const MAGIC_END_BYTES: &[u8] = b"rust-end-file";
    let mut dummy_bytes = vec![0u8; 4096 - MAGIC_END_BYTES.len()];
    dummy_bytes.extend_from_slice(MAGIC_END_BYTES);
    let owned_slice = slice_owned(dummy_bytes, std::ops::Deref::deref);

    // Create MetadataBlob without validation since we won't decode from it
    let blob = MetadataBlob::new_unvalidated(owned_slice);

    // Use the stub_root directly
    let root = stub_root;

    let _macro_def_indices: Vec<DefIndex> = (0..proc_macros.len())
        .map(|i| DefIndex::from_u32((i + 1) as u32))
        .collect();
    // Create minimal CrateSource for the WASM file
    let source = CrateSource {
        dylib: Some((wasm_path.to_path_buf(), PathKind::All)),
        rlib: None,
        rmeta: None,
    };

    // Create CrateMetadata with the stub data using the specialized constructor
    CrateMetadata::new_wasm_proc_macro_stub(
        blob,
        root,
        Some(Box::leak(proc_macros.to_vec().into_boxed_slice())),
        cnum,
        CrateNumMap::new(),
        CrateDepKind::MacrosOnly,
        source,
    )
}

fn fn_spans(krate: &ast::Crate, name: Symbol) -> Vec<Span> {
    struct Finder {
        name: Symbol,
        spans: Vec<Span>,
    }
    impl<'ast> visit::Visitor<'ast> for Finder {
        fn visit_item(&mut self, item: &'ast ast::Item) {
            if let Some(ident) = item.kind.ident()
                && ident.name == self.name
                && attr::contains_name(&item.attrs, sym::rustc_std_internal_symbol)
            {
                self.spans.push(item.span);
            }
            visit::walk_item(self, item)
        }
    }

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
        let mut archive_member = std::ffi::OsString::from("a(");
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
        eprintln!("[WASM SLOT] slot_0_derive called!");
        let slots = get_slots().lock().unwrap();
        let data = slots[0].as_ref().expect("Slot 0 not initialized");
        eprintln!("[WASM SLOT] About to call proc_macro_derive for function: {}", data.function_name);
        let result = data.wasm_macro.proc_macro_derive(data.function_name, input);
        eprintln!("[WASM SLOT] proc_macro_derive returned successfully");
        result
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
        eprintln!("[WASM SLOT] slot_1_derive called!");
        let slots = get_slots().lock().unwrap();
        let data = slots[1].as_ref().expect("Slot 1 not initialized");
        eprintln!("[WASM SLOT] About to call proc_macro_derive for {}", data.function_name);
        let result = data.wasm_macro.proc_macro_derive(data.function_name, input);
        eprintln!("[WASM SLOT] proc_macro_derive returned successfully");
        result
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
