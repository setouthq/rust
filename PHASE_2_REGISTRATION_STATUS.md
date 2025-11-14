# Phase 2: Proc Macro Registration - Status Update

**Date:** November 10, 2025

## TL;DR

After extensive research and investigation, implementing proc macro registration is more complex than initially estimated. The core issue is that rustc's architecture assumes all crates have `.rmeta` metadata files, which WASM proc macros do not have.

## What Was Attempted

### Approach 1: Use --extern with WASM files
**Attempted:** Testing if `--extern Demo=path.wasm` works with existing WASM support in `dlsym_proc_macros`

**Result:** Failed with `unreachable` trap. The WASM support in `dlsym_proc_macros` only works AFTER metadata has been loaded, but the `.wasm` file has no `.rmeta` file.

### Approach 2: Research synthetic metadata creation
**Investigated:** How to create minimal `CrateMetadata` without a `.rmeta` file

**Findings:** Requires creating:
- `MetadataBlob`: Binary-encoded metadata in rustc's custom format
- `CrateRoot`: Complex structure with many LazyArray fields that require proper metadata encoding
- `DefPathHashMap`: Maps definition path hashes to indices
- `StableCrateId`: Unique crate identifier that must be registered with TyCtxt
- Plus 15+ other interconnected fields

**Complexity:** This is not a quick implementation. It requires understanding rustc's metadata encoding format and creating valid (even if minimal) metadata structures.

## Key Insights from Research

### How Proc Macros Are Loaded (Normal Path)

1. User specifies `--extern crate_name=path.rlib`
2. Crate loader uses `CrateLocator` to find the `.rlib` file
3. Metadata is extracted from the `.rmeta` section
4. `CrateMetadata::new()` is called with the decoded metadata
5. For proc macro crates, `dlsym_proc_macros()` is called to load the actual functions
6. The `CrateMetadata` (including `raw_proc_macros` field) is registered in `CStore`
7. Resolver queries `CStore` to find proc macros by name
8. When found, `load_proc_macro()` converts the `ProcMacro` to a `SyntaxExtension`

### Where WASM Proc Macros Break

- **Steps 1-4 fail:** No `.rmeta` file exists for WASM proc macros
- **Step 5 works:** `dlsym_proc_macros` has WASM support (lines 761-766 in creader.rs)
- **Step 6 blocked:** Can't create `CrateMetadata` without metadata
- **Steps 7-8 never reached:** Resolver never finds the proc macros

### What --wasm-proc-macro Currently Does

✅ **Working:**
- Parses command-line flag
- Reads WASM file (503KB loaded)
- Extracts proc macro metadata from `.rustc_proc_macro_decls` section
- Obtains `&'static [ProcMacro]` array

❌ **Not Working:**
- Registering proc macros in `CStore`
- Making proc macros discoverable by resolver

## Implementation Options Analysis

### Option A: Full Synthetic Metadata ⭐ (Recommended for Production)

**Approach:** Create complete but minimal metadata structures programmatically

**Required Implementation:**
```rust
// In rustc_metadata/src/rmeta/mod.rs or new file

/// Create minimal metadata blob for synthetic proc macro crate
fn create_synthetic_metadata_blob(name: Symbol) -> MetadataBlob {
    // Use MetadataBuilder or manually construct binary format
    // Must be valid enough to pass MemDecoder validation
    // Include:
    // - METADATA_HEADER
    // - Compressed length
    // - CrateRoot position
    // - Minimal encoded data
}

/// Create minimal CrateRoot for synthetic proc macro crate
fn create_synthetic_crate_root(
    name: Symbol,
    stable_crate_id: StableCrateId,
    proc_macro_count: usize,
) -> CrateRoot {
    CrateRoot {
        header: CrateHeader {
            triple: /* target triple */,
            hash: Svh::new(&[0; 20]), // Synthetic hash
            name,
            is_proc_macro_crate: true,
        },
        extra_filename: String::new(),
        stable_crate_id,
        required_panic_strategy: None,
        panic_in_drop_strategy: PanicStrategy::Unwind,
        edition: Edition::Edition2021,
        has_global_allocator: false,
        has_alloc_error_handler: false,
        has_panic_handler: false,
        has_default_lib_allocator: false,
        // All lazy arrays need valid (even if empty) encoding
        crate_deps: LazyArray::empty(),
        dylib_dependency_formats: LazyArray::empty(),
        lib_features: LazyArray::empty(),
        // ... many more fields ...
        // proc_macro_data needs DefIndex entries matching proc_macro count
        proc_macro_data: /* LazyArray with proc_macro_count entries */,
    }
}

// In rustc_metadata/src/rmeta/decoder.rs
impl CrateMetadata {
    pub fn new_synthetic_proc_macro(
        sess: &Session,
        cstore: &CStore,
        tcx: TyCtxt<'_>,
        name: Symbol,
        proc_macros: &'static [ProcMacro],
        wasm_path: &Path,
    ) -> Result<(CrateNum, CrateMetadata), CrateError> {
        let stable_crate_id = StableCrateId::new(
            name,
            /* is_local */ false,
            sess.local_stable_crate_id(),
            /* hash parts... */,
        );

        // Allocate CrateNum through TyCtxt
        let cnum = /* allocate via tcx.create_crate_num() */;

        let blob = create_synthetic_metadata_blob(name);
        let root = create_synthetic_crate_root(name, stable_crate_id, proc_macros.len());

        let source = CrateSource {
            dylib: None,
            rlib: None,
            rmeta: Some((wasm_path.to_path_buf(), PathKind::All)),
        };

        let metadata = CrateMetadata::new(
            sess,
            cstore,
            blob,
            root,
            Some(proc_macros),
            cnum,
            CrateNumMap::new(), // No dependencies
            CrateDepKind::MacrosOnly,
            source,
            false, // not private
            None, // no host_hash
        );

        Ok((cnum, metadata))
    }
}
```

**Pros:**
- Most compatible with rustc architecture
- Handles all edge cases properly
- Clean separation of concerns
- Production-ready when complete

**Cons:**
- Complex to implement (estimated 1-2 days)
- Requires deep understanding of metadata format
- Must create valid encoded LazyArray structures
- Need to handle DefIndex allocation for proc macros

**Estimated Time:** 1-2 days of careful implementation

### Option B: Bypass CrateMetadata (Simpler)

**Approach:** Add separate registry for WASM proc macros in resolver

**Required Implementation:**
```rust
// In rustc_resolve/src/lib.rs
pub struct Resolver<'ra, 'tcx> {
    // ... existing fields ...

    /// WASM proc macros loaded via --wasm-proc-macro flag
    /// Maps macro name to (proc_macros, edition)
    wasm_proc_macros: FxHashMap<Symbol, (&'static [ProcMacro], Edition)>,
}

impl<'ra, 'tcx> Resolver<'ra, 'tcx> {
    pub fn register_wasm_proc_macro(
        &mut self,
        name: Symbol,
        proc_macros: &'static [ProcMacro],
        edition: Edition,
    ) {
        self.wasm_proc_macros.insert(name, (proc_macros, edition));
    }
}

// In rustc_resolve/src/macros.rs
impl<'ra, 'tcx> Resolver<'ra, 'tcx> {
    fn resolve_macro_or_delegation_path(
        &mut self,
        // ... params ...
    ) -> Result<(Option<Lrc<SyntaxExtension>>, Res), Determinacy> {
        // ... existing code ...

        // BEFORE normal resolution, check WASM proc macros
        if path.len() == 1 && kind == Some(MacroKind::Derive) {
            let name = path[0].ident.name;
            if let Some((proc_macros, edition)) = self.wasm_proc_macros.get(&name) {
                // Found a WASM proc macro!
                // Create SyntaxExtension directly from ProcMacro
                for pm in proc_macros.iter() {
                    if let ProcMacro::CustomDerive { trait_name, .. } = pm {
                        if Symbol::intern(trait_name) == name {
                            let ext = self.create_syntax_extension_from_proc_macro(
                                pm,
                                *edition,
                                path_span,
                            );
                            return Ok((Some(Lrc::new(ext)), Res::Def(DefKind::Macro(MacroKind::Derive), /* DefId */)));
                        }
                    }
                }
            }
        }

        // Fall through to normal resolution
        // ... existing code ...
    }

    fn create_syntax_extension_from_proc_macro(
        &self,
        proc_macro: &ProcMacro,
        edition: Edition,
        span: Span,
    ) -> SyntaxExtension {
        // Convert ProcMacro to SyntaxExtension
        // Similar to CrateMetadataRef::load_proc_macro() but without metadata
        match proc_macro {
            ProcMacro::CustomDerive { trait_name, attributes, client } => {
                let helper_attrs = attributes.iter()
                    .map(|s| Symbol::intern(s))
                    .collect();
                let kind = SyntaxExtensionKind::Derive(Box::new(DeriveProcMacro {
                    client: *client,
                }));
                SyntaxExtension::new(
                    self.tcx.sess,
                    self.tcx.features(),
                    kind,
                    span,
                    helper_attrs,
                    edition,
                    Symbol::intern(trait_name),
                    &[], // no attributes
                    false, // not builtin
                )
            }
            // Similar for Attr and Bang...
        }
    }
}

// In rustc_metadata/src/creader.rs
impl CrateLoader<'_, '_> {
    pub fn load_wasm_proc_macros(&mut self) {
        #[cfg(target_family = "wasm")]
        {
            // ... existing loading code ...

            // NEW: Pass to resolver instead of trying to register in cstore
            for (name, proc_macros) in loaded_wasm_macros {
                self.tcx.resolver_for_lowering(()).borrow_mut()
                    .register_wasm_proc_macro(
                        Symbol::intern(name),
                        Box::leak(proc_macros),
                        Edition::Edition2021, // Could parse from WASM or make configurable
                    );
            }
        }
    }
}
```

**Pros:**
- Much simpler than Option A
- Bypasses complex metadata machinery
- Direct path from flag to usage
- Can be implemented in 4-6 hours

**Cons:**
- Not consistent with how other proc macros work
- Requires modifying Resolver struct
- May miss edge cases that normal path handles
- Resolver/metadata separation becomes less clean
- Need to handle DefId generation for Res::Def

**Estimated Time:** 4-6 hours

### Option C: Use Normal --extern Path with Synthetic .rmeta

**Approach:** Generate a `.rmeta` file for each WASM proc macro and use normal loading

**Required Implementation:**
```rust
// When --wasm-proc-macro is specified, generate a companion .rmeta file

// In rustc_metadata/src/creader.rs
impl CrateLoader<'_, '_> {
    pub fn load_wasm_proc_macros(&mut self) {
        #[cfg(target_family = "wasm")]
        {
            for (name, path) in &self.sess.opts.wasm_proc_macros {
                // 1. Load WASM and extract proc macros (already working)
                let wasm_bytes = fs::read(path)?;
                let wasm_macro = WasmMacro::new_owned(wasm_bytes);
                let proc_macros = create_wasm_proc_macros(wasm_macro);

                // 2. Generate synthetic .rmeta file
                let rmeta_path = path.with_extension("rmeta");
                generate_synthetic_rmeta_file(
                    &rmeta_path,
                    name,
                    &proc_macros,
                )?;

                // 3. Add to externs so normal loading finds it
                let name_symbol = Symbol::intern(name);
                self.sess.opts.externs.insert(
                    name_symbol.to_string(),
                    ExternEntry {
                        location: ExternLocation::ExactPaths(/* rmeta_path */),
                        is_private_dep: false,
                        add_prelude: true,
                        nounused_dep: false,
                        force: false,
                    },
                );

                // 4. Let normal crate loading handle it
                // The .rmeta will be found and loaded normally
                // dlsym_proc_macros will load the .wasm file
            }
        }
    }
}

fn generate_synthetic_rmeta_file(
    path: &Path,
    name: &str,
    proc_macros: &[ProcMacro],
) -> Result<(), std::io::Error> {
    use rustc_metadata::rmeta::encoder::encode_metadata;

    // Create minimal metadata using encoder
    // This is still complex but at least uses existing infrastructure
    let encoded_metadata = /* create minimal EncodedMetadata */;

    fs::write(path, encoded_metadata.raw_data())?;
    Ok(())
}
```

**Pros:**
- Uses normal crate loading path
- Existing infrastructure handles most complexity
- `.rmeta` can be cached and reused

**Cons:**
- Still requires understanding metadata encoding
- File I/O overhead
- Where to store generated `.rmeta` files?
- Still complex to generate valid metadata

**Estimated Time:** 1 day

## Recommended Path Forward

### Short Term (Proof of Concept): **Option B**

For getting something working quickly to validate the approach:

1. Implement Option B (bypass CrateMetadata)
2. It's the fastest path to a working prototype
3. Proves that WASM proc macros CAN work in rustc.wasm
4. Estimated time: 4-6 hours

### Long Term (Production): **Option A**

For a proper, maintainable implementation:

1. Implement Option A (full synthetic metadata)
2. More consistent with rustc architecture
3. Handles edge cases properly
4. Better for upstreaming to rustc project
5. Estimated time: 1-2 days

## Alternative: Focus on Solution A from Original Plan

Recall that the original `PROC_MACRO_LOADING_ISSUE.md` had three solutions:

- **Solution A:** Generate `.rlib` during proc macro compilation (cleanest)
- **Solution B:** Automatic metadata generation in linker
- **Solution C:** `--wasm-proc-macro` flag (current approach)

Given the complexity of Solution C's Phase 2, it may be worth reconsidering **Solution A**: modifying the compiler to automatically generate `.rlib` files when compiling proc macros to WASM. This would make WASM proc macros work exactly like native ones, with no special handling needed.

## Files That Would Need Modification

### For Option B (Bypass):
1. `compiler/rustc_resolve/src/lib.rs` - Add wasm_proc_macros field
2. `compiler/rustc_resolve/src/macros.rs` - Check wasm_proc_macros in resolution
3. `compiler/rustc_metadata/src/creader.rs` - Pass macros to resolver

### For Option A (Synthetic):
1. `compiler/rustc_metadata/src/rmeta/mod.rs` - Add synthetic metadata creation
2. `compiler/rustc_metadata/src/rmeta/decoder.rs` - Add CrateMetadata::new_synthetic_proc_macro()
3. `compiler/rustc_metadata/src/creader.rs` - Call synthetic constructor

## Technical Blockers

### Blocker 1: MetadataBlob Creation
**Issue:** MetadataBlob expects binary-encoded metadata that passes MemDecoder validation

**Possible Solutions:**
- Use existing `encoder::EncodeContext` to create minimal but valid metadata
- Create a special "synthetic" variant that bypasses normal decoding
- Manually craft the binary format (error-prone)

### Blocker 2: CrateRoot Complexity
**Issue:** CrateRoot has 30+ fields, many are LazyArray that need proper encoding

**Possible Solutions:**
- Use encoder to create valid LazyArray::empty() for unused fields
- Create wrapper that returns empty/default for all queries
- Modify CrateRoot to have a "synthetic" mode that uses defaults

### Blocker 3: DefIndex Allocation
**Issue:** Proc macros need DefIndex entries in proc_macro_data

**Possible Solutions:**
- Allocate synthetic DefIndex values
- Create proc_macro_data LazyArray with correct count
- Ensure DefIndex → ProcMacro mapping works

### Blocker 4: StableCrateId Registration
**Issue:** Must register unique StableCrateId with TyCtxt without conflicts

**Possible Solutions:**
- Generate deterministic StableCrateId from WASM file hash
- Check for conflicts before registration
- Use special namespace for synthetic crates

## Next Steps

### If Continuing with Option C:

1. **Decision Point:** Choose Option A or Option B
2. **Option B (Quick):**
   - Add wasm_proc_macros field to Resolver
   - Modify macro resolution to check it
   - Test with Demo macro
3. **Option A (Proper):**
   - Study metadata encoder deeply
   - Create minimal metadata generation functions
   - Test incrementally

### If Reconsidering Approach:

1. Re-evaluate **Solution A** from original plan
2. Investigate modifying proc-macro compilation to generate `.rlib`
3. This might actually be simpler than synthetic metadata

## Conclusion

Phase 1 (flag infrastructure) is ✅ **complete and working**.

Phase 2 (registration) is more complex than initially estimated. The core challenge is that rustc's architecture assumes all crates have `.rmeta` metadata files. Creating synthetic metadata is a significant undertaking.

**Recommendation:** Implement Option B for proof of concept, then decide whether to:
- Polish Option B for use
- Invest in Option A for proper implementation
- Pivot to Solution A (automatic `.rlib` generation)

The `--wasm-proc-macro` flag infrastructure is solid and the WASM proc macros are successfully loaded. We're ~70% of the way there - just need to bridge the gap between loaded proc macros and the resolver.

---

**Status:** Phase 1 Complete ✅ | Phase 2 Blocked on metadata complexity ⏸️
