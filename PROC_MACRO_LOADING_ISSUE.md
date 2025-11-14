# Proc Macro Loading Issue - Root Cause Found

**Date:** November 10, 2025

## Problem

WASM proc macros aren't being loaded by rustc.wasm. The `--extern Demo=watt_demo_with_metadata.wasm` command fails with "cannot find derive macro `Demo`".

## Root Cause

The proc macro loading flow requires TWO components:

1. **Metadata file** (.rlib or .rmeta) that tells rustc:
   - This is a proc-macro crate
   - Here's where the dylib is located
   - Here are the proc macros it exports

2. **Dylib file** (.so/.dll or .wasm) that contains:
   - The actual proc macro implementation code
   - Export symbols for the proc macros

## Current Situation

When we use `--extern Demo=watt_demo_with_metadata.wasm`, rustc tries to:

1. Load metadata from `watt_demo_with_metadata.wasm`
2. Check `if crate_root.is_proc_macro_crate()` (line 441 of creader.rs)
3. Extract dylib path from metadata: `dlsym_source.dylib.as_ref()` (line 450)
4. Call `dlsym_proc_macros(dylib_path)` to load the actual code

**What fails:** Step 1 - the .wasm file doesn't have rustc metadata in the expected format

## Why It Fails

### Native Proc Macros
```
libfoo.so          ← Dylib with code
libfoo.rlib        ← Metadata: "I'm proc-macro, dylib is at libfoo.so"
```

When you `--extern foo=libfoo.rlib`:
1. Rustc reads metadata from libfoo.rlib
2. Finds it's a proc-macro crate
3. Extracts dylib path: "libfoo.so"
4. Calls dlsym on libfoo.so

### Our WASM Proc Macros
```
watt_demo.wasm     ← WASM module with code + .rustc_proc_macro_decls section
(no .rlib!)        ← Missing!
```

When you `--extern Demo=watt_demo.wasm`:
1. Rustc tries to read metadata from watt_demo.wasm
2. FAILS - WASM module doesn't have rustc metadata format
3. Never gets to step 3-4

## Solutions

### Solution A: Generate Proper .rlib During Compilation

**Modify rustc's proc-macro compilation for WASM:**

When compiling `--crate-type proc-macro --target wasm32-*`:
1. Generate the .wasm file (already works)
2. ALSO generate a .rlib file containing:
   - Crate metadata
   - Proc macro declarations from `.rustc_proc_macro_decls` section
   - Reference to the .wasm file as the "dylib"

**Files:** `compiler/rustc_codegen_ssa/src/back/link.rs` (linker)

**Pros:**
- Clean, standard approach
- Works with normal Cargo workflow
- Follows Rust conventions

**Cons:**
- Requires modifying linker/codegen code
- Need to understand .rlib format
- More complex implementation

### Solution B: Create .rlib Manually

**Post-process after compilation:**

1. Compile proc macro to .wasm with our rustc
2. Run a tool that creates a corresponding .rlib:
   ```bash
   create_proc_macro_rlib simple_test.wasm > libsimple_test.rlib
   ```
3. Use: `--extern simple_test=libsimple_test.rlib`

**Pros:**
- Doesn't require modifying rustc
- Can prototype quickly
- Easier to understand

**Cons:**
- Extra build step
- Manual workflow
- Not integrated with Cargo

### Solution C: Direct WASM Loading (Bypass Metadata)

**Add a new rustc flag:**

```bash
--wasm-proc-macro Demo=watt_demo.wasm
```

This would:
1. Skip metadata loading
2. Directly read .rustc_proc_macro_decls from WASM
3. Call dlsym_proc_macros_wasm

**Files:** `compiler/rustc_session/src/config.rs` (add flag)
**Files:** `compiler/rustc_metadata/src/creader.rs` (use flag)

**Pros:**
- Simpler than modifying codegen
- Direct path to testing
- No .rlib needed

**Cons:**
- Non-standard flag
- Doesn't work with Cargo
- Not the "right" long-term solution

## Recommendation

**For immediate testing: Solution C** (Direct WASM Loading)
- Gets us to Phase 3 completion fastest
- Can verify the watt runtime works
- Proves the concept

**For production: Solution A** (Proper .rlib Generation)
- Standard Rust approach
- Works with Cargo
- Long-term maintainable

## Implementation Plan for Solution C

### Step 1: Add Command-Line Flag

In `compiler/rustc_session/src/config.rs`:

```rust
pub struct Options {
    // ... existing fields ...
    pub wasm_proc_macros: Vec<(Symbol, PathBuf)>,
}
```

Parse `--wasm-proc-macro Name=path.wasm` into this vector.

### Step 2: Use Flag in Crate Loader

In `compiler/rustc_metadata/src/creader.rs`, add to `CrateLoader`:

```rust
fn load_wasm_proc_macros(&mut self) {
    for (name, path) in &self.sess.opts.wasm_proc_macros {
        let wasm_bytes = fs::read(path)?;
        let wasm_macro = WasmMacro::new_owned(wasm_bytes);
        let proc_macros = create_wasm_proc_macros(wasm_macro);

        // Register proc macros under the given name
        self.register_proc_macros(*name, proc_macros);
    }
}
```

Call this early in the compilation process, before resolving derives.

### Step 3: Test

```bash
wasmtime run -Sthreads=yes --dir .::/ dist/bin/rustc.wasm --sysroot dist \
  test_watt_demo.rs --target wasm32-wasip1 \
  --wasm-proc-macro Demo=watt_demo_with_metadata.wasm \
  --edition 2021 -o tmp/test_watt_demo.wasm
```

## Expected Results

With Solution C implemented:
1. `--wasm-proc-macro` flag directly loads the WASM
2. `create_wasm_proc_macros()` extracts metadata and creates ProcMacro instances
3. Slot-based registry holds WasmMacro references
4. When `#[derive(Demo)]` is encountered, slot function is called
5. Watt interpreter executes the WASM proc macro
6. TokenStream marshaling works correctly
7. **Phase 3 complete!**

## Files to Modify (Solution C)

1. `compiler/rustc_session/src/options.rs` - Add wasm_proc_macros field
2. `compiler/rustc_session/src/config.rs` - Parse --wasm-proc-macro flag
3. `compiler/rustc_interface/src/interface.rs` - Call load_wasm_proc_macros
4. `compiler/rustc_metadata/src/creader.rs` - Implement load_wasm_proc_macros

Estimated time: 2-3 hours of implementation + testing

## Next Steps

1. **Immediate:** Implement Solution C to unblock testing
2. **After verification:** Document findings and assess Solution A feasibility
3. **Future:** Full Cargo integration with proper .rlib generation

The slot-based registry (Option 2) is complete and working. This loading issue is the last blocker for Phase 3.
