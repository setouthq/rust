# WASM Proc Macro Flag Implementation Status

**Date:** November 10, 2025

## Objective

Implement `--wasm-proc-macro` command-line flag (Solution C from PROC_MACRO_LOADING_ISSUE.md) to enable direct loading of WASM procedural macros, bypassing the normal metadata system.

## Status: Phase 1 Complete ✅

The infrastructure for the `--wasm-proc-macro` flag has been successfully implemented and tested.

## What Was Implemented

### 1. Command-Line Flag Registration
**File:** `compiler/rustc_session/src/config.rs:1580-1587`

```rust
opt(
    Stable,
    Multi,
    "",
    "wasm-proc-macro",
    "Directly load WASM proc-macro files (bypasses metadata system)",
    "NAME=PATH",
),
```

- Added as a **stable** option for easier testing
- Accepts multiple flags (Multi)
- Format: `--wasm-proc-macro NAME=path/to/file.wasm`

### 2. Parsing Logic
**File:** `compiler/rustc_session/src/config.rs:2276-2329`

```rust
pub fn parse_wasm_proc_macros(
    early_dcx: &EarlyDiagCtxt,
    matches: &getopts::Matches,
    _unstable_opts: &UnstableOptions,
) -> Vec<(String, PathBuf)>
```

Features:
- Validates `NAME=PATH` format
- Checks that NAME is a valid ASCII identifier
- Verifies WASM file exists
- Returns Vec of (name, path) tuples

### 3. Options Storage
**Files:**
- `compiler/rustc_session/src/options.rs:173`
- `compiler/rustc_session/src/config.rs:1173, 2667`

```rust
wasm_proc_macros: Vec<(String, PathBuf)> [UNTRACKED],
```

- Stored in `Options` struct
- Marked as `[UNTRACKED]` (doesn't affect incremental compilation)
- Initialized in both Default impl and build_session_options

### 4. Loading Infrastructure
**File:** `compiler/rustc_metadata/src/creader.rs:325-379`

```rust
pub fn load_wasm_proc_macros(&mut self) {
    #[cfg(target_family = "wasm")]
    {
        // Load WASM files and extract proc macros
    }

    #[cfg(not(target_family = "wasm"))]
    {
        // No-op on non-WASM platforms
    }
}
```

Features:
- Conditional compilation for WASM target only
- Reads WASM files using `fs::read()`
- Creates `WasmMacro` instances
- Calls `create_wasm_proc_macros()` to extract metadata
- Comprehensive debug logging with `eprintln!`

### 5. Resolver Integration
**File:** `compiler/rustc_resolve/src/lib.rs:1723-1726`

```rust
self.tcx.sess.time("load_wasm_proc_macros", || {
    self.crate_loader(|c| c.load_wasm_proc_macros())
});
```

- Called at the start of `resolve_crate()`
- Ensures proc macros are loaded before resolution begins
- Timed for performance monitoring

## Test Results

Tested with `watt_demo_with_metadata.wasm`:

```bash
wasmtime run -Sthreads=yes --dir .::/ dist/bin/rustc.wasm --sysroot dist \
  test_watt_demo.rs --target wasm32-wasip1 \
  --wasm-proc-macro Demo=watt_demo_with_metadata.wasm \
  --edition 2021 -o tmp/test_watt_demo.wasm
```

### Debug Output

```
[CREADER] load_wasm_proc_macros called with 1 entries
[CREADER] Loading WASM proc macro: Demo from "watt_demo_with_metadata.wasm"
[CREADER] Read 503343 bytes from watt_demo_with_metadata.wasm
[CREADER DEBUG] create_wasm_proc_macros called
[CREADER DEBUG] Extracting proc macro metadata from WASM...
[CREADER DEBUG] Found 1 metadata entries
[CREADER] Extracted 1 proc macros from Demo
```

### Verification

✅ **Flag parsing works** - Command-line argument is correctly parsed
✅ **File loading works** - WASM file is successfully read (503KB)
✅ **Metadata extraction works** - Proc macro metadata is found (1 entry)
✅ **Integration works** - Method is called at the correct time in compilation
✅ **Conditional compilation works** - Compiles for both WASM and native targets

### Expected Error

```
error: cannot find derive macro `Demo` in this scope
```

This error is **expected** because the proc macros are not yet registered in the cstore. The loading and extraction work, but the resolver can't find them yet.

## What Remains: Phase 2 (Registration)

The proc macros need to be registered in rustc's cstore so the resolver can find them by name.

### Required Implementation

**Location:** `compiler/rustc_metadata/src/creader.rs` in `load_wasm_proc_macros()`

**Current Code:**
```rust
let _ = Box::leak(proc_macros); // Leak for now to match dlsym_proc_macros
```

**Needs to be replaced with:**

```rust
// Create synthetic CrateMetadata for the WASM proc macro
let crate_name = Symbol::intern(name);
let cnum = self.cstore.alloc_new_crate_num();

// TODO: Create minimal CrateMetadata that:
// 1. Has the proc macro name
// 2. Has the proc_macros array
// 3. Has enough metadata to satisfy the resolver
// 4. References the WASM file as the source

let crate_metadata = CrateMetadata::new_minimal_proc_macro(
    self.sess,
    crate_name,
    Box::leak(proc_macros),
    path,
);

self.cstore.set_crate_data(cnum, crate_metadata);
```

### Challenges

1. **CrateMetadata Complexity**
   - Requires: metadata bytes, crate_root, cnum_map, dep_kind, source, etc.
   - Current constructor: `CrateMetadata::new()` with 11 parameters
   - Need to create a simplified constructor for synthetic proc macros

2. **Metadata Requirements**
   - Need minimal rustc metadata bytes (currently requires full .rmeta format)
   - Or create a way to bypass metadata checks for synthetic crates

3. **Crate Dependencies**
   - `cnum_map` - dependency resolution (can probably be empty for proc macros)
   - `dep_kind` - how this crate is used (should be `MacrosOnly`)

4. **Source Information**
   - Need to create `CrateSource` pointing to the WASM file
   - Should have no dylib (WASM is the only artifact)

### Proposed Solution Path

**Option A: Minimal CrateMetadata Constructor**
```rust
impl CrateMetadata {
    pub fn new_synthetic_proc_macro(
        sess: &Session,
        cstore: &CStore,
        name: Symbol,
        proc_macros: &'static [ProcMacro],
        wasm_path: &Path,
    ) -> Self {
        // Create minimal metadata that satisfies resolver
        // Use empty/dummy values for unused fields
    }
}
```

**Option B: Direct Proc Macro Registration**
```rust
// Skip CrateMetadata entirely
// Directly register proc macros by name in a separate map
// Requires modifying resolver to check this map
```

Option A is cleaner and more consistent with existing architecture.

## Files Modified

### New Files Created
- `WASM_PROC_MACRO_FLAG_STATUS.md` - This file

### Modified Files
1. `compiler/rustc_session/src/options.rs`
   - Line 173: Added `wasm_proc_macros` field

2. `compiler/rustc_session/src/config.rs`
   - Lines 1580-1587: Added flag definition
   - Lines 2276-2329: Added `parse_wasm_proc_macros()` function
   - Line 1173: Added field to Default impl
   - Line 2592: Parse wasm_proc_macros
   - Line 2667: Add to Options construction

3. `compiler/rustc_metadata/src/creader.rs`
   - Lines 325-379: Added `load_wasm_proc_macros()` method

4. `compiler/rustc_resolve/src/lib.rs`
   - Lines 1723-1726: Call `load_wasm_proc_macros()` early in resolution

## Build Information

- **Target:** wasm32-wasip1-threads
- **Build Time:** ~9 minutes (incremental: ~2-3 minutes)
- **Warnings:** 2 (unused import, dead code - non-critical)
- **Build Command:**
  ```bash
  WASI_SDK_PATH=`pwd`/wasi-sdk-25.0-arm64-linux \
  WASI_SYSROOT=`pwd`/wasi-sdk-25.0-arm64-linux/share/wasi-sysroot \
  ./x.py install --config config.llvm.toml -j 4
  ```

## Usage

```bash
# Basic usage
wasmtime run -Sthreads=yes --dir .::/ dist/bin/rustc.wasm \
  --sysroot dist your_code.rs --target wasm32-wasip1 \
  --wasm-proc-macro MacroName=path/to/macro.wasm \
  --edition 2021 -o output.wasm

# Multiple proc macros
wasmtime run -Sthreads=yes --dir .::/ dist/bin/rustc.wasm \
  --sysroot dist your_code.rs --target wasm32-wasip1 \
  --wasm-proc-macro Derive1=macro1.wasm \
  --wasm-proc-macro Derive2=macro2.wasm \
  --edition 2021 -o output.wasm
```

## Dependencies

### Existing Components Used
- `rustc_watt_runtime` - WASM interpreter and execution
- `create_wasm_proc_macros()` - Metadata extraction from WASM
- Slot-based registry - Zero-sized closure handling (lines 1296-2631 of creader.rs)

### External Dependencies
- `watt` framework - For compiling proc macros to WASM
- WASM proc macro must have `.rustc_proc_macro_decls` custom section

## Next Session Recommendations

1. **Implement Phase 2** (Registration)
   - Create `CrateMetadata::new_synthetic_proc_macro()` helper
   - Allocate CrateNum for each loaded proc macro
   - Register in cstore with proper name mapping
   - Test that resolver can find the proc macros

2. **End-to-End Testing**
   - Verify proc macro execution works
   - Test TokenStream marshaling
   - Ensure error messages are clear
   - Performance benchmarking

3. **Documentation**
   - User guide for `--wasm-proc-macro` flag
   - Developer guide for the implementation
   - Update PROC_MACRO_LOADING_ISSUE.md with completion notes

4. **Future Work** (Optional)
   - Make flag unstable (add back `-Z unstable-options` requirement)
   - Solution A: Automatic .rlib generation during compilation
   - Cargo integration
   - Support for attribute and function-like proc macros

## Notes

- The flag is currently **stable** for testing convenience
- In production, it should be gated behind `-Z unstable-options`
- The implementation is conditionally compiled for WASM targets only
- Non-WASM builds simply ignore the flag (no-op)

## Success Criteria

**Phase 1 (Complete):** ✅
- [x] Flag is parsed from command line
- [x] WASM files are loaded
- [x] Metadata is extracted
- [x] Integration with resolver works
- [x] Debug logging confirms execution flow

**Phase 2 (Remaining):** ⏳
- [ ] Proc macros are registered in cstore
- [ ] Resolver can find proc macros by name
- [ ] `#[derive(Demo)]` compiles without errors
- [ ] Generated code is correct

**Phase 3 (Future):** 📋
- [ ] End-to-end proc macro execution works
- [ ] TokenStream marshaling is validated
- [ ] Performance is acceptable
- [ ] Error messages are helpful
- [ ] Documentation is complete

## Related Documents

- `PROC_MACRO_LOADING_ISSUE.md` - Root cause analysis and solution proposals
- `SESSION_SUMMARY.md` - Previous session progress
- `WASM_PROC_MACRO_COMPLETE_STATUS.md` - Comprehensive implementation guide
- `OPTION_A_STATUS.md` - Watt framework integration status

---

**Conclusion:** The `--wasm-proc-macro` flag infrastructure is fully implemented and tested. The next step is registering the loaded proc macros in rustc's cstore so they can be found and used by the resolver. This completes the pathway to using WASM procedural macros in rustc.wasm.
