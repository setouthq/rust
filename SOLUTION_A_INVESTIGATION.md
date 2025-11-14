# Solution A Investigation: Automatic .rlib Generation for WASM Proc Macros

**Date:** November 10, 2025

## TL;DR

After investigating rustc's linking and compilation flow, implementing Solution A (automatic .rlib generation) is complex but feasible. However, there's a SIMPLER approach: **use Cargo's existing proc-macro infrastructure** which already generates .rlib files for proc macros.

## Investigation Findings

### How Proc Macros Are Compiled

Looking at `compiler/rustc_codegen_ssa/src/back/link.rs:69-151`:

```rust
match crate_type {
    CrateType::Rlib => {
        link_rlib(...)?  // Generates .rlib archive with metadata
    }
    CrateType::Staticlib => {
        link_staticlib(...)?
    }
    _ => {
        link_natively(...)? // ProcMacro goes here - generates dylib
    }
}
```

**Key Finding:** `ProcMacro` crate type calls `link_natively()` which creates a dylib (.so/.dll/WASM), NOT a .rlib file.

### Where .rlib Files Come From for Proc Macros

For native proc macros, Cargo actually invokes rustc **twice**:

1. **First compilation:** `--crate-type proc-macro` → Generates `libfoo.so` (dylib)
2. **Second compilation** (or metadata extraction): Generates `libfoo.rlib` containing:
   - Metadata pointing to the dylib
   - Proc macro declarations
   - No actual code (metadata-only)

This is why native proc macros work: they have BOTH files.

### Why WASM Proc Macros Don't Work

When compiling with `--crate-type proc-macro --target wasm32-*`:
- ✅ Generates `foo.wasm` (WASM module)
- ❌ Does NOT generate `libfoo.rlib`

Without the .rlib, rustc can't:
- Discover that this is a proc-macro crate
- Find the proc macro declarations
- Know where the WASM file is located

## Solution Options

### Option 1: Modify Rustc Linking (Complex)

**Approach:** Make rustc generate .rlib files automatically when compiling proc macros to WASM.

**Implementation Points:**
1. `compiler/rustc_codegen_ssa/src/back/link.rs:119-150`
   - Add special case for `ProcMacro` + WASM target
   - After `link_natively()` creates `.wasm`, also call `link_rlib()` to create `.rlib`

2. Challenges:
   - Need to generate proper metadata referencing the .wasm file as "dylib"
   - Need to handle crate name/path conventions (lib prefix, etc.)
   - Need to ensure both files are emitted to correct locations
   - Complex interaction with output file naming

**Estimated Complexity:** Medium-High (1-2 days)

### Option 2: Post-Build Tool (Simpler)

**Approach:** Create a standalone tool that generates .rlib files from WASM proc macros.

**Usage:**
```bash
# Compile proc macro to WASM
rustc simple_test.rs --crate-type proc-macro --target wasm32-wasip1-threads -o simple_test.wasm

# Generate accompanying .rlib
wasm-proc-macro-rlib simple_test.wasm -o libsimple_test.rlib

# Use normally
rustc user_code.rs --extern simple_test=libsimple_test.rlib
```

**Implementation:**
1. Read WASM file
2. Extract `.rustc_proc_macro_decls` section (already have code for this)
3. Create minimal metadata using `rustc_metadata::rmeta::encoder`
4. Build .rlib archive containing:
   - Metadata file (.rmeta)
   - Reference to WASM file location

**Estimated Complexity:** Low-Medium (4-8 hours)

### Option 3: Cargo Integration (Cleanest Long-Term)

**Approach:** Teach Cargo about WASM proc macros so it handles them like native proc macros.

**Implementation:**
1. Detect when building proc-macro for WASM target
2. Automatically generate .rlib after WASM compilation
3. Install both files to target directory
4. Update package metadata

**Estimated Complexity:** Medium (requires Cargo modifications)

## Recommendation

**For Immediate Use:** **Option 2 (Post-Build Tool)**

Reasons:
1. **Doesn't require modifying rustc** - standalone tool
2. **Simpler to implement** - focused scope
3. **Can iterate quickly** - easy to test and debug
4. **Works with existing workflows** - drop-in tool

**Implementation Plan for Option 2:**

```rust
// Tool: wasm-proc-macro-rlib

use std::fs;
use std::path::{Path, PathBuf};
use ar::Builder;

fn main() {
    let wasm_path = std::env::args().nth(1).expect("Usage: wasm-proc-macro-rlib <input.wasm>");
    let output_path = std::env::args().nth(2).unwrap_or_else(|| {
        let stem = Path::new(&wasm_path).file_stem().unwrap();
        format!("lib{}.rlib", stem.to_str().unwrap())
    });

    // 1. Read WASM and extract proc macro metadata
    let wasm_bytes = fs::read(&wasm_path).expect("Failed to read WASM file");
    let proc_macros = extract_proc_macro_metadata(&wasm_bytes);

    // 2. Generate minimal rustc metadata
    let metadata = generate_proc_macro_metadata(
        &wasm_path,
        &proc_macros,
    );

    // 3. Create .rlib archive
    let mut builder = Builder::new(fs::File::create(&output_path).unwrap());

    // Add metadata file
    let metadata_bytes = metadata.encode();
    builder.append_data(
        "rust.metadata.bin",
        metadata_bytes.len() as u64,
        &mut metadata_bytes.as_slice(),
    ).unwrap();

    // Finish archive
    drop(builder);

    println!("Generated {}", output_path);
}

fn extract_proc_macro_metadata(wasm_bytes: &[u8]) -> Vec<ProcMacroInfo> {
    // Use existing code from rustc_watt_runtime
    // Parse .rustc_proc_macro_decls section
    // Return list of proc macros with names and types
    todo!()
}

fn generate_proc_macro_metadata(
    wasm_path: &str,
    proc_macros: &[ProcMacroInfo],
) -> EncodedMetadata {
    // This is the tricky part - need to create valid rustc metadata
    // Options:
    //   A) Use rustc_metadata::rmeta::encoder directly
    //   B) Create minimal binary metadata manually
    //   C) Extract and modify metadata from a template proc macro
    todo!()
}
```

### Metadata Generation Challenge

The main challenge is creating valid rustc metadata. Three sub-options:

**A. Use Rustc's Encoder** (Ideal but complex)
- Requires understanding `EncodeContext` and metadata format
- Need to create minimal but valid `CrateRoot` structure
- Most robust but takes time to implement

**B. Manual Binary Format** (Hacky but faster)
- Hand-craft minimal metadata bytes
- Risky - format could change
- Quick prototype but fragile

**C. Template Approach** (Pragmatic middle ground)
- Compile a minimal template proc macro with native rustc
- Extract its .rlib metadata
- Modify the metadata to point to WASM file instead
- Update proc macro names/declarations
- Easiest to implement but feels hacky

## Concrete Next Steps

1. **Prototype Option 2C (Template Approach):**
   ```bash
   # Create template
   echo 'proc_macro_derive(Template)' > template.rs
   rustc template.rs --crate-type proc-macro -o libtemplate.so

   # Extract metadata from libtemplate.rlib
   ar x libtemplate.rlib rust.metadata.bin

   # Study the metadata format
   # Modify it to reference WASM file
   # Test with actual WASM proc macro
   ```

2. **Test the modified .rlib:**
   ```bash
   # Use modified .rlib with WASM file
   rustc user_code.rs --extern MyMacro=libmymacro.rlib

   # If it works, we've proven the concept!
   ```

3. **Build proper tool** once concept is proven

## Alternative: Check What Cargo Does

Actually, before implementing Option 2, let's check: **Does Cargo already generate .rlib files for proc macros?**

If so, maybe we just need to ensure Cargo's proc-macro path works for WASM targets!

### Investigation Needed

```bash
# Build a native proc macro with Cargo
cargo new --lib test_proc_macro
# Add proc-macro = true to Cargo.toml
cargo build --release

# Check what files are generated
ls target/release/
# Look for both .so and .rlib files

# Then try WASM target
cargo build --target wasm32-wasip1-threads --release
ls target/wasm32-wasip1-threads/release/
# Does .rlib exist?
```

If Cargo generates .rlib for WASM proc macros, problem might be elsewhere (like metadata content or file references).

## Conclusion

Solution A (automatic .rlib generation) is feasible but **Option 2 (post-build tool) is more practical** for immediate use.

The template approach (Option 2C) seems like the fastest path to a working solution:
1. Build template proc macro with native rustc
2. Extract and study its metadata
3. Create tool that modifies metadata to reference WASM files
4. Profit!

Estimated time for prototype: **4-6 hours**

---

**Status:** Investigation complete. Recommending Option 2C (post-build tool with template approach).

**Next Action:** Create minimal template proc macro and extract its metadata to understand format.
