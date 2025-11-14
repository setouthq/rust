# WASM Proc Macro Integration Status

## ✅ Completed (Phase 1 - Loading & Metadata)

### 1. Watt Runtime Integration
- **Location**: `compiler/rustc_watt_runtime/`
- **Status**: ✅ Complete and building
- Vendored watt runtime with modifications for rustc
- Added `WasmMacro::new_owned()` for owned byte vectors
- Public API for all three proc macro types

### 2. Metadata Extraction Module
- **Location**: `compiler/rustc_watt_runtime/src/metadata.rs`
- **Status**: ✅ Complete and building
- Parses custom WASM section `.rustc_proc_macro_decls`
- Supports all three proc macro types:
  - `derive:TraitName:function_name[:attributes]`
  - `attr:name:function_name`
  - `bang:name:function_name`
- Simple WASM parser with LEB128 decoding
- Text-based metadata format

### 3. WASM Proc Macro Loader
- **Location**: `compiler/rustc_metadata/src/creader.rs:702-768, 1291-1371`
- **Status**: ✅ Complete and building
- `dlsym_proc_macros()` detects `.wasm` files on WASM targets
- `dlsym_proc_macros_wasm()` loads WASM modules using watt runtime
- `create_wasm_proc_macros()` converts metadata to `ProcMacro` instances
- Proper `Client` wrappers for each macro type
- All strings leaked to `'static` for closure compatibility

### 4. Build Verification
- **Status**: ✅ rustc_metadata builds successfully for WASM
- All metadata handling compiles cleanly
- Integration with watt runtime complete

## 🚧 In Progress (Phase 2 - Compilation)

### Proc Macro Harness Modifications
- **Location**: `compiler/rustc_builtin_macros/src/proc_macro_harness.rs`
- **Status**: ⚠️ Partially implemented, needs fixes
- Added `mk_wasm_decls()` function to generate WASM exports
- Detects WASM targets via `sess.target.llvm_target.contains("wasm")`
- **Current issues**:
  - AST API usage needs corrections
  - `StrLit` structure needs `style` field
  - Attribute creation APIs need to use `cx.attr_word()`
  - Need to properly set `extern "C"` ABI

### What mk_wasm_decls Should Generate

For each proc macro, it should create:

1. **Wrapper function** with `#[no_mangle]` and `extern "C"`:
   ```rust
   #[no_mangle]
   pub extern "C" fn my_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
       original_my_derive(input)
   }
   ```

2. **Metadata section** embedded in WASM:
   ```rust
   #[link_section = ".rustc_proc_macro_decls"]
   #[used]
   static PROC_MACRO_METADATA: &str = "derive:MyDerive:my_derive\n";
   ```

### Required Fixes

1. **Fix attribute creation**:
   ```rust
   // Current (wrong):
   thin_vec![cx.attribute(cx.meta_word(span, sym::no_mangle))]

   // Should be:
   let mut item = cx.item_fn(...);
   item.attrs.push(cx.attr_word(sym::no_mangle, span));
   ```

2. **Fix StrLit creation**:
   ```rust
   // Add style field:
   ast::StrLit {
       symbol: Symbol::intern("C"),
       suffix: None,
       span,
       symbol_unescaped: Symbol::intern("C"),
       style: ast::StrStyle::Cooked,  // Add this
   }
   ```

3. **Fix extern ABI setting**:
   - May need to use `cx.fn_sig()` or similar helper
   - Or set during `item_fn` creation with proper signature construction

## 📋 Remaining Work

### Phase 2 Completion (Est. 1-2 days)
- [ ] Fix AST API usage in `mk_wasm_decls()`
- [ ] Test proc macro compilation for WASM targets
- [ ] Verify metadata section is embedded correctly
- [ ] Verify functions are exported with correct names

### Phase 3: Testing (Est. 2-3 days)
- [ ] Create simple test proc macro crate
- [ ] Compile it to WASM
- [ ] Verify metadata extraction works
- [ ] Test actual macro expansion via watt runtime
- [ ] Integration test with WASM rustc

### Phase 4: Integration & Documentation (Est. 1-2 days)
- [ ] Full self-hosting test
- [ ] Document compilation requirements
- [ ] Update rustc documentation
- [ ] Create example proc macros for WASM

## 🎯 Success Criteria

- [ ] WASM rustc can load `.wasm` proc macro files
- [ ] Metadata is correctly extracted from WASM custom sections
- [ ] Proc macros can be invoked through watt runtime
- [ ] Self-hosting test passes (WASM rustc compiles code using proc macros)
- [ ] All three proc macro types work (derive, attribute, function-like)

## 📝 Notes

### Architecture
```
┌─────────────────────────────────────────────────────────┐
│ rustc (WASM)                                            │
│  ├─ rustc_metadata/creader.rs                           │
│  │   └─ dlsym_proc_macros_wasm() ✅                     │
│  │        └─ Loads .wasm files                          │
│  │        └─ Calls extract_proc_macro_metadata()        │
│  │        └─ Creates ProcMacro instances                │
│  │                                                       │
│  ├─ rustc_watt_runtime ✅                               │
│  │   ├─ WasmMacro (watt runtime wrapper)                │
│  │   └─ metadata.rs (custom section parser)             │
│  │                                                       │
│  └─ rustc_builtin_macros/proc_macro_harness.rs ⚠️      │
│       └─ mk_wasm_decls() (generates WASM exports)       │
│            ├─ #[no_mangle] wrapper functions            │
│            └─ Metadata section                          │
└─────────────────────────────────────────────────────────┘
```

### Metadata Format
Simple text format, one macro per line:
- `derive:TraitName:function_name`
- `derive:TraitName:function_name:attr1,attr2`
- `attr:name:function_name`
- `bang:name:function_name`

### Key Design Decisions
1. **Text-based metadata**: Easy to parse, human-readable
2. **Custom WASM section**: Standard approach for embedding metadata
3. **Watt runtime**: Zero-dependency WASM interpreter, proven in production
4. **Function name matching**: Direct mapping between metadata and exports
5. **Leak for 'static**: Necessary for closure capture in ProcMacro::Client

## 🔗 Related Files

- `WATT_INTEGRATION_ANALYSIS.md` - Original design document
- `PROC_MACRO_WASM_OPTIONS.md` - Analysis of implementation approaches
- `compiler/rustc_watt_runtime/` - Watt runtime integration
- `compiler/rustc_metadata/src/creader.rs` - Proc macro loading
- `compiler/rustc_builtin_macros/src/proc_macro_harness.rs` - Proc macro generation

---

**Last Updated**: 2025-11-07
**Status**: Phase 1 Complete, Phase 2 In Progress
