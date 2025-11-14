# WASM Procedural Macro Implementation Status - Final

## Date
2025-11-14

## Implementation Approach
Solution A: Skip CStore registration entirely - store WASM proc macros directly in resolver

## Components Implemented

### 1. WASM Loading (✅ Complete)
- **File**: `compiler/rustc_metadata/src/creader.rs`
- **Functionality**:
  - Loads WASM files via `--wasm-proc-macro` flag
  - Extracts metadata from custom sections
  - Converts watt ProcMacro to rustc SyntaxExtension
  - Successfully loads and extracts macros from WASM files

### 2. Resolver Storage (✅ Complete)
- **File**: `compiler/rustc_resolve/src/lib.rs`
- **Functionality**:
  - Added `wasm_proc_macros: FxHashMap<Symbol, MacroData>` for name-based lookup
  - Added `wasm_proc_macro_def_id_counter: u32` for DefId generation
  - Added `wasm_proc_macro_def_id_to_name: FxHashMap<DefId, Symbol>` for reverse lookup
  - Implemented `register_wasm_proc_macros()` function
  - Stores macros in both `wasm_proc_macros` AND `macro_map` (DefId-based)
  - Uses sequential DefIndex values (1, 2, 3, etc.) for unique DefIds

### 3. Name Resolution Hook (✅ Complete)
- **File**: `compiler/rustc_resolve/src/ident.rs`
- **Functionality**:
  - Hooks into `Scope::MacroUsePrelude` in identifier resolution
  - Checks `wasm_proc_macros` map before standard prelude
  - Looks up corresponding DefId from `wasm_proc_macro_def_id_to_name`
  - Returns `NameBinding` with correct DefId and `Res::Def(DefKind::Macro(MacroKind::Derive), def_id)`

## Test Results

### Build Status
- ✅ rustc compiles successfully with all modifications
- ✅ Build time: ~8 minutes
- ⚠️  Minor warnings about unused imports (non-blocking)

### Runtime Testing
Test file: `test_watt_demo.rs`
```rust
#[derive(Demo)]
struct MyStruct;

fn main() {
    println!("{}", MyStruct::MESSAGE);
}
```

WASM file: `watt_demo_with_metadata.wasm` (503KB, contains Demo derive macro)

#### Test Output Analysis

**What Works:**
1. ✅ WASM file loaded: "Read 503343 bytes from watt_demo_with_metadata.wasm"
2. ✅ Metadata extraction: "Found 1 metadata entries"
3. ✅ Macro registration: "Registering WASM proc macro: Demo"
4. ✅ DefId assignment: "Assigned synthetic DefId DefId(0:1 ~ test_watt_demo[bab8])"
5. ✅ Name resolution: "Found WASM proc macro: Demo" (appears 3 times)
6. ✅ DefId lookup: "Using DefId DefId(0:1 ~ test_watt_demo[bab8])"
7. ✅ Stored in both maps: `wasm_proc_macros` and `macro_map`

**What Doesn't Work:**
❌ Macro expansion fails with error:
```
error: cannot determine resolution for the derive macro `Demo`
 --> test_watt_demo.rs:1:10
  |
1 | #[derive(Demo)]
  |          ^^^^
  |
  = note: import resolution is stuck, try simplifying macro imports
```

## Root Cause Analysis

The error originates from `compiler/rustc_resolve/src/macros.rs:857` in the `finalize_macro_resolutions()` function. This function runs POST-expansion and checks if macros that were used were properly resolved.

The condition triggering the error (line 848):
```rust
if initial_res.is_none() && this.tcx.dcx().has_errors().is_none() && this.privacy_errors.is_empty()
```

This means:
- `initial_res` is `None` - the macro was NOT resolved during the initial speculative resolution phase
- No other errors have been reported
- Therefore, it reports "cannot determine resolution"

### Resolution Flow for Derive Macros

1. **Initial Resolution** (happens during macro expansion):
   - For single-segment paths like `Demo`, calls `early_resolve_ident_in_lexical_scope()`
   - This goes through our hook in `ident.rs` at `Scope::MacroUsePrelude`
   - Our hook DOES find the macro and returns a NameBinding
   - The binding is stored in `single_segment_macro_resolutions` with the result

2. **Finalization** (happens post-expansion):
   - Checks `single_segment_macro_resolutions`
   - If `initial_res` (from step 1) is None, reports error
   - But our hook IS returning a binding, so why is `initial_res` None?

### Hypothesis

The most likely issue is that even though our hook returns a NameBinding with the correct DefId, something in the resolution flow is:

1. Not properly recording the result in `initial_res`, OR
2. The binding we return doesn't have the right properties/flags, OR
3. There's another check earlier that's failing and preventing `initial_res` from being set, OR
4. The macro expansion itself is failing silently, causing `initial_res` to remain None

The fact that we see "Found WASM proc macro: Demo" 3 times suggests the resolution is being attempted multiple times, which might indicate retries or different resolution phases.

## Key Implementation Details

### DefId Generation Strategy

**Evolution:**
1. First attempt: `u32::MAX - counter` → Failed (DefIndex > 0xFFFF_FF00)
2. Second attempt: `0xFFFF_FF00 - counter` → Failed (still too high for definitions table)
3. Third attempt: `CRATE_DEF_INDEX` (0:0) → Failed (all macros shared same DefId, causing overwrites in macro_map)
4. Current: Sequential from 1 (`1 + counter`) → No crashes, but expansion fails

### Storage Architecture

Macros are stored in TWO places:
1. `wasm_proc_macros`: HashMap<Symbol, MacroData> - for name-based lookup during resolution
2. `macro_map`: HashMap<DefId, MacroData> - for DefId-based lookup during expansion

This dual storage ensures:
- Name resolution can find macros by name
- Expansion can retrieve SyntaxExtension by DefId
- Each macro has a unique DefId (no overwrites)

## Next Steps to Investigate

1. **Add debug logging to `resolve_macro_path`** to see what's being returned when "Demo" is resolved

2. **Check if binding has correct properties** - does the NameBinding we create need additional flags or metadata?

3. **Investigate `single_segment_macro_resolutions`** - is our binding actually being stored correctly? Can we log what gets recorded?

4. **Check macro expansion phase** - is the SyntaxExtension being retrieved from `macro_map` when expansion is attempted?

5. **Verify DefId validity** - even though DefIndex 1 is low, does the definitions table actually have an entry at that index? Or does it need to?

6. **Consider if derive macros need special handling** - do they go through a different expansion path than we've hooked?

## Files Modified

1. `/home/ubuntu/macovedj/rust/compiler/rustc_metadata/src/creader.rs`
   - Added WASM proc macro loading
   - Implemented ProcMacro to SyntaxExtension conversion

2. `/home/ubuntu/macovedj/rust/compiler/rustc_resolve/src/lib.rs`
   - Added WASM proc macro storage fields
   - Implemented registration function
   - Stores in both wasm_proc_macros and macro_map

3. `/home/ubuntu/macovedj/rust/compiler/rustc_resolve/src/ident.rs`
   - Added MacroUsePrelude scope check for WASM macros
   - Returns NameBinding with correct DefId

## Conclusion

We've successfully implemented the infrastructure for loading and storing WASM procedural macros. The resolution phase correctly finds the macros by name. However, there's a disconnect between resolution and expansion - the macro is found but not properly expanded.

The issue appears to be in how the resolution result is communicated to the expansion phase, or possibly in the macro expansion phase itself not being able to retrieve/execute the WASM macro.

Further investigation is needed to understand:
- Why `initial_res` remains None despite successful resolution
- Whether the issue is in our NameBinding creation, the expansion phase, or somewhere in between
- If there are additional hooks or registrations needed for derive macros specifically
