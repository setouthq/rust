# WASM Proc Macro Implementation - Current Status

**Date:** November 11, 2025
**Status:** Investigation Complete, Ready for Phase 2 Implementation

## What We Accomplished

### ✅ Phase 1: Complete and Working
- `--wasm-proc-macro NAME=PATH` flag parsing ✅
- WASM file loading ✅
- Proc macro metadata extraction ✅
- Integration into compilation flow ✅
- Debug logging implemented ✅

**Code locations:**
- `compiler/rustc_session/src/options.rs:173` - Options field
- `compiler/rustc_session/src/config.rs:1580-2667` - Flag parsing
- `compiler/rustc_metadata/src/creader.rs:325-376` - Loading logic
- `compiler/rustc_resolve/src/lib.rs:1723-1726` - Integration

### ✅ Investigation: Breakthrough Discoveries

**Key Discovery:** Proc macros use a "wrapper library pattern"
- Proc macros (`.so` files) cannot be used directly
- Must be accessed through wrapper libraries (`.rlib`) that re-export them
- This is how serde works: `user_code` → `serde.rlib` → `serde_derive.so`

**Validation:**
- Created working native proc macro example ✅
- Tested wrapper library pattern ✅
- Analyzed Cargo's build process ✅
- Documented complete pattern ✅

**Files created:**
- `CRITICAL_DISCOVERY.md` - Discovered .rlib false assumption
- `PROC_MACRO_PATTERN_DISCOVERED.md` - Complete pattern documentation
- `FINAL_PROC_MACRO_SOLUTION.md` - Implementation roadmap
- Working test files (template_proc_macro.rs, template_lib.rs, test_template_v2.rs)

## What's Needed: Phase 2 Implementation

### The Goal

Make `--wasm-proc-macro Demo=demo.wasm` work like this:

```rust
// User code can import and use the macro
use Demo::SomeMacro;

#[derive(SomeMacro)]
struct Foo {
    // ...
}
```

### The Approach

Create a "synthetic wrapper crate" that:
1. Acts like a normal library crate
2. Re-exports proc macros from the WASM module
3. Is registered in the CStore
4. Is visible to the resolver

### Implementation Complexity

The `register_crate` function (compiler/rustc_metadata/src/creader.rs:454-526) requires:
- `Library` struct with `source` and `metadata`
- `MetadataBlob` with encoded crate information
- `CrateRoot` structure
- Dependency resolution (`resolve_crate_deps`)
- `CrateMetadata` creation

**Challenge:** Creating minimal but valid metadata is non-trivial.

### Two Implementation Options

#### Option A: Full Synthetic Metadata (Recommended)

Create complete `CrateMetadata`:
```rust
// Pseudo-code
let metadata = create_minimal_metadata(name, &proc_macros);
let library = Library {
    source: CrateSource::from_wasm(path),
    metadata: metadata,
};
let cnum = self.register_crate(None, None, library, dep_kind, name, None)?;
```

**Pros:**
- Most compatible with rustc
- Follows existing patterns
- Robust and maintainable

**Cons:**
- Complex (requires understanding metadata encoding)
- Estimated 10-14 hours

**Recommendation:** This is the correct long-term approach.

#### Option B: Direct Resolver Integration (Simpler)

Bypass `register_crate` and directly inject proc macros:
```rust
// Pseudo-code
// Register directly with resolver
self.resolver.register_wasm_proc_macros(name, proc_macros);
```

**Pros:**
- Simpler/faster (4-6 hours)
- Less code

**Cons:**
- May break rustc assumptions
- Less maintainable
- Might not handle all cases

**Recommendation:** Only if Option A proves too difficult.

## Detailed Next Steps for Phase 2

### Step 1: Study Metadata Encoding (2-3 hours)

Understand how to create minimal metadata:
1. Read `compiler/rustc_metadata/src/rmeta/encoder.rs`
2. Understand `MetadataBlob` structure
3. Find minimal metadata for proc-macro-only crate
4. Study existing proc-macro crate metadata

**Suggested approach:**
```bash
# Extract metadata from a real proc-macro crate
cargo build --release
ar x target/release/libserde_derive-*.so
xxd rust.metadata.bin | head -100
```

### Step 2: Create Synthetic Metadata Helper (4-5 hours)

```rust
// compiler/rustc_metadata/src/wasm_synthetic.rs

use rustc_metadata::rmeta::*;

pub fn create_synthetic_proc_macro_metadata(
    name: Symbol,
    proc_macros: &[ProcMacro],
    wasm_path: &Path,
) -> MetadataBlob {
    // Create minimal metadata containing:
    // 1. Crate name
    // 2. Proc macro declarations
    // 3. Source path (pointing to WASM)

    let mut encoder = Encoder::new();

    // Encode header
    encoder.emit_raw_bytes(&METADATA_HEADER);

    // Encode crate info
    encoder.encode_crate_name(name);
    encoder.encode_stable_crate_id(/* generate */);

    // Encode proc macros
    for pm in proc_macros {
        encoder.encode_proc_macro(pm);
    }

    // Create blob
    MetadataBlob::new(encoder.into_inner())
}
```

### Step 3: Create Synthetic Library (2-3 hours)

```rust
// In creader.rs, modify load_wasm_proc_macros:

pub fn load_wasm_proc_macros(&mut self) {
    #[cfg(target_family = "wasm")]
    {
        for (name, path) in &self.sess.opts.wasm_proc_macros {
            // Phase 1: Load WASM and extract macros (existing)
            let wasm_bytes = fs::read(path)?;
            let wasm_macro = WasmMacro::new_owned(wasm_bytes);
            let proc_macros = create_wasm_proc_macros(wasm_macro);

            // Phase 2: Create synthetic library (NEW)
            let metadata = create_synthetic_proc_macro_metadata(
                Symbol::intern(name),
                &proc_macros,
                path,
            );

            let library = Library {
                source: CrateSource {
                    dylib: Some((path.clone(), PathKind::All)),
                    rlib: None,
                    rmeta: None,
                },
                metadata,
            };

            // Register as normal crate
            let name_symbol = Symbol::intern(name);
            match self.register_crate(
                None,  // host_lib
                None,  // root
                library,
                CrateDepKind::Explicit,
                name_symbol,
                None,  // private_dep
            ) {
                Ok(cnum) => {
                    eprintln!("[CREADER] Registered WASM proc macro crate: {} (cnum={})",
                             name, cnum);
                }
                Err(e) => {
                    self.dcx().fatal(format!(
                        "Failed to register WASM proc macro {}: {:?}",
                        name, e
                    ));
                }
            }
        }
    }
}
```

### Step 4: Handle WASM Execution (2-3 hours)

Ensure that when proc macros are expanded, they execute the WASM:
```rust
// The existing WasmMacro and watt runtime should handle this
// Just need to ensure the ProcMacro structs point to the right runtime
```

### Step 5: Testing (2-3 hours)

```bash
# Test 1: Basic derive
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

# Test 2: Multiple macros
# Test 3: Attribute macros
# Test 4: Function-like macros
```

## Alternative: Simpler Approach for Quick Win

If full metadata creation is too complex, try this simpler approach:

### Treat WASM proc macros like `--extern`

```bash
# User creates wrapper library manually
cat > demo_wrapper.rs << 'EOF'
pub use demo_proc_macro::SomeMacro;
EOF

# Compile wrapper with WASM proc macro
rustc.wasm demo_wrapper.rs --crate-type lib \
  --wasm-proc-macro demo_proc_macro=demo.wasm

# Use wrapper in user code
rustc.wasm user_code.rs --extern demo_wrapper=libdemo_wrapper.rlib
```

This requires users to create wrapper libraries manually, but:
- ✅ Works with existing infrastructure
- ✅ Much simpler implementation
- ✅ Proves the concept
- ❌ Less user-friendly

## Estimated Timelines

| Approach | Time | Complexity | User Experience |
|----------|------|------------|-----------------|
| Full Synthetic Metadata | 10-14 hours | High | Excellent |
| Direct Resolver Integration | 4-6 hours | Medium | Good |
| Manual Wrapper Libraries | 1-2 hours | Low | Poor |

## Recommendation

**For Production:** Implement Full Synthetic Metadata (Option A)
- Takes longer but provides best experience
- Most maintainable approach
- Follows rustc patterns

**For Proof of Concept:** Try Direct Resolver Integration (Option B)
- Can validate the approach quickly
- Can upgrade to Option A later if it works

**For Immediate Testing:** Manual Wrapper Libraries
- Tests that the WASM runtime works
- Validates end-to-end flow
- Can be implemented in 1-2 hours

## Current Blockers

None - investigation complete, ready for implementation.

## Required Knowledge for Phase 2

1. **Metadata encoding** - How rustc encodes crate metadata
2. **CStore internals** - How crates are registered and stored
3. **Proc macro representation** - How proc macros are represented in metadata
4. **Resolver integration** - How resolver finds and uses proc macros

**Suggested reading:**
- `compiler/rustc_metadata/src/rmeta/encoder.rs`
- `compiler/rustc_metadata/src/rmeta/decoder.rs`
- `compiler/rustc_middle/src/middle/exported_symbols.rs`

## Files to Modify for Phase 2

1. **Create new file:** `compiler/rustc_metadata/src/wasm_synthetic.rs`
   - Synthetic metadata creation
   - Helper functions

2. **Modify:** `compiler/rustc_metadata/src/creader.rs`
   - Update `load_wasm_proc_macros` (lines 362-366)
   - Add synthetic crate registration

3. **Modify:** `compiler/rustc_metadata/src/lib.rs`
   - Export new wasm_synthetic module

4. **Possibly modify:** `compiler/rustc_resolve/src/macros.rs`
   - If direct resolver integration is chosen

## Success Criteria

Phase 2 will be complete when:

- ✅ `--wasm-proc-macro Demo=demo.wasm` registers a crate named "Demo"
- ✅ User can write `use Demo::SomeMacro;`
- ✅ `#[derive(SomeMacro)]` compiles without errors
- ✅ Macro expansion executes WASM code
- ✅ Generated code compiles correctly
- ✅ Multiple WASM proc macros work simultaneously

## Test Command

```bash
# Should work when Phase 2 is complete:
wasmtime run -Sthreads=yes --dir .::/ dist/bin/rustc.wasm \
  --sysroot dist \
  --wasm-proc-macro Demo=watt_demo_with_metadata.wasm \
  test_watt_demo.rs \
  --target wasm32-wasip1 \
  --edition 2021 \
  -o test.wasm

# Expected: No "cannot find derive macro" error
# Expected: Macro expands correctly
# Expected: Code compiles
```

## Next Session Recommendations

1. **Start with metadata study** - Spend time understanding how to create minimal metadata
2. **Prototype separately** - Create a standalone tool that generates synthetic metadata first
3. **Test incrementally** - Test each piece (metadata creation, registration, resolution) separately
4. **Use debug logging** - Add extensive logging to track where things break
5. **Compare with native** - Use working native proc macros as reference

## Conclusion

**Status: Investigation Complete ✅**

We have:
- ✅ Working Phase 1 implementation
- ✅ Complete understanding of proc macro pattern
- ✅ Clear implementation path
- ✅ Multiple implementation options
- ✅ Comprehensive documentation

**Next: Implement Phase 2**

The work ahead is well-defined. Choose an implementation approach and proceed systematically.

---

**Total Investigation Time:** ~8 hours
**Estimated Implementation Time:** 10-14 hours (Option A) or 4-6 hours (Option B)
**Confidence:** High - pattern validated with working examples
