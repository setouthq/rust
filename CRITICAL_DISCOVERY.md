# CRITICAL DISCOVERY: Proc Macros Don't Use .rlib Files!

**Date:** November 10, 2025

## The False Assumption

We spent significant time investigating how to generate .rlib files for WASM proc macros based on the assumption that:
- Native proc macros have both .so and .rlib files
- The .rlib contains metadata pointing to the .so
- WASM proc macros fail because they lack .rlib files

**THIS ASSUMPTION WAS WRONG.**

## What We Actually Discovered

### Test 1: Native Rustc with Proc Macros

```bash
# Compile proc macro
rustc template_proc_macro.rs --crate-type proc-macro --print file-names
# Output: libtemplate_proc_macro.so
# NOTE: NO .rlib file is generated!
```

### Test 2: Cargo with Proc Macros

```bash
# Build proc macro with Cargo
cargo build --release

# Check generated files
ls target/release/deps/
# Output: libtest_cargo_proc_macro-*.so
# NOTE: NO .rlib file is generated!
```

### Test 3: Using .so Directly

```bash
# Try using the .so directly
rustc test.rs --extern foo=libfoo.so
# Result: FAILS - "cannot find derive macro"
```

## The Real Truth About Proc Macros

**Proc macros DO NOT use .rlib files at all!**

When you compile with `--crate-type proc-macro`:
- ✅ Generates dylib (.so, .dll, or .dylib)
- ❌ Does NOT generate .rlib
- ❌ Does NOT generate standalone .rmeta

The .rlib approach was a red herring.

## How Do Native Proc Macros Actually Work?

Based on the tests, there must be a different mechanism. Cargo must:
1. Build the proc macro → generates .so
2. Store metadata about the proc macro (NOT in .rlib)
3. Pass this information to rustc in a special way

Let me investigate how Cargo actually passes proc macros to rustc...

##The Real Problem

When we use:
```bash
rustc code.rs --extern Demo=watt_demo.wasm
```

Rustc tries to:
1. Load metadata from watt_demo.wasm
2. Discovers it's not a valid crate file format
3. Fails

When Cargo uses proc macros, it must have a different mechanism that:
- Doesn't rely on .rlib files
- Can locate and load the dylib
- Can discover the proc macro declarations

## What This Means For Our Approach

Our "generate .rlib files" approach was solving the WRONG problem.

The real issue is that:
1. Rustc's `--extern` expects either:
   - An .rlib file
   - A directory it can search
   - Some other metadata source

2. It does NOT accept raw dylib files (.so or .wasm)

3. Cargo must be doing something special to make proc macros work

## Next Steps: Find The Real Solution

We need to investigate:

1. **How does Cargo actually invoke rustc for proc macros?**
   ```bash
   cargo build -v  # verbose output shows rustc commands
   ```

2. **Does Cargo use a special flag or mechanism?**
   - Maybe `--extern-location`?
   - Maybe fingerprint files?
   - Maybe the dep-info (.d) files?

3. **How does rustc actually discover proc macros?**
   - Is there a proc macro registry?
   - Does it scan directories?
   - Does it use .d files?

## Hypothesis: The Real Solution

Based on the evidence, I hypothesize that:

**Option A:** Cargo generates metadata files we haven't found yet
- Maybe .rmeta files in deps/?
- Maybe fingerprint metadata?

**Option B:** Rustc has a special proc-macro loading path
- Cargo might pass proc macros differently than normal crates
- There might be a special flag like `--extern-proc-macro`

**Option C:** The `--wasm-proc-macro` flag was the RIGHT approach all along
- We just need to complete Phase 2 (registration)
- The flag approach bypasses the normal --extern mechanism
- This might be simpler than we thought

## Recommendation: Pivot Strategy

**STOP** trying to generate .rlib files - that's not how proc macros work.

**START** investigating one of:

1. **How Cargo actually does it** (most informative)
   - Run `cargo build -v` with proc macros
   - Examine exact rustc invocation
   - Replicate that mechanism

2. **Complete the `--wasm-proc-macro` implementation** (most direct)
   - We already have Phase 1 working
   - Phase 2 just needs registration
   - This might be simpler than we thought

3. **Find if rustc has proc-macro specific flags** (fastest if exists)
   - Check `rustc --help` for proc-macro flags
   - Look at rustc source for special proc-macro handling

## The Silver Lining

Our investigation wasn't wasted - we now understand:
- ✅ Proc macros don't use .rlib files
- ✅ There's a special mechanism for proc macros
- ✅ Metadata format and structure
- ✅ How to use ar archives

This knowledge will be valuable for finding the real solution.

---

**Status:** Major discovery made. Entire approach needs revision.

**Next Action:** Investigate how Cargo actually passes proc macros to rustc, OR complete Phase 2 of `--wasm-proc-macro` flag.
