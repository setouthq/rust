# WASM Proc Macro Implementation - Complete Guide

**Project:** Adding WASM procedural macro support to rustc.wasm
**Date:** November 11, 2025
**Status:** Phase 1 Complete, Phase 2 Ready for Implementation

## Quick Summary

This project implements support for WebAssembly procedural macros in rustc compiled to WASM. Through extensive investigation, we discovered the fundamental pattern behind how proc macros work and created a clear implementation path.

## Key Achievement: The Wrapper Library Pattern Discovery

**Breakthrough:** Proc macros cannot be used directly - they must be accessed through "wrapper libraries" that re-export them.

```
Native:  user_code.rs → serde.rlib → serde_derive.so
WASM:    user_code.rs → Demo (synthetic) → demo.wasm
```

This discovery resolved all previous confusion and provides the path forward.

## Documentation Files

### Primary Documents (Read in Order)

1. **CRITICAL_DISCOVERY.md**
   - Discovered proc macros don't generate .rlib files
   - Evidence from rustc and Cargo testing
   - Why the original approach was wrong

2. **PROC_MACRO_PATTERN_DISCOVERED.md**
   - Complete explanation of the wrapper library pattern
   - Working native example validation
   - Implications for WASM implementation

3. **FINAL_PROC_MACRO_SOLUTION.md**
   - Detailed implementation roadmap
   - Step-by-step guide with code examples
   - Timeline estimates

4. **IMPLEMENTATION_STATUS.md** ⭐ **START HERE**
   - Current status summary
   - What's done vs what's needed
   - Next steps for Phase 2
   - Multiple implementation options

### Historical Documents

- **WASM_PROC_MACRO_SUMMARY.md** - Earlier comprehensive summary
- **PHASE_2_REGISTRATION_PLAN.md** - Original Phase 2 plan
- **PHASE_2_REGISTRATION_STATUS.md** - Investigation findings
- **SOLUTION_A_INVESTIGATION.md** - .rlib generation investigation

## Current Implementation Status

### ✅ Phase 1: COMPLETE

**What works:**
```bash
rustc.wasm --wasm-proc-macro Demo=demo.wasm test.rs
```

- Flag parsing ✅
- WASM file loading ✅
- Proc macro metadata extraction ✅
- Integration with compilation flow ✅

**Code locations:**
- `compiler/rustc_session/src/options.rs:173`
- `compiler/rustc_session/src/config.rs:1580-2667`
- `compiler/rustc_metadata/src/creader.rs:325-376`
- `compiler/rustc_resolve/src/lib.rs:1723-1726`

### ⏸️ Phase 2: NEEDS IMPLEMENTATION

**What's needed:**
- Create synthetic "wrapper crate" for each WASM proc macro
- Register in CStore so resolver can find it
- Make proc macros available for expansion

**Implementation time:** 10-14 hours (full approach) or 4-6 hours (simpler approach)

## Test Files

### Working Native Example

Created and validated the wrapper library pattern:

```bash
# 1. Proc macro
rustc template_proc_macro.rs --crate-type proc-macro
# → libtemplate_proc_macro.so

# 2. Wrapper library
rustc template_lib.rs --crate-type lib \
  --extern template_proc_macro=libtemplate_proc_macro.so
# → libtemplate_lib.rlib

# 3. User code
rustc test_template_v2.rs \
  --extern template_lib=libtemplate_lib.rlib -L .
# → test_template_v2 (WORKS! ✅)

./test_template_v2
# Output: Hello from template macro!
```

**Files:**
- `template_proc_macro.rs` - Minimal test proc macro
- `template_lib.rs` - Wrapper library
- `test_template_v2.rs` - User code
- All three compile and work perfectly!

### Cargo Analysis

Created test project at `/tmp/test_proc_macro_usage/` with:
- serde dependency (uses proc macros)
- Verbose build output captured in `/tmp/cargo_verbose_build.log`
- Confirmed: Cargo passes `.so` files when compiling wrapper libraries
- Confirmed: User code never sees proc macro `.so` directly

## How to Continue

### For Implementation

1. **Read IMPLEMENTATION_STATUS.md** - Get current status
2. **Choose implementation approach:**
   - Option A: Full synthetic metadata (recommended, 10-14 hours)
   - Option B: Direct resolver integration (simpler, 4-6 hours)
   - Option C: Manual wrapper libraries (proof of concept, 1-2 hours)
3. **Follow the step-by-step guide in FINAL_PROC_MACRO_SOLUTION.md**

### For Understanding

1. **Read PROC_MACRO_PATTERN_DISCOVERED.md** - Understand the pattern
2. **Examine the working example:**
   ```bash
   cat template_proc_macro.rs
   cat template_lib.rs
   cat test_template_v2.rs
   ```
3. **Read CRITICAL_DISCOVERY.md** - Understand what we learned

## Key Insights

### What We Learned

1. **Proc macros DON'T generate .rlib files**
   - Only `.so` (or `.dll`, `.dylib`) files
   - This is by design in rustc

2. **You CANNOT use proc macro .so files directly**
   - `--extern foo=libfoo.so` doesn't work for user code
   - Rustc can't find the proc macros this way

3. **Wrapper libraries are REQUIRED**
   - They import the proc macro
   - Re-export the macros
   - Get compiled to .rlib
   - User code depends on the wrapper, not the proc macro

4. **This is how serde works**
   - `serde_derive.so` is the proc macro
   - `serde.rlib` is the wrapper that re-exports it
   - User code: `use serde::Serialize;` (from wrapper)
   - Never touches serde_derive directly

### Why Our Original Approach Failed

We tried to:
- Generate .rlib files for proc macros ❌
- Use .so files directly with --extern ❌
- Register proc macros without wrapper pattern ❌

All failed because we didn't understand the wrapper library pattern.

### Why The New Approach Will Work

Following the wrapper pattern:
- WASM proc macro → Synthetic wrapper "crate" → User code ✅
- Matches native behavior exactly ✅
- Uses existing rustc infrastructure ✅
- Natural user experience ✅

## Quick Start for Next Session

```bash
# 1. Review status
cat IMPLEMENTATION_STATUS.md

# 2. Check what's done
git diff compiler/rustc_session/src/options.rs
git diff compiler/rustc_session/src/config.rs
git diff compiler/rustc_metadata/src/creader.rs

# 3. See working example
ls -l template* test_template* libtemplate*

# 4. Start Phase 2 implementation
# (Follow detailed steps in IMPLEMENTATION_STATUS.md)
```

## Success Criteria

Phase 2 will be complete when this works:

```bash
cat > test_demo.rs << 'EOF'
use Demo::SomeMacro;

#[derive(SomeMacro)]
struct Test;

fn main() {}
EOF

wasmtime run -Sthreads=yes --dir . dist/bin/rustc.wasm \
  --sysroot dist \
  --wasm-proc-macro Demo=demo.wasm \
  test_demo.rs \
  --target wasm32-wasip1 \
  --edition 2021
# ✅ Should compile without "cannot find derive macro" error
```

## Timeline

| Phase | Status | Time Spent | Time Remaining |
|-------|--------|------------|----------------|
| Investigation | ✅ Complete | ~8 hours | - |
| Phase 1 | ✅ Complete | (previous work) | - |
| Phase 2 | ⏸️ Ready | - | 10-14 hours |
| Testing | ⏸️ Pending | - | 2-3 hours |

**Total estimated:** 12-17 hours to complete implementation

## Contact/Questions

For questions about this implementation:
- Read IMPLEMENTATION_STATUS.md for detailed next steps
- Check PROC_MACRO_PATTERN_DISCOVERED.md for pattern explanation
- Review working example files (template_*.rs)

## File Index

### Documentation
- `README.md` ← You are here
- `IMPLEMENTATION_STATUS.md` ← **Start here for next steps**
- `PROC_MACRO_PATTERN_DISCOVERED.md` ← **Pattern explanation**
- `FINAL_PROC_MACRO_SOLUTION.md` ← **Implementation guide**
- `CRITICAL_DISCOVERY.md` ← **Key finding**
- `WASM_PROC_MACRO_SUMMARY.md` ← Earlier summary
- `PHASE_2_REGISTRATION_PLAN.md` ← Original plan
- `PHASE_2_REGISTRATION_STATUS.md` ← Investigation
- `SOLUTION_A_INVESTIGATION.md` ← .rlib investigation

### Test Code
- `template_proc_macro.rs` ← Test proc macro
- `libtemplate_proc_macro.so` ← Compiled proc macro
- `template_lib.rs` ← Wrapper library ⭐
- `libtemplate_lib.rlib` ← Compiled wrapper
- `test_template_v2.rs` ← User code
- `test_template_v2` ← Working binary!

### Cargo Test
- `/tmp/test_proc_macro_usage/` ← Test project
- `/tmp/cargo_verbose_build.log` ← Build analysis

## Modified Rustc Files

```
compiler/rustc_session/src/
├── options.rs              [MODIFIED] - Added wasm_proc_macros field
└── config.rs               [MODIFIED] - Flag parsing

compiler/rustc_metadata/src/
├── creader.rs              [MODIFIED] - Load WASM proc macros
└── lib.rs                  [Will modify] - Export new module

compiler/rustc_resolve/src/
└── lib.rs                  [MODIFIED] - Call load_wasm_proc_macros

compiler/rustc_watt_runtime/
└── (multiple files)        [NEW] - WASM runtime support
```

## Conclusion

**Investigation: ✅ Complete**
**Phase 1: ✅ Complete**
**Phase 2: ⏸️ Ready for Implementation**

We have successfully:
- Discovered the fundamental proc macro pattern
- Validated it with working examples
- Created comprehensive documentation
- Implemented Phase 1 infrastructure
- Designed clear path for Phase 2

The next session can proceed directly to implementation with all necessary context and understanding in place.

---

**Last Updated:** November 11, 2025
**Ready for:** Phase 2 Implementation
