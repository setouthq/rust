# Option A: Watt Framework - Status Report

**Date:** November 10, 2025

## Summary

Successfully integrated watt proc macro framework approach, but encountering runtime compatibility issues between wasm32-unknown-unknown (watt's standard target) and our wasm32-wasip1-threads environment.

## Progress Made

### 1. Built Watt Demo ✓
- Located watt repository at `/home/ubuntu/macovedj/watt`
- Successfully compiled watt demo to WASM using native rustc
- Output: `watt_demo.wasm` (492KB)
- Verified exports: `demo`, `raw_to_token_stream`, `token_stream_into_raw`

### 2. Metadata Injection ✓
- Created `inject_metadata.rs` tool to add `.rustc_proc_macro_decls` custom section
- Successfully injected metadata: `derive:Demo:demo\n`
- Output: `watt_demo_with_metadata.wasm`
- Verified metadata is readable by strings

### 3. Compilation Attempt ❌
**Command:**
```bash
wasmtime run -Sthreads=yes --dir .::/ dist/bin/rustc.wasm --sysroot dist \  test_watt_demo.rs --target wasm32-wasip1 \
  --extern Demo=watt_demo_with_metadata.wasm --edition 2021 \
  -o tmp/test_watt_demo.wasm
```

**Error:**
```
error: cannot find derive macro `Demo` in this scope
...
wasm trap: wasm `unreachable` instruction executed
```

## Root Cause Analysis

### Issue 1: Target Mismatch

**watt_demo.wasm compiled for:**
- Target: `wasm32-unknown-unknown`
- No threading support
- No WASI features
- Pure computation model

**Our runtime expects:**
- Target: `wasm32-wasip1` or `wasm32-wasip1-threads`
- WASI system interface
- Threading support (for rustc.wasm itself)

### Issue 2: Import/Export Compatibility

The watt interpreter (`compiler/rustc_watt_runtime/src/interpret.rs:122`) expects WASM modules to import from `"watt-0.5"`:

```rust
assert_eq!(module, "watt-0.5", "Wasm import from unknown module");
```

But `watt_demo.wasm` was compiled with watt's proc_macro2 fork, which expects to link against a specific import environment.

## Two Possible Solutions

### Solution 1: Compile Watt Proc Macros for WASI

**Approach:**
1. Modify watt's proc-macro2 to support wasm32-wasip1 target
2. Recompile demo with WASI support
3. Test with our runtime

**Pros:**
- More compatible with our WASI-based rustc
- Could support WASI features in proc macros

**Cons:**
- Requires modifying watt's proc-macro2
- May introduce complexity
- watt wasn't designed for WASI

### Solution 2: Make Watt Runtime Target-Agnostic

**Approach:**
1. Modify our watt runtime to load wasm32-unknown-unknown modules
2. Ensure the interpreter doesn't assume WASI features
3. Keep proc macros as pure wasm32-unknown-unknown

**Pros:**
- Follows watt's original design
- Simpler for proc macro authors
- Better sandboxing (no WASI access)

**Cons:**
- Runtime modifications needed
- Need to verify wasmtime can load mixed targets

## Current Status

The slot-based registry (Option 2) successfully solves the zero-sized closure problem. We now have two remaining tasks:

1. **Fix runtime compatibility** - Make watt interpreter work with wasm32-unknown-unknown modules
2. **Verify execution** - Ensure TokenStream marshaling works correctly

## Next Steps

### Immediate: Debug the Unreachable Trap

The trap is likely occurring when:
1. rustc.wasm tries to instantiate watt_demo.wasm
2. The watt interpreter calls `module_imports()` and `instantiate_module()`
3. Either imports don't match or there's a WASI/non-WASI conflict

**Action:** Add debug logging to watt runtime to see exactly where it fails.

### Alternative: Test with Simple Native Execution

Before testing in the full rustc.wasm context:
1. Create a standalone test that uses watt runtime directly
2. Load watt_demo.wasm and call it
3. Verify TokenStream marshaling works
4. Then integrate back into rustc

## Files Created

- `inject_metadata.rs` - Tool to inject proc macro metadata into WASM
- `watt_demo_with_metadata.wasm` - Watt demo with metadata section
- `test_watt_demo.rs` - Test file using the Demo derive macro
- `OPTION_B_ANALYSIS.md` - Analysis of why Option B is complex
- `OPTION_A_STATUS.md` - This file

## Key Insight

The fundamental challenge is not about TokenStream marshaling or the bridge protocol - **watt already solved that**. Our challenge is integrating watt's solution into a WASI-based rustc.wasm environment, which requires ensuring compatibility between:

- wasm32-unknown-unknown (proc macros)
- wasm32-wasip1-threads (rustc.wasm)
- The watt interpreter (bridging both)

## Recommendation

**Debug the unreachable trap** by enabling watt_debug feature and adding logging to understand exactly where the instantiation fails. Once we can successfully load and call the watt proc macro, we'll have completed Phase 3 (Runtime Execution).
