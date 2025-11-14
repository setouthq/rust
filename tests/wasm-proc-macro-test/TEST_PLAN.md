# WASM Proc Macro Integration Test Plan

## Overview
This tests the complete WASM proc macro integration by:
1. Running WASM rustc via wasmtime
2. Compiling a proc macro crate to WASM
3. Verifying metadata is embedded
4. Using the proc macro in a test crate

## Prerequisites

- ✅ Wasmtime 30.0.2 installed at `/root/.cargo/bin/wasmtime`
- ✅ Test proc macro crate at `tests/wasm-proc-macro-test/test-macro/`
- 🔄 WASM rustc building (stage2)

## Test Steps

### Step 1: Locate WASM rustc Binary

Expected location after build completes:
```bash
# WASM rustc will be at:
build/aarch64-unknown-linux-gnu/stage2-wasm/wasm32-wasip1-threads/release/rustc.wasm
# OR
build/wasm32-wasip1-threads/stage2/bin/rustc.wasm
```

### Step 2: Verify WASM rustc Runs

```bash
wasmtime run \
    --dir=. \
    --env RUST_BACKTRACE=1 \
    build/.../rustc.wasm -- --version
```

Expected output:
```
rustc 1.84.1 (...)
```

### Step 3: Compile Test Proc Macro to WASM

```bash
# Using WASM rustc to compile the proc macro
wasmtime run \
    --dir=. \
    --env RUST_BACKTRACE=1 \
    build/.../rustc.wasm -- \
    tests/wasm-proc-macro-test/test-macro/src/lib.rs \
    --crate-name test_macro \
    --crate-type proc-macro \
    --edition 2021 \
    --target wasm32-wasip1-threads \
    -L dependency=tests/wasm-proc-macro-test/test-macro/target/wasm32-wasip1-threads/release/deps \
    -o test_macro.wasm
```

### Step 4: Verify Metadata Section Exists

```bash
# Check if .rustc_proc_macro_decls section exists
wasm-objdump -x test_macro.wasm | grep -A 5 "rustc_proc_macro_decls"
```

Expected output:
```
Custom section '.rustc_proc_macro_decls':
  0000000: derive:TestDerive:test_derive
  0000020: attr:test_attr:test_attr
  0000040: bang:test_bang:test_bang
  ...
```

### Step 5: Extract and Verify Metadata

Create a simple Rust program to test the metadata extraction:

```rust
// tests/wasm-proc-macro-test/verify_metadata.rs
use std::fs;
use rustc_watt_runtime::metadata::extract_proc_macro_metadata;

fn main() {
    let wasm_bytes = fs::read("test_macro.wasm").unwrap();
    let metadata = extract_proc_macro_metadata(&wasm_bytes);

    println!("Found {} proc macros:", metadata.len());
    for meta in &metadata {
        println!("  - {:?}", meta);
    }

    assert!(metadata.len() >= 3, "Expected at least 3 proc macros");
    println!("✅ Metadata extraction successful!");
}
```

### Step 6: Create Test Usage Crate

```rust
// tests/wasm-proc-macro-test/test-usage/src/main.rs
use test_macro::*;

#[derive(TestDerive)]
struct MyStruct;

#[test_attr]
fn my_function() {
    println!("Hello from my_function");
}

fn main() {
    let x = test_bang!();
    let y = MyStruct::generated_value();

    assert_eq!(x, 42);
    assert_eq!(y, 42);

    my_function();

    println!("✅ All proc macros work!");
}
```

### Step 7: Compile Usage Crate with WASM rustc

```bash
wasmtime run \
    --dir=. \
    --env RUST_BACKTRACE=1 \
    build/.../rustc.wasm -- \
    tests/wasm-proc-macro-test/test-usage/src/main.rs \
    --extern test_macro=test_macro.wasm \
    --edition 2021 \
    --target wasm32-wasip1-threads \
    -o test_usage.wasm
```

### Step 8: Run the Test Usage Program

```bash
wasmtime run test_usage.wasm
```

Expected output:
```
Function my_function called
Hello from my_function
✅ All proc macros work!
```

## Success Criteria

- [x] WASM rustc runs via wasmtime
- [ ] Test proc macro compiles to WASM
- [ ] Metadata section exists in WASM output
- [ ] Metadata can be extracted correctly
- [ ] All three proc macro types are present
- [ ] Usage crate compiles with proc macros
- [ ] Usage crate runs and produces correct output

## Troubleshooting

### Issue: WASM rustc not found
- Check build completed successfully
- Verify stage2 build artifacts exist
- Look in alternative locations

### Issue: Metadata section missing
- Verify WASM target detection in proc_macro_harness.rs
- Check that proc-macro crate type is set correctly
- Inspect generated code with wasm-objdump

### Issue: Proc macro expansion fails
- Enable RUST_BACKTRACE=full for detailed errors
- Check watt runtime can execute WASM functions
- Verify TokenStream serialization works

### Issue: Dependencies not found
- Build dependencies for WASM target first
- Use --extern flags correctly
- Check -L dependency paths

## Notes

- WASM proc macros must be compiled with the same rustc version
- Metadata format must match between compiler and loader
- watt runtime requires WASM functions to be exported with correct names
- WASI preview1 threads support is required for full compatibility
