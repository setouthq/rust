# FINAL SOLUTION: WASM Proc Macro Implementation Path

**Date:** November 11, 2025

## Executive Summary

After comprehensive investigation, I've discovered the fundamental pattern behind how proc macros work in Rust. This discovery resolves all previous confusion and provides a clear path forward for implementing WASM proc macro support in rustc.wasm.

## The Breakthrough Discovery

**KEY INSIGHT:** Proc macros are NEVER used directly by user code. They must be accessed through a "wrapper library" that re-exports them.

### The Pattern (Native)

```
User Code  →  Wrapper Library  →  Proc Macro (.so)
  .rs          .rlib                 .so
```

Example with serde:
```
user_app.rs → serde.rlib → serde_derive.so
```

### How It Works

1. **Proc macro crate** is compiled to `.so` (or `.dll`, `.dylib`)
2. **Wrapper library** depends on proc macro, re-exports macros as `.rlib`
3. **User code** depends ONLY on wrapper library, never touches `.so`

## What We Discovered

### Investigation Timeline

1. ✅ **Discovered:** Proc macros don't generate `.rlib` files (CRITICAL_DISCOVERY.md)
2. ✅ **Investigated:** How Cargo passes proc macros to rustc (Cargo verbose build)
3. ✅ **Found:** Cargo passes `.so` files with `--extern` when compiling wrapper libraries
4. ✅ **Realized:** User code never sees the `.so` file
5. ✅ **Tested:** Cannot use `.so` files directly with `--extern` in user code
6. ✅ **Validated:** Wrapper library pattern works perfectly

### Test Results

**Working Example:**

```bash
# Step 1: Compile proc macro
rustc template_proc_macro.rs --crate-type proc-macro
# Output: libtemplate_proc_macro.so

# Step 2: Compile wrapper library
rustc template_lib.rs --crate-type lib \
  --extern template_proc_macro=libtemplate_proc_macro.so
# Output: libtemplate_lib.rlib

# Step 3: Compile user code
rustc test_template_v2.rs \
  --extern template_lib=libtemplate_lib.rlib \
  -L .
# Output: test_template_v2 (WORKS!)

# Step 4: Run
./test_template_v2
# Output: Hello from template macro!
```

## Solution for WASM Proc Macros

### Current Status

**Phase 1: ✅ COMPLETE**
- `--wasm-proc-macro` flag parsing
- WASM file loading
- Proc macro metadata extraction
- Integration with compilation flow

**Phase 2: ⏸️ NEEDS REVISION**
- Original plan: Register proc macros in CStore
- NEW understanding: Create synthetic "wrapper crate"

### The Real Solution: Synthetic Wrapper Crates

The `--wasm-proc-macro Demo=demo.wasm` flag should:

1. **Load WASM file** ✅ (Done in Phase 1)
   ```rust
   let wasm_bytes = fs::read("demo.wasm")?;
   let proc_macros = extract_metadata(&wasm_bytes)?;
   ```

2. **Create synthetic wrapper crate** (Phase 2 - NEW approach)
   ```rust
   // Conceptually creates a virtual crate:
   //
   // crate Demo {
   //   pub use <from wasm>::SomeMacro;
   //   pub use <from wasm>::AnotherMacro;
   // }
   ```

3. **Register wrapper in CStore** (Phase 2)
   ```rust
   let synthetic_crate = create_synthetic_wrapper_crate(
       "Demo",
       &proc_macros,
       wasm_runtime
   );
   tcx.cstore.register_crate(synthetic_crate);
   ```

4. **Make available to resolver** (Phase 2)
   - Resolver can find `Demo::SomeMacro`
   - When macro is used, execute WASM
   - Everything works naturally!

### Implementation Strategy

#### Option A: Full Synthetic CrateMetadata (Most Robust)

Create complete `CrateMetadata` with:
- MetadataBlob (minimal, containing just proc macro info)
- CrateRoot structure
- DefPathHashMap
- ProcMacro structs pointing to WASM runtime

**Pros:**
- Most compatible with rustc internals
- Follows existing patterns
- Robust and maintainable

**Cons:**
- Moderate complexity (1-2 days)
- Requires understanding metadata encoding

**Recommendation:** This is the RIGHT approach!

#### Option B: Minimal Wrapper Registration (Simpler)

Skip full CrateMetadata, directly register proc macros with resolver.

**Pros:**
- Faster implementation (4-6 hours)
- Less code

**Cons:**
- May break assumptions in rustc
- Less maintainable
- Might not work for all macro types

**Recommendation:** Use only if Option A proves too complex.

### Detailed Implementation Plan

#### Step 1: Design Synthetic Crate Structure (2 hours)

```rust
// compiler/rustc_metadata/src/wasm_wrapper.rs

pub struct SyntheticWasmCrate {
    name: Symbol,
    proc_macros: Vec<ProcMacroInfo>,
    wasm_runtime: WasmRuntime,
}

impl SyntheticWasmCrate {
    pub fn new(name: &str, wasm_path: PathBuf) -> Result<Self> {
        // Load WASM
        let wasm_bytes = fs::read(&wasm_path)?;

        // Extract metadata
        let proc_macros = extract_proc_macro_metadata(&wasm_bytes)?;

        // Initialize WASM runtime
        let wasm_runtime = WasmRuntime::new(wasm_bytes)?;

        Ok(Self {
            name: Symbol::intern(name),
            proc_macros,
            wasm_runtime,
        })
    }

    pub fn to_crate_metadata(&self, tcx: TyCtxt) -> CrateMetadata {
        // Create minimal metadata
        let metadata_blob = self.create_metadata_blob();

        // Create CrateRoot
        let crate_root = self.create_crate_root(tcx);

        // Create CrateMetadata
        CrateMetadata {
            name: self.name,
            blob: metadata_blob,
            root: crate_root,
            // ... other fields with sensible defaults
        }
    }
}
```

#### Step 2: Metadata Blob Creation (3-4 hours)

```rust
impl SyntheticWasmCrate {
    fn create_metadata_blob(&self) -> MetadataBlob {
        // Create minimal rustc metadata
        // Must include:
        // - Crate name
        // - Proc macro declarations
        // - Export information

        let mut encoder = MetadataEncoder::new();

        // Encode crate name
        encoder.encode_crate_name(&self.name);

        // Encode proc macros
        for pm in &self.proc_macros {
            encoder.encode_proc_macro(pm);
        }

        MetadataBlob::new(encoder.finish())
    }

    fn create_crate_root(&self, tcx: TyCtxt) -> CrateRoot {
        // Create minimal CrateRoot
        // Most fields can be default/empty
        // Only proc macro info needs to be accurate

        CrateRoot {
            name: self.name,
            extra_filename: String::new(),
            proc_macros: self.create_proc_macro_array(),
            // ... other fields with defaults
        }
    }
}
```

#### Step 3: Integration with creader.rs (1-2 hours)

```rust
// compiler/rustc_metadata/src/creader.rs

impl<'a, 'tcx> CrateLoader<'a, 'tcx> {
    pub fn load_wasm_proc_macros(&mut self) {
        for (name, path) in &self.sess.opts.wasm_proc_macros {
            // Create synthetic crate
            let synthetic = SyntheticWasmCrate::new(name, path.clone())
                .expect("Failed to load WASM proc macro");

            // Convert to CrateMetadata
            let metadata = synthetic.to_crate_metadata(self.tcx);

            // Register in CStore
            let cnum = self.cstore.alloc_new_crate_num();
            self.cstore.set_crate_data(cnum, metadata);

            // Register with resolver
            self.register_crate_for_resolution(cnum, name);
        }
    }
}
```

#### Step 4: WASM Execution Integration (2-3 hours)

```rust
// When a proc macro is invoked:

impl ProcMacro for WasmProcMacro {
    fn expand(
        &self,
        input: TokenStream,
    ) -> Result<TokenStream> {
        // Execute WASM module
        let output = self.runtime.call_proc_macro(
            self.name,
            input,
        )?;

        Ok(output)
    }
}
```

#### Step 5: Testing (2-3 hours)

```bash
# Test 1: Basic derive macro
wasmtime run -Sthreads=yes --dir . dist/bin/rustc.wasm \
  --sysroot dist \
  --wasm-proc-macro Demo=watt_demo_with_metadata.wasm \
  test_watt_demo.rs \
  --target wasm32-wasip1 \
  --edition 2021 \
  -o test.wasm

# Expected: Compiles successfully!
# #[derive(Demo)] works!

# Test 2: Multiple macros
wasmtime run -Sthreads=yes --dir . dist/bin/rustc.wasm \
  --sysroot dist \
  --wasm-proc-macro Demo=demo.wasm \
  --wasm-proc-macro Helper=helper.wasm \
  complex_test.rs \
  --target wasm32-wasip1 \
  --edition 2021 \
  -o test.wasm

# Expected: Both macros available!

# Test 3: Attribute macros
# Test 4: Function-like macros
```

### Total Estimated Time

- Step 1 (Design): 2 hours
- Step 2 (Metadata): 3-4 hours
- Step 3 (Integration): 1-2 hours
- Step 4 (Execution): 2-3 hours
- Step 5 (Testing): 2-3 hours

**Total: 10-14 hours** for complete implementation

## Why This Will Work

### 1. Follows Native Pattern

Native: `proc_macro.so` → `wrapper.rlib` → `user_code`
WASM: `proc_macro.wasm` → `synthetic.rlib` → `user_code`

### 2. Minimal rustc Changes

- No changes to resolver logic
- No changes to proc macro expansion
- Just adds synthetic crate registration

### 3. Works with Existing Infrastructure

- Uses existing CrateMetadata
- Uses existing proc macro system
- Uses existing WASM runtime (watt/wasmtime)

### 4. Natural User Experience

```rust
// Just works like native!
use Demo::SomeMacro;

#[derive(SomeMacro)]
struct Foo {
    // ...
}
```

## Files Created During Investigation

### Documentation
1. **CRITICAL_DISCOVERY.md** - Discovered proc macros don't use .rlib files
2. **WASM_PROC_MACRO_SUMMARY.md** - Previous comprehensive summary
3. **PROC_MACRO_PATTERN_DISCOVERED.md** - Pattern breakthrough
4. **FINAL_PROC_MACRO_SOLUTION.md** - This file

### Test Files
1. **template_proc_macro.rs** - Test proc macro
2. **libtemplate_proc_macro.so** - Compiled proc macro
3. **template_lib.rs** - Wrapper library (KEY!)
4. **libtemplate_lib.rlib** - Compiled wrapper
5. **test_template_v2.rs** - Working user code
6. **test_template_v2** - Working binary

### Cargo Test
1. **/tmp/test_proc_macro_usage/** - Cargo test project
2. **/tmp/cargo_verbose_build.log** - Cargo verbose output analysis

## Comparison: Before vs After Understanding

### Before (Wrong Understanding)
```
WASM proc macro → (somehow generate .rlib) → rustc finds it
```
- Tried to generate .rlib files
- Tried to use .so files directly
- Couldn't understand why Cargo worked

### After (Correct Understanding)
```
WASM proc macro → Synthetic wrapper crate → Resolver → User code
                  (acts like serde pattern)
```
- Understands wrapper pattern
- Knows synthetic crate is needed
- Clear implementation path

## Success Criteria

When implementation is complete:

- ✅ `--wasm-proc-macro Demo=demo.wasm` registers synthetic crate
- ✅ User can `use Demo::SomeMacro;`
- ✅ `#[derive(SomeMacro)]` resolves and expands correctly
- ✅ WASM module executes during expansion
- ✅ Code generates correctly
- ✅ Multiple WASM proc macros work simultaneously
- ✅ All proc macro types work (derive, attribute, function-like)

## Next Action

**Implement Phase 2 with synthetic wrapper crate approach.**

Start with Step 1 (Design) and proceed through each step systematically.

Estimated timeline: **2-3 days** for complete, tested implementation.

## Conclusion

The investigation was highly valuable:
- Discovered the fundamental proc macro pattern
- Validated it with working native examples
- Found the correct approach for WASM implementation
- Have a clear, achievable implementation plan

**The path forward is clear and straightforward.**

---

**Status:** Investigation complete. Implementation plan ready. Awaiting go-ahead to implement Phase 2.
