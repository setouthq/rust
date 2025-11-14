# WASM Proc Macro Implementation - Comprehensive Summary

**Date:** November 10, 2025

## Executive Summary

We successfully implemented Phase 1 of WASM proc macro support (the `--wasm-proc-macro` flag infrastructure), but discovered that Phase 2 (registration) requires a different approach than initially planned. After extensive investigation, we've identified a simpler, more practical solution.

## What We've Accomplished

### ✅ Phase 1: Complete and Working

**Implemented:**
1. **Command-line flag** (`--wasm-proc-macro NAME=PATH`)
   - Flag registration in rustc
   - Argument parsing and validation
   - Storage in Options struct

2. **WASM file loading**
   - Reads WASM files successfully (tested with 503KB file)
   - Extracts proc macro metadata from `.rustc_proc_macro_decls` section
   - Creates `ProcMacro` structures

3. **Integration with resolver**
   - Called early in compilation (before resolution)
   - Proper timing ensures availability

**Test Results:**
```
[CREADER] load_wasm_proc_macros called with 1 entries
[CREADER] Loading WASM proc macro: Demo from "watt_demo_with_metadata.wasm"
[CREADER] Read 503343 bytes from watt_demo_with_metadata.wasm
[CREADER] Extracted 1 proc macros from Demo
```

**Files Modified:**
- `compiler/rustc_session/src/options.rs` - Added wasm_proc_macros field
- `compiler/rustc_session/src/config.rs` - Flag definition and parsing
- `compiler/rustc_metadata/src/creader.rs` - Loading implementation
- `compiler/rustc_resolve/src/lib.rs` - Resolver integration

### ⏸️ Phase 2: Registration (Blocked)

**The Challenge:**
Registering loaded proc macros requires creating synthetic `CrateMetadata`, which is complex because:
- Requires `MetadataBlob` in rustc's binary format
- Requires `CrateRoot` with 30+ fields including LazyArray encodings
- Requires `DefPathHashMap` and other interconnected structures

**Three Options Investigated:**
- **Option A:** Full synthetic metadata (1-2 days, most robust)
- **Option B:** Bypass CrateMetadata (4-6 hours, simpler but less clean)
- **Option C:** Generate .rmeta files (1 day, uses existing infrastructure)

**Conclusion:** All options are substantial undertakings for indirect approaches.

## The Root Cause (Discovered)

The fundamental issue is that rustc expects TWO files for proc macros:

1. **Metadata file** (.rlib or .rmeta)
   - Tells rustc this is a proc macro crate
   - Contains proc macro declarations
   - Points to the dylib location

2. **Dylib file** (.so, .dll, or .wasm)
   - Contains actual implementation code
   - Has export symbols

**Current Situation:**
- Native proc macros: Have BOTH files ✅
- WASM proc macros: Only have .wasm file ❌

**Why native works:**
```
libfoo.so    ← Dylib with code
libfoo.rlib  ← Metadata pointing to libfoo.so
```

**Why WASM doesn't:**
```
foo.wasm     ← WASM module with code
(no .rlib)   ← Missing!
```

## Recommended Solution

### 🎯 Post-Build Tool Approach (Option 2C - Template Method)

**Create a standalone tool that generates .rlib files for WASM proc macros.**

**Why This Approach:**
1. **Simpler than modifying rustc** - No complex compiler changes
2. **Practical and usable today** - Can be built in 4-6 hours
3. **Works with existing workflows** - Drop-in tool
4. **Leverages existing infrastructure** - Uses template metadata

**How It Works:**
```bash
# Step 1: Compile proc macro to WASM (already works)
rustc simple_test.rs --crate-type proc-macro --target wasm32-wasip1-threads \
  -o simple_test.wasm

# Step 2: Generate .rlib (new tool)
wasm-proc-macro-rlib simple_test.wasm -o libsimple_test.rlib

# Step 3: Use normally
rustc user_code.rs --extern simple_test=libsimple_test.rlib
```

**Implementation Strategy:**
1. Build a minimal template proc macro with native rustc
2. Extract and study its .rlib metadata format
3. Create tool that:
   - Reads WASM file
   - Extracts proc macro declarations
   - Modifies template metadata to reference WASM file
   - Outputs valid .rlib archive

**Estimated Time:** 4-6 hours for working prototype

## Implementation Plan for Recommended Solution

### Phase 1: Create Template and Extract Metadata (1 hour)

```bash
# Create minimal template proc macro
cat > template.rs << 'EOF'
extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro_derive(Template)]
pub fn template_derive(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
EOF

# Compile with native rustc to get reference .rlib
rustc template.rs --crate-type proc-macro --edition 2021

# Extract .rlib contents
mkdir template_rlib
cd template_rlib
ar x ../libtemplate.rlib

# Study the metadata
ls -lah
# Look for: rust.metadata.bin (the key file)

# Examine metadata structure
xxd rust.metadata.bin | head -50
```

### Phase 2: Build Metadata Modification Tool (3-4 hours)

```rust
// Tool: wasm-proc-macro-rlib
// File: src/main.rs

use std::fs;
use std::path::Path;
use ar::{Builder, Header};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wasm_path = std::env::args().nth(1)
        .expect("Usage: wasm-proc-macro-rlib <input.wasm> [-o output.rlib]");

    let output_path = std::env::args()
        .position(|a| a == "-o")
        .and_then(|i| std::env::args().nth(i + 1))
        .unwrap_or_else(|| {
            let stem = Path::new(&wasm_path).file_stem().unwrap();
            format!("lib{}.rlib", stem.to_str().unwrap())
        });

    println!("Generating .rlib for {}", wasm_path);

    // 1. Extract proc macro info from WASM
    let wasm_bytes = fs::read(&wasm_path)?;
    let proc_macros = extract_proc_macro_info(&wasm_bytes)?;

    println!("Found {} proc macros", proc_macros.len());
    for pm in &proc_macros {
        println!("  - {} ({})", pm.name, pm.kind);
    }

    // 2. Load template metadata
    let template_metadata = include_bytes!("../template_metadata.bin");

    // 3. Modify metadata to reference WASM file and update proc macro list
    let modified_metadata = modify_metadata(
        template_metadata,
        &wasm_path,
        &proc_macros,
    )?;

    // 4. Create .rlib archive
    create_rlib(&output_path, &modified_metadata)?;

    println!("Successfully generated: {}", output_path);
    Ok(())
}

struct ProcMacroInfo {
    name: String,
    kind: ProcMacroKind,
}

enum ProcMacroKind {
    Derive,
    Attribute,
    FunctionLike,
}

fn extract_proc_macro_info(wasm_bytes: &[u8]) -> Result<Vec<ProcMacroInfo>, Box<dyn std::error::Error>> {
    // Parse WASM module
    // Find .rustc_proc_macro_decls custom section
    // Decode the section data
    // Return list of proc macros

    // This code already exists in rustc_watt_runtime
    // We can either:
    //   A) Extract it to a separate library
    //   B) Re-implement the parsing
    //   C) Shell out to a helper that uses rustc_watt_runtime

    todo!("Extract from WASM")
}

fn modify_metadata(
    template: &[u8],
    wasm_path: &str,
    proc_macros: &[ProcMacroInfo],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // This is the tricky part
    // Need to understand rustc metadata format

    // Approach:
    // 1. Parse metadata to find:
    //    - Crate name field → replace with our name
    //    - Dylib path field → replace with wasm_path
    //    - Proc macro count → update to match
    //    - Proc macro names → replace with our names
    // 2. Reserialize with modifications

    // For MVP, we might:
    // - Use binary search/replace for known patterns
    // - Or use rustc_metadata crate to parse/modify properly

    todo!("Modify metadata")
}

fn create_rlib(
    output_path: &str,
    metadata: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::create(output_path)?;
    let mut builder = Builder::new(file);

    // Add metadata with correct name
    let mut header = Header::new(b"rust.metadata.bin".to_vec(), metadata.len() as u64);
    builder.append(&header, metadata)?;

    // Finalize archive
    drop(builder);
    Ok(())
}
```

### Phase 3: Testing (1 hour)

```bash
# Test with watt_demo_with_metadata.wasm
cargo run -- watt_demo_with_metadata.wasm -o libDemo.rlib

# Try using it
wasmtime run -Sthreads=yes --dir .::/ dist/bin/rustc.wasm \
  --sysroot dist test_watt_demo.rs --target wasm32-wasip1 \
  --extern Demo=libDemo.rlib \
  --edition 2021 -o tmp/test.wasm

# If successful: proc macro is found and used!
# If not: debug and iterate
```

## Alternative: Check If Cargo Already Generates .rlib

Before building the tool, we should verify whether Cargo already generates .rlib files for WASM proc macros:

```bash
# Test with Cargo
cargo new --lib test_proc_macro
cd test_proc_macro

# Edit Cargo.toml
cat >> Cargo.toml << 'EOF'

[lib]
proc-macro = true
EOF

# Add minimal proc macro
cat > src/lib.rs << 'EOF'
extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro_derive(Test)]
pub fn test_derive(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
EOF

# Build for WASM
cargo build --target wasm32-wasip1-threads --release

# Check what files exist
ls -lah target/wasm32-wasip1-threads/release/

# If libtest_proc_macro.rlib exists, great!
# If not, that confirms our tool is needed
```

## Files Created During Investigation

### Documentation Files
1. **`WASM_PROC_MACRO_FLAG_STATUS.md`** - Phase 1 completion status
2. **`PHASE_2_REGISTRATION_PLAN.md`** - Detailed analysis of registration options
3. **`PHASE_2_REGISTRATION_STATUS.md`** - Investigation findings and blockers
4. **`SOLUTION_A_INVESTIGATION.md`** - Solution A analysis and tool approach
5. **`WASM_PROC_MACRO_SUMMARY.md`** - This file (comprehensive summary)

### Code Files Modified
1. `compiler/rustc_session/src/options.rs` - Options field
2. `compiler/rustc_session/src/config.rs` - Flag and parsing
3. `compiler/rustc_metadata/src/creader.rs` - Loading logic
4. `compiler/rustc_resolve/src/lib.rs` - Integration

## Current State

### What Works
✅ Command-line flag parsing
✅ WASM file loading
✅ Proc macro metadata extraction
✅ Integration with compilation flow

### What Doesn't Work Yet
❌ Proc macros not registered in CStore
❌ Resolver can't find proc macros by name
❌ `#[derive(Demo)]` errors with "cannot find derive macro"

### Why
The `--wasm-proc-macro` flag successfully loads proc macros but they're not registered in rustc's internal crate storage system, so the resolver can't find them.

## Recommended Next Steps

1. **Verify Cargo behavior** (15 minutes)
   - Check if Cargo generates .rlib for WASM proc macros
   - If yes, investigate why it's not working
   - If no, proceed to step 2

2. **Build template proc macro** (30 minutes)
   - Create minimal template
   - Compile with native rustc
   - Extract and study metadata format

3. **Create `wasm-proc-macro-rlib` tool** (3-4 hours)
   - Implement metadata extraction from WASM
   - Implement metadata modification
   - Implement .rlib generation

4. **Test and iterate** (1-2 hours)
   - Test with watt_demo_with_metadata.wasm
   - Debug any issues
   - Refine tool based on findings

## Success Criteria

When complete:
- ✅ Can generate .rlib from WASM proc macro
- ✅ `--extern Demo=libDemo.rlib` works
- ✅ `#[derive(Demo)]` does not error
- ✅ Proc macro expansion occurs correctly
- ✅ Code generation works

## Long-Term Considerations

### For Production Use
- Integrate tool into build systems (Cargo plugin?)
- Consider upstreaming into rustc itself
- Handle edge cases (multiple proc macros, attributes, function-like)

### For Rustc Integration
- Modify rustc to auto-generate .rlib for WASM proc macros
- Update linker to create both .wasm and .rlib
- Teach Cargo about WASM proc macros

## Conclusion

We've made significant progress:
- **Phase 1 infrastructure is solid** and ready
- **Root cause is clearly understood** - missing .rlib files
- **Practical solution identified** - post-build tool
- **Implementation plan is clear** - 4-6 hours of work

The `--wasm-proc-macro` flag approach (Solution C from original plan) taught us what the real problem is: not registration complexity, but missing metadata files. The post-build tool approach solves this elegantly.

**Recommended Action:** Build the `wasm-proc-macro-rlib` tool using the template approach. This is the fastest path to working WASM proc macros.

---

**Status:** Investigation complete. Ready to implement `wasm-proc-macro-rlib` tool.

**Estimated Time to Working Solution:** 4-6 hours

**Key Insight:** Sometimes the indirect approach reveals the direct solution. By trying to register proc macros directly, we discovered they just need proper .rlib files - which we can generate with a simple tool.
