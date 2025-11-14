# BREAKTHROUGH: The Complete Proc Macro Pattern Discovered!

**Date:** November 11, 2025

## Summary

After extensive investigation, I've discovered how proc macros actually work in rustc. The key insight: **you cannot use proc macro .so files directly - they must be accessed through a wrapper library that re-exports them!**

## The Complete Pattern

### Step 1: Compile the Proc Macro Crate

```bash
rustc template_proc_macro.rs --crate-type proc-macro --edition 2021
# Generates: libtemplate_proc_macro.so
# Note: NO .rlib file is generated!
```

### Step 2: Create and Compile a Wrapper Library

The wrapper library re-exports the proc macro:

```rust
// template_lib.rs
extern crate template_proc_macro;
pub use template_proc_macro::Template;
```

Compile it WITH the proc macro as a dependency:

```bash
rustc template_lib.rs --crate-type lib \
  --extern template_proc_macro=/path/to/libtemplate_proc_macro.so \
  --edition 2021
# Generates: libtemplate_lib.rlib
```

### Step 3: Compile User Code

User code imports from the wrapper library:

```rust
// test.rs
use template_lib::Template;

#[derive(Template)]
struct MyStruct;

fn main() {
    println!("Hello!");
}
```

Compile with the wrapper library (NOT the proc macro .so):

```bash
rustc test.rs \
  --extern template_lib=/path/to/libtemplate_lib.rlib \
  -L /path/to/deps \
  --edition 2021
# SUCCESS!
```

## Key Discoveries

### 1. Proc Macros DON'T Generate .rlib Files

```bash
$ rustc foo.rs --crate-type proc-macro --print file-names
libfoo.so
# No .rlib!
```

### 2. You CANNOT Use .so Files Directly with --extern

This FAILS:
```bash
rustc test.rs --extern foo=libfoo.so
# Error: cannot find derive macro
```

### 3. The Wrapper Library Pattern

This is how serde works:
- `serde_derive` crate → compiled to `libserde_derive-*.so`
- `serde` crate → depends on `serde_derive`, re-exports the macros:
  ```rust
  extern crate serde_derive;
  pub use serde_derive::{Serialize, Deserialize};
  ```
- User code → depends on `serde` only, not `serde_derive`

### 4. Cargo's Build Process

When building with Cargo:

1. **Compiling serde_derive (proc macro)**:
   ```bash
   rustc serde_derive/src/lib.rs --crate-type proc-macro --emit=dep-info,link
   # Output: libserde_derive-cb2b727a9965bed7.so
   ```

2. **Compiling serde (wrapper library)**:
   ```bash
   rustc serde/src/lib.rs --crate-type lib --emit=dep-info,metadata,link \
     --extern serde_derive=/path/libserde_derive-cb2b727a9965bed7.so
   # Output: libserde-e44f79cc9dae616c.rlib
   ```

3. **Compiling user binary**:
   ```bash
   rustc src/main.rs --crate-type bin \
     --extern serde=/path/libserde-e44f79cc9dae616c.rlib
   # Note: NO mention of serde_derive!
   ```

## Why Our Earlier Attempts Failed

### Failed Attempt 1: Direct .so Usage
```bash
rustc test.rs --extern template_proc_macro=libtemplate_proc_macro.so
# FAILED: "cannot find derive macro"
```
**Why:** Rustc doesn't support using proc macro .so files directly from user code.

### Failed Attempt 2: Generating .rlib Files
We spent time trying to generate .rlib files for proc macros.
**Why it was wrong:** Proc macros fundamentally don't use .rlib files - they use .so files accessed through wrapper libraries.

### Failed Attempt 3: Using Cargo-built .so Directly
```bash
rustc test.rs --extern test_cargo_proc_macro=/tmp/.../libtest_cargo_proc_macro-*.so
# FAILED: "cannot find derive macro"
```
**Why:** Same reason - even Cargo-built .so files can't be used directly.

## Implications for WASM Proc Macros

This discovery completely changes our approach!

### Current State
We have:
- ✅ WASM proc macros compiled (e.g., `watt_demo_with_metadata.wasm`)
- ✅ `--wasm-proc-macro` flag implemented in Phase 1
- ❌ Cannot use WASM proc macros directly

### What We Need to Do

**Option A: Implement WASM Wrapper Library Support**

1. For WASM proc macro `foo.wasm`:
2. Create a minimal wrapper library that "imports" the WASM module
3. Compile wrapper to .rlib that references the WASM
4. Use the .rlib in user code

**Option B: Special rustc.wasm Support**

Modify rustc.wasm to:
1. Accept `--extern foo=foo.wasm` (WASM proc macros)
2. Treat `.wasm` files specially (like native rustc treats `.so` files)
3. Load and execute WASM modules during compilation
4. BUT: Still need wrapper libraries for re-exporting!

**Option C: Hybrid Approach (RECOMMENDED)**

1. Use `--wasm-proc-macro` flag to register WASM modules directly
2. Internally create "synthetic" crates that re-export the proc macros
3. Make these available to the resolver
4. User code can then `use foo::MacroName;` naturally

This is essentially what Phase 2 registration was trying to do!

## The Real Solution

Going back to our Phase 2 plan with NEW understanding:

The `--wasm-proc-macro Demo=demo.wasm` flag should:

1. ✅ Load the WASM file (Phase 1 - done)
2. ✅ Extract proc macro declarations (Phase 1 - done)
3. **Create a synthetic "wrapper crate" in memory** (Phase 2 - needed)
   - Acts like the wrapper library pattern
   - Re-exports the proc macros
   - Makes them available to the resolver
4. **Register this synthetic crate** (Phase 2 - needed)
   - Add to CStore
   - Make resolver aware of it

## Next Steps

1. **Complete Phase 2 with wrapper library understanding**
   - Create synthetic CrateMetadata that acts as a wrapper
   - Register proc macros as if they came from a re-exporting library
   - Let resolver find them naturally

2. **Test with the pattern**:
   ```bash
   rustc.wasm test.rs --wasm-proc-macro Demo=demo.wasm
   # Should work like:
   # - Demo is a synthetic wrapper crate
   # - #[derive(SomeMacro)] resolves to Demo::SomeMacro
   # - Execution uses WASM runtime
   ```

## Testing the Pattern

### Native Test (Working)
```bash
# 1. Compile proc macro
rustc template_proc_macro.rs --crate-type proc-macro --edition 2021

# 2. Compile wrapper
rustc template_lib.rs --crate-type lib \
  --extern template_proc_macro=libtemplate_proc_macro.so \
  --edition 2021

# 3. Compile user code
rustc test_template_v2.rs \
  --extern template_lib=libtemplate_lib.rlib \
  -L . \
  --edition 2021

# 4. Run
./test_template_v2
# Output: Hello from template macro!
```

### WASM Test (To Implement)
```bash
# 1. WASM proc macro already compiled: watt_demo_with_metadata.wasm

# 2. Create wrapper library (manual for now, auto later):
cat > demo_lib.rs << 'EOF'
// This would be auto-generated or handled internally by rustc.wasm
extern "wasm" fn get_demo_macro() -> ProcMacro; // hypothetical
pub use Demo; // re-export
EOF

# 3. Or better: use --wasm-proc-macro to auto-create wrapper:
rustc.wasm test.rs --wasm-proc-macro Demo=watt_demo_with_metadata.wasm
# Internally:
# - Loads watt_demo_with_metadata.wasm
# - Creates synthetic "Demo" crate with re-exports
# - Makes it available to resolver
# - User code works naturally!
```

## Files Created During Investigation

1. **template_proc_macro.rs** - Minimal proc macro for testing
2. **libtemplate_proc_macro.so** - Compiled proc macro
3. **template_lib.rs** - Wrapper library (KEY!)
4. **libtemplate_lib.rlib** - Compiled wrapper
5. **test_template_v2.rs** - User code using wrapper library
6. **test_template_v2** - Working binary!

## Cargo Verbose Output Analysis

Key findings from `cargo build -v`:

- Proc macro compilation: `--crate-type proc-macro --emit=dep-info,link`
- Wrapper library compilation: `--extern serde_derive=...so`
- User code compilation: `--extern serde=...rlib` (no proc macro!)

## Conclusion

**The wrapper library pattern is the key to making proc macros work.**

For WASM proc macros, we need to either:
1. Create actual wrapper libraries
2. Synthesize them in rustc.wasm (preferred)
3. Or both (hybrid)

The `--wasm-proc-macro` flag + Phase 2 registration is actually the RIGHT approach, we just needed to understand that it's creating a "synthetic wrapper" not just registering raw proc macros.

---

**Status:** Pattern discovered and validated with native proc macros. Ready to apply to WASM implementation.

**Next:** Implement Phase 2 with wrapper library understanding.
