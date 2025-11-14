# Phase 2: Proc Macro Registration Plan

**Date:** November 10, 2025

## Current Status

✅ **Phase 1 Complete** - The `--wasm-proc-macro` flag infrastructure works:
- Command-line flag parsing ✓
- WASM file loading ✓ (503KB loaded successfully)
- Metadata extraction ✓ (1 proc macro found)
- Integration with resolver ✓

⏳ **Phase 2 Remaining** - Register proc macros so resolver can find them

## The Challenge

The proc macros are loaded and extracted but not registered in rustc's cstore. The resolver cannot find them by name, resulting in:
```
error: cannot find derive macro `Demo` in this scope
```

## What Needs to Happen

For the resolver to find proc macros, they must be registered in the `CStore` as `CrateMetadata`. This requires:

1. **Allocate CrateNum** - Get a unique identifier for this proc macro crate
2. **Create CrateMetadata** - Package the proc macros with required metadata
3. **Register in CStore** - Call `cstore.set_crate_data(cnum, metadata)`

## The Complexity

### CrateMetadata Requirements

Looking at `compiler/rustc_metadata/src/rmeta/decoder.rs:76-125`:

```rust
pub(crate) struct CrateMetadata {
    blob: MetadataBlob,              // ❌ Complex: Binary metadata format
    root: CrateRoot,                  // ❌ Complex: Crate-level metadata
    trait_impls: FxIndexMap<...>,    // ✓ Can be empty
    incoherent_impls: FxIndexMap<...>, // ✓ Can be empty
    raw_proc_macros: Option<&'static [ProcMacro]>, // ✅ We have this!
    source_map_import_info: Lock<Vec<...>>, // ✓ Can be empty
    def_path_hash_map: DefPathHashMapRef<'static>, // ❌ Complex
    expn_hash_map: OnceLock<...>,    // ✓ Can be empty
    alloc_decoding_state: AllocDecodingState, // ✓ Can be empty
    def_key_cache: Lock<...>,        // ✓ Can be empty
    cnum: CrateNum,                   // ✅ We allocate this
    cnum_map: CrateNumMap,            // ✓ Can be empty (no dependencies)
    dependencies: Vec<CrateNum>,      // ✓ Empty (standalone proc macro)
    dep_kind: CrateDepKind,           // ✅ MacrosOnly
    source: Lrc<CrateSource>,         // ⚠️ Medium: Point to WASM file
    private_dep: bool,                // ✅ false
    host_hash: Option<Svh>,           // ✅ None
    used: bool,                       // ✅ true
}
```

### The Difficult Parts

**1. MetadataBlob** (`blob`)
- This is the encoded .rmeta file format
- Contains serialized crate information
- Required for most CrateMetadata operations
- **Problem**: We don't have a .rmeta file, only a .wasm file
- **Possible solution**: Create minimal synthetic metadata

**2. CrateRoot** (`root`)
- Top-level crate metadata structure
- Includes: name, hash, edition, dependencies, items, etc.
- Used by many queries and operations
- **Problem**: Complex to construct from scratch
- **Possible solution**: Create minimal CrateRoot with just name/hash

**3. DefPathHashMap** (`def_path_hash_map`)
- Maps `DefPathHash` → `DefIndex` for all definitions
- Used for efficient def lookup
- **Problem**: We have no definitions (proc macros are special)
- **Possible solution**: Empty map?

## Proposed Solutions

### Option A: Full Synthetic Metadata (Most Compatible)

Create complete but minimal synthetic metadata:

```rust
impl CrateMetadata {
    pub fn new_synthetic_proc_macro(
        sess: &Session,
        cstore: &CStore,
        name: Symbol,
        proc_macros: &'static [ProcMacro],
        wasm_path: &Path,
        cnum: CrateNum,
    ) -> Self {
        // 1. Create minimal MetadataBlob
        let blob = MetadataBlob::new_synthetic_proc_macro(name);

        // 2. Create minimal CrateRoot
        let root = CrateRoot {
            name,
            hash: Svh::new(&[0; 20]), // Synthetic hash
            stable_crate_id: StableCrateId::new(name, /* ... */),
            required_panic_strategy: None,
            panic_in_drop_strategy: PanicStrategy::Unwind,
            edition,
            has_global_allocator: false,
            has_alloc_error_handler: false,
            has_panic_handler: false,
            has_default_lib_allocator: false,
            proc_macro_data: Some(/* ... */),
            // ... minimal required fields
        };

        // 3. Create CrateSource pointing to WASM file
        let source = Lrc::new(CrateSource {
            dylib: None,
            rlib: None,
            rmeta: Some((wasm_path.to_path_buf(), PathKind::All)),
        });

        // 4. Assemble CrateMetadata
        CrateMetadata {
            blob,
            root,
            trait_impls: Default::default(),
            incoherent_impls: Default::default(),
            raw_proc_macros: Some(proc_macros),
            source_map_import_info: Default::default(),
            def_path_hash_map: DefPathHashMap::default(),
            expn_hash_map: Default::default(),
            alloc_decoding_state: Default::default(),
            def_key_cache: Default::default(),
            cnum,
            cnum_map: CrateNumMap::default(),
            dependencies: vec![],
            dep_kind: CrateDepKind::MacrosOnly,
            source,
            private_dep: false,
            host_hash: None,
            used: true,
        }
    }
}
```

**Pros:**
- Most compatible with existing rustc infrastructure
- All code paths that expect CrateMetadata will work
- Clean separation of concerns

**Cons:**
- Requires implementing `MetadataBlob::new_synthetic_proc_macro()`
- Requires implementing `CrateRoot` construction
- Complex to get all details right

### Option B: Bypass CrateMetadata (Simpler)

Add a separate registry for WASM proc macros in the resolver:

```rust
// In Resolver
pub struct Resolver<'a, 'tcx> {
    // ... existing fields
    wasm_proc_macros: FxHashMap<Symbol, &'static [ProcMacro]>,
}

// In load_wasm_proc_macros()
pub fn load_wasm_proc_macros(&mut self) {
    for (name, path) in &self.sess.opts.wasm_proc_macros {
        let wasm_bytes = fs::read(path)?;
        let wasm_macro = WasmMacro::new_owned(wasm_bytes);
        let proc_macros = create_wasm_proc_macros(wasm_macro);
        let name_symbol = Symbol::intern(name);

        // Store in resolver directly
        self.register_wasm_proc_macro(name_symbol, Box::leak(proc_macros));
    }
}

// In resolver macro lookup
fn resolve_macro_path(&mut self, ...) {
    // First check wasm_proc_macros map
    if let Some(proc_macros) = self.wasm_proc_macros.get(&name) {
        return Some(proc_macros);
    }

    // Then fall back to normal cstore lookup
    // ...
}
```

**Pros:**
- Much simpler implementation
- Bypasses complex metadata machinery
- Direct path from flag to usage

**Cons:**
- Requires modifying Resolver struct
- Not consistent with how other proc macros work
- May miss edge cases that normal path handles

### Option C: Minimal Stub Metadata (Pragmatic Middle Ground)

Create the bare minimum CrateMetadata that satisfies the resolver:

```rust
// In load_wasm_proc_macros() after extracting proc_macros:
let name_symbol = Symbol::intern(name);

// Allocate CrateNum (simplified - may need refinement)
let cnum = CrateNum::from_usize(self.cstore.metas.len());
self.cstore.metas.push(None);

// Create minimal stub metadata
// This is a HACK but might work for testing
let stub_metadata = create_stub_proc_macro_metadata(
    name_symbol,
    Box::leak(proc_macros),
    cnum,
    path,
);

self.cstore.set_crate_data(cnum, stub_metadata);
eprintln!("[CREADER] Registered proc macro crate: {} as {:?}", name, cnum);
```

Where `create_stub_proc_macro_metadata` creates the absolute minimum:

```rust
fn create_stub_proc_macro_metadata(
    name: Symbol,
    proc_macros: &'static [ProcMacro],
    cnum: CrateNum,
    wasm_path: &Path,
) -> CrateMetadata {
    // This is a stub - many fields will panic if accessed
    // But proc macro lookup might work

    // Try to use existing CrateMetadata::new() with dummy values
    // OR create fields manually with unsafe/transmute as last resort
}
```

**Pros:**
- Faster to implement than Option A
- More compatible than Option B
- Good for rapid prototyping/testing

**Cons:**
- Fragile - may panic if wrong code path is taken
- Not production-ready
- Hard to debug when it fails

## Recommended Approach

**For immediate testing:** Start with **Option C** (Minimal Stub)
- Quick to implement
- Proves the concept
- Identifies what's actually required

**For production:** Implement **Option A** (Full Synthetic)
- Robust and maintainable
- Consistent with rustc architecture
- Handles edge cases properly

## Implementation Steps for Option C (Quick Test)

1. **Add helper function in `creader.rs`:**
   ```rust
   fn alloc_crate_num(&mut self) -> CrateNum {
       let cnum = CrateNum::from_usize(self.cstore.metas.len());
       self.cstore.metas.push(None);
       cnum
   }
   ```

2. **Modify `load_wasm_proc_macros()` to register:**
   ```rust
   let cnum = self.alloc_crate_num();
   let name_symbol = Symbol::intern(name);

   // Try to create minimal metadata
   // Start by looking at what CrateMetadata::new() requires
   ```

3. **Test and iterate:**
   - See what panics/fails
   - Add minimum fields to make it work
   - Document what's actually needed

## Key Questions to Answer

1. **How does the resolver look up proc macros?**
   - By crate name in cstore?
   - By scanning all crates for proc_macros field?
   - Need to trace through resolver code

2. **What's the minimum viable CrateMetadata?**
   - Which fields are actually accessed for proc macros?
   - Can we stub out complex fields with panics?
   - What breaks if we use dummy values?

3. **How are proc macro crates normally registered?**
   - Study the normal path in `register_crate()`
   - What happens after `dlsym_proc_macros()` succeeds?
   - Can we replicate just that part?

## Files to Modify

### For Option C (Minimal):
1. `compiler/rustc_metadata/src/creader.rs`
   - Modify `load_wasm_proc_macros()` to register
   - Add `alloc_crate_num()` helper
   - Add `create_stub_proc_macro_metadata()` helper

### For Option A (Full):
1. `compiler/rustc_metadata/src/rmeta/decoder.rs`
   - Add `CrateMetadata::new_synthetic_proc_macro()`
   - Add `MetadataBlob::new_synthetic()`
   - Add `CrateRoot::new_synthetic()`

2. `compiler/rustc_metadata/src/creader.rs`
   - Modify `load_wasm_proc_macros()` to use new constructor

### For Option B (Bypass):
1. `compiler/rustc_resolve/src/lib.rs`
   - Add `wasm_proc_macros` field to `Resolver`
   - Modify macro resolution to check this map first

2. `compiler/rustc_metadata/src/creader.rs`
   - Pass proc macros to resolver instead of cstore

## Next Steps

1. **Investigate resolver macro lookup**
   - Find where "cannot find derive macro" error is generated
   - Trace back to how it looks up proc macros
   - Understand the lookup mechanism

2. **Try Option C implementation**
   - Start with absolute minimum
   - See what breaks
   - Add only what's needed

3. **Document findings**
   - What actually works
   - What fields are required
   - Path to Option A if needed

## Success Criteria

When this is working:
- `#[derive(Demo)]` does not error
- Proc macro expansion is attempted
- TokenStream marshaling is tested
- Code generation works correctly

## Time Estimate

- **Option C (Minimal Stub):** 2-4 hours of trial and error
- **Option A (Full Synthetic):** 1-2 days of careful implementation
- **Option B (Bypass):** 4-6 hours but may have hidden issues

## Related Code References

- `CrateMetadata` struct: `compiler/rustc_metadata/src/rmeta/decoder.rs:76`
- `CStore::set_crate_data()`: `compiler/rustc_metadata/src/creader.rs:210`
- `intern_stable_crate_id()`: `compiler/rustc_metadata/src/creader.rs:172`
- Normal crate registration: `compiler/rustc_metadata/src/creader.rs:400-526`
- Proc macro loading: `compiler/rustc_metadata/src/creader.rs:702-769`

---

**Status:** Ready to implement. Recommend starting with Option C for rapid feedback, then moving to Option A for production quality.
