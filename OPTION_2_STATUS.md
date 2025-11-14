# Option 2: Slot-Based Registry - Status Report

**Date:** November 10, 2025

## Summary

Successfully implemented a slot-based registry system that bypasses the "closure must be zero-sized" error in the proc_macro bridge. The implementation compiles successfully and progresses to runtime execution, where it encounters a different issue related to WASM module format expectations.

## Implementation Complete ✓

### Changes Made

**File:** `compiler/rustc_metadata/src/creader.rs`

1. **Slot Registry (lines 1304-1334)**
   - Global static registry with 256 slots
   - Each slot stores: WasmMacro reference, function name, and slot type
   - Thread-safe using `OnceLock<Mutex<Vec<Option<SlotData>>>>`

2. **Zero-Sized Function Items (lines 1336-2297)**
   - Generated 64 distinct functions for each slot (0-63)
   - Each slot has 3 functions: `slot_N_derive`, `slot_N_attr`, `slot_N_bang`
   - Functions are zero-sized because they don't capture any data
   - Each function looks up data from the global registry

3. **Client Factory Functions (lines 2298-2506)**
   - `make_derive_client(slot)` - Returns `Client<TokenStream, TokenStream>`
   - `make_attr_client(slot)` - Returns `Client<(TokenStream, TokenStream), TokenStream>`
   - `make_bang_client(slot)` - Returns `Client<TokenStream, TokenStream>`
   - Each factory matches on slot index and directly passes the zero-sized function item to `Client::expand1()` or `Client::expand2()`

### How It Works

```rust
// When a proc macro is loaded:
let slot = allocate_slot(SlotData {
    wasm_macro: &'static WasmMacro,
    function_name: "simple_test",
    slot_type: SlotType::Derive,
});

// Create client with zero-sized function item
ProcMacro::CustomDerive {
    trait_name: "SimpleTest",
    attributes: &[],
    client: make_derive_client(slot),  // Returns Client::expand1(slot_0_derive)
}

// When the proc macro is called:
fn slot_0_derive(input: TokenStream) -> TokenStream {
    let slots = get_slots().lock().unwrap();
    let data = slots[0].as_ref().expect("Slot 0 not initialized");
    data.wasm_macro.proc_macro_derive(data.function_name, input)
}
```

## Current Status

### ✅ Completed
- Type system requirements satisfied
- Zero-sized closure constraint bypassed
- Code compiles successfully
- Metadata generation works (Phase 2)

### ❌ Runtime Issue Discovered

**Error:** WASM trap - "unreachable instruction executed"

**Root Cause:** The watt interpreter expects WASM proc macros to export specific helper functions:
- `raw_to_token_stream` - Converts raw handles to TokenStream
- `token_stream_into_raw` - Converts TokenStream to raw handles

Standard Rust proc macros (compiled normally) don't export these functions. These are provided by the `watt` framework when you compile a proc macro specifically for watt.

**Location:** `compiler/rustc_watt_runtime/src/interpret.rs:80-99`

```rust
impl Exports {
    fn collect(instance: &ModuleInst, entry_point: &str) -> Self {
        let main = match get_export(instance, entry_point) {
            Ok(ExternVal::Func(main)) => main,
            _ => unimplemented!("unresolved macro: {:?}", entry_point),
        };
        let raw_to_token_stream = match get_export(instance, "raw_to_token_stream") {
            Ok(ExternVal::Func(func)) => func,
            _ => unimplemented!("raw_to_token_stream not found"),  // ← FAILS HERE
        };
        // ...
    }
}
```

## Next Steps

### Option A: Use Watt Framework
Compile test proc macros using the watt crate's infrastructure, which provides the required export functions.

**Pros:**
- Works with existing watt runtime
- Proven approach (used by watt crate)

**Cons:**
- Requires modifying how proc macros are compiled
- Not compatible with standard Rust proc macros

### Option B: Modify Watt Runtime
Adapt the watt interpreter to work with standard Rust proc macros by:
1. Generating the conversion functions at runtime
2. Providing our own bridge between standard proc macros and the interpreter

**Pros:**
- Works with any Rust proc macro
- More flexible

**Cons:**
- Significant runtime modifications needed
- Complex implementation

### Option C: Use Wasmtime Directly
Replace the watt interpreter with direct wasmtime execution:
1. Load WASM module with wasmtime
2. Set up WASI environment
3. Call proc macro functions directly

**Pros:**
- More standard approach
- Better performance
- Works with standard proc macros

**Cons:**
- Need to implement proc_macro bridge for WASM
- More complex integration

## Recommendation

**Option C (Wasmtime)** is the most promising because:
1. It works with standard Rust proc macros
2. It's a more maintainable long-term solution
3. It provides better performance
4. It's closer to how native proc macros work

The key challenge is implementing the proc_macro bridge to work across the WASM boundary, but this is a cleaner architecture than trying to adapt the watt interpreter.

## Files Modified

- `compiler/rustc_metadata/src/creader.rs` - Slot-based registry implementation

## Build Command

```bash
WASI_SDK_PATH=`pwd`/wasi-sdk-25.0-arm64-linux \
WASI_SYSROOT=`pwd`/wasi-sdk-25.0-arm64-linux/share/wasi-sysroot \
./x.py install --config config.llvm.toml -j 4
```

## Test Command

```bash
# Compile proc macro
wasmtime run -Sthreads=yes --dir .::/ dist/bin/rustc.wasm --sysroot dist \
  simple_test_macro.rs --target wasm32-wasip1-threads --crate-type proc-macro \
  -o simple_test_macro_v3.wasm

# Try to use it (currently fails at runtime)
wasmtime run -Sthreads=yes --dir .::/ dist/bin/rustc.wasm --sysroot dist \
  test_user.rs --target wasm32-wasip1 \
  --extern simple_test_macro=simple_test_macro_v3.wasm \
  --edition 2021 -o tmp/test_user_v3.wasm
```
