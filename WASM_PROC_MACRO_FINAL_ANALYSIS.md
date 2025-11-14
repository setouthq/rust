# WASM Proc Macro Implementation - Final Analysis

## What We've Tried

### Attempt 1: Synthetic Metadata from Scratch
- **Approach**: Create minimal metadata blob manually
- **Problem**: Can't create valid encoded CrateRoot without using the encoder
- **Blocker**: FileEncoder requires filesystem, which isn't available in WASM

### Attempt 2: Pre-loaded Proc Macros with Metadata Template
- **Approach**: Use real metadata from compiled proc-macro crate as template
- **Implementation**: Modified `register_crate()` to accept pre-loaded proc macros
- **Problem**: Template metadata contains encoded strings/data for `examine_proc_macro` crate
- **Error**: `assertion failed: bytes[len] == STR_SENTINEL` when decoding strings
- **Root Cause**: rustc actually decodes the CrateRoot to get crate name, dependencies, stable_crate_id, etc.

## The Fundamental Issue

**rustc's architecture assumes proc macros come from registered crates with valid metadata.**

When we call `register_crate()`:
1. It calls `metadata.get_root()` which decodes the CrateRoot
2. It reads crate name, hash, dependencies from the decoded metadata
3. It creates a `StableCrateId` from the metadata
4. It registers the crate in CStore with this information

Even with pre-loaded proc macros, we still need:
- Valid, decode-able metadata
- Matching crate name and hash
- Proper StableCrateId

## Two Possible Solutions

### Solution A: Skip CStore Registration Entirely
**Don't register WASM proc macros as crates at all.**

Instead:
1. Load WASM file and extract proc macros
2. Store them in a separate `HashMap<Symbol, ProcMacro>`
3. Hook into macro resolution to check this map first
4. When a macro is requested, return it directly without going through CStore

**Pros**:
- No metadata needed
- Clean separation of concerns
- WASM proc macros are truly external

**Cons**:
- Requires modifying macro resolution system
- Need to understand how macros are looked up
- More invasive changes to rustc

### Solution B: Generate Real Metadata On-Demand
**Compile a dummy proc-macro crate for each WASM file.**

When loading a WASM proc macro:
1. Extract proc macro names from WASM
2. Generate a `.rs` file with those proc macros (empty implementations)
3. Compile it to get real metadata
4. Use that metadata for CStore registration
5. Pass pre-loaded WASM proc macros to override the empty implementations

**Pros**:
- Works within existing architecture
- Metadata is always valid

**Cons**:
- Complex: requires invoking rustc recursively
- Requires filesystem (may not work in WASM rustc)
- Slow: compilation for every WASM file

## Recommendation

**Solution A** is cleaner and more appropriate for "external" proc macros. WASM proc macros are fundamentally different from native crates - they're pre-compiled, runtime-loaded plugins. They shouldn't pretend to be crates.

The implementation would:
1. Add a field to store WASM proc macros: `wasm_proc_macros: FxHashMap<Symbol, Vec<ProcMacro>>`
2. Modify macro lookup to check this map first
3. Keep the existing WASM loading infrastructure

## Current Status

We have working code for:
- ✅ WASM loading and interpretation (rustc_watt_runtime)
- ✅ Proc macro extraction from WASM custom sections
- ✅ Slot-based registry (64 static slots)
- ✅ Command-line flag parsing (`--wasm-proc-macro`)
- ✅ Pre-loaded proc macro support in `register_crate()`

What's needed:
- ❌ Alternative to CStore registration for WASM proc macros
- ❌ Hook into macro resolution system

## Files Modified So Far

1. `compiler/rustc_metadata/src/creader.rs` - Added `load_wasm_proc_macros()` and pre-loaded support
2. `compiler/rustc_metadata/src/wasm_synthetic.rs` - Metadata generation attempts
3. `compiler/rustc_session/src/options.rs` - Added `--wasm-proc-macro` flag
4. `compiler/rustc_watt_runtime/` - Entire WASM runtime crate

## Next Steps

If pursuing Solution A:
1. Find where macros are resolved (likely in `rustc_resolve`)
2. Add storage for WASM proc macros
3. Modify lookup to check WASM macros first
4. Remove CStore registration attempt

If pursuing Solution B:
1. Implement on-demand metadata generation
2. Test if it works in WASM rustc environment
