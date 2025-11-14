# Option C: Use Standard --extern Flow (BREAKTHROUGH)

## Discovery

Instead of creating synthetic metadata or bypassing the system entirely, we can use the **standard `--extern` mechanism** that native proc-macros already use!

## How Native Proc-Macros Work

Native proc-macros consist of TWO files:
1. **`.dylib`/`.so`** - The compiled shared library containing proc macro code
2. **`.rmeta`** - The metadata file with crate information and proc macro declarations

When you use `--extern MyMacro=libmymacro.so`, rustc:
1. Loads `.rmeta` to get metadata (including proc macro names/types)
2. Calls `dlsym_proc_macros()` to load actual implementations from `.dylib`
3. Registers the crate in CStore with both metadata + raw proc macros

## Proposed Solution for WASM

Make WASM proc-macros work the same way:
1. **Compilation**: `rustc my_macro.rs --crate-type proc-macro --target wasm32-wasip1-threads`
   - Emits `libmy_macro.wasm` (the "dylib")
   - Emits `libmy_macro.rmeta` (the metadata) ← **Need to verify this happens**

2. **Usage**: `rustc user_code.rs --extern MyMacro=libmy_macro.wasm`
   - Standard extern resolution finds the `.rmeta` file
   - Loads metadata normally
   - When it sees `is_proc_macro_crate()`, calls `dlsym_proc_macros()`
   - Our existing `dlsym_proc_macros_wasm()` handles the WASM loading ✅ (already implemented!)

## What Needs To Be Done

### 1. Verify `.rmeta` emission for WASM proc-macros
Check if rustc already emits `.rmeta` files when compiling proc-macros to WASM.

### 2. Update crate locator (if needed)
Ensure `compiler/rustc_metadata/src/locator.rs` can find `.wasm` files as the "dylib" for proc-macro crates.

### 3. Test the flow
```bash
# Compile proc macro
wasmtime run dist/bin/rustc.wasm my_macro.rs \\
  --crate-type proc-macro \\
  --target wasm32-wasip1-threads \\
  --extern proc_macro=dist/lib/rustlib/wasm32-wasip1-threads/lib/libproc_macro-*.rlib

# Use it (standard --extern)
wasmtime run dist/bin/rustc.wasm user_code.rs \\
  --extern MyMacro=libmy_macro.wasm
```

## Advantages

✅ **No custom flags needed** - Uses standard `--extern`
✅ **No synthetic metadata** - Uses real metadata from compilation
✅ **No registration changes** - Standard flow works as-is
✅ **Minimal code changes** - Only locator adjustments (if any)
✅ **Already 90% done** - `dlsym_proc_macros_wasm()` already works!

## Status

- ✅ Phase 1: WASM loading and proc macro extraction working
- ✅ `dlsym_proc_macros_wasm()` implemented
- 🔄 Need to verify `.rmeta` generation for WASM proc-macros
- 🔄 May need locator updates to find `.wasm` as dylib
- ⏳ Test complete flow

## Next Steps

1. Check if proc-macro compilation to WASM already generates usable `.rmeta`
2. If not, investigate why and how to enable it
3. Test locator with `.wasm` files
4. End-to-end integration test
