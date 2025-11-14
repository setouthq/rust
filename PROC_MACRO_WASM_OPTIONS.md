# Proc Macro Support for WASM/WASI Rustc: Implementation Options

## Executive Summary

This document outlines three approaches for adding procedural macro support to rustc compiled for and targeting WASM/WASI. The current branch (1.84.1-wasi) successfully compiles proc macros to WASM dylibs but lacks the runtime loading mechanism to execute them.

## Current State Analysis

### What Works ✅

1. **Proc Macro Compilation to WASM**
   - Proc macros compile to `.wasm` dylib files
   - Proper symbol exports (`__rustc_proc_macro_decls_*`)
   - Metadata correctly embedded in object files
   - Location: `compiler/rustc_codegen_ssa/src/back/link.rs`

2. **WASI Threading Support**
   - Target `wasm32-wasip1-threads` configured with:
     - Shared memory and atomics enabled
     - Thread-local storage support
     - Dynamic linking capabilities
   - Location: `compiler/rustc_target/src/spec/targets/wasm32_wasip1_threads.rs`

3. **WASM Linker Infrastructure**
   - In-process LLD linking via embedded wrapper
   - Proper handling of thread model and memory settings
   - Location: `compiler/rustc_llvm/llvm-wrapper/LLD.cpp` (new), `compiler/rustc_codegen_ssa/src/back/linker.rs`

### Critical Gap ❌

**Proc Macro Loading/Execution**
- Native platforms (Unix/Windows): Use `libloading` crate for FFI-based dynamic loading
- WASI: Returns hardcoded error "dlopen not supported on this platform"
- Location: `compiler/rustc_metadata/src/creader.rs:720-722`

The issue is that WASI doesn't have `dlopen`/`LoadLibrary` equivalents, so the standard FFI approach doesn't work.

---

## Option A: WASM Component Model with Embedded Runtime

### Overview

Embed a WASM runtime (Wasmtime or Wasmer) inside rustc to dynamically load and execute proc macro dylibs as WASM modules.

### Architecture

```
rustc (running as WASM on WASI)
  └─> Embedded WASM Runtime (Wasmtime/Wasmer)
      └─> Load proc_macro.wasm
          └─> Call expand() via component model interface
              └─> Pass TokenStream, receive TokenStream
```

### Implementation Steps

1. **Add WASM runtime dependency**
   ```toml
   # compiler/rustc_metadata/Cargo.toml
   [target.'cfg(target_family = "wasm")'.dependencies]
   wasmtime = { version = "...", features = ["component-model", "cranelift"] }
   ```

2. **Modify proc macro loader** (`compiler/rustc_metadata/src/creader.rs`)
   - Add WASI-specific branch in `dlsym_proc_macros()`
   - Create WASM runtime engine
   - Load the `.wasm` dylib as a module
   - Link component model interface
   - Call proc macro functions through runtime

3. **Create component model interface** (`compiler/rustc_proc_macro_interface/`)
   - Define WIT (WebAssembly Interface Types) for proc macro protocol
   - TokenStream serialization format
   - Error handling across boundary

4. **Update proc macro dylib compilation**
   - Ensure proc macros export component model interface
   - Add necessary adapter code for component model

### Example Code Structure

```rust
// compiler/rustc_metadata/src/creader.rs

#[cfg(target_family = "wasm")]
fn dlsym_proc_macros_wasi(path: &Path) -> Result<DylibData, String> {
    use wasmtime::*;

    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    let module = Module::from_file(&engine, path)
        .map_err(|e| format!("Failed to load proc macro: {}", e))?;

    let instance = Instance::new(&mut store, &module, &[])
        .map_err(|e| format!("Failed to instantiate: {}", e))?;

    // Look up proc macro exports
    let decls_func = instance
        .get_func(&mut store, "__rustc_proc_macro_decls")
        .ok_or("Missing proc macro declarations")?;

    // Call and parse declarations
    // ... implement token stream bridge ...

    Ok(DylibData { /* ... */ })
}
```

### Pros

- **Most "native" solution**: Directly loads WASM modules as designed
- **Self-hosting capable**: Rustc compiled to WASM can compile code using WASM proc macros
- **Security**: WASM sandbox provides isolation
- **Portable**: Works across any WASI runtime

### Cons

- **Runtime overhead**: Embedding a full WASM runtime adds significant binary size (~5-10MB+)
- **Performance**: Extra layer of indirection, JIT compilation overhead
- **Memory complexity**: Managing memory between host WASM (rustc) and guest WASM (proc macro)
- **Maturity**: Component model still evolving, tooling not fully stable
- **Recursive WASM**: Running WASM inside WASM can hit implementation limits

### Technical Challenges

1. **Memory model**: Linear memory in rustc WASM vs linear memory in proc macro WASM
2. **TokenStream serialization**: Efficient cross-boundary data transfer
3. **Runtime initialization cost**: Creating engine/store per proc macro
4. **Stack depth**: Nested WASM calls may hit stack limits
5. **Component model adoption**: Not all proc macros may support component model exports

### Estimated Effort

**High** - 4-6 weeks for experienced developer
- 1 week: Runtime integration and basic loading
- 2 weeks: Component model interface design and implementation
- 1-2 weeks: TokenStream bridge and serialization
- 1 week: Testing, debugging, optimization

---

## Option B: RPC/IPC Plugin System

### Overview

Run proc macros as separate WASI processes and communicate via IPC (stdin/stdout). Similar to how rust-analyzer handles proc macros.

### Architecture

```
rustc (WASM process)
  └─> spawn proc_macro.wasm as subprocess
  └─> communicate via stdin/stdout (JSON-RPC)
      └─> Request: { "method": "expand", "tokens": [...] }
      └─> Response: { "result": [...] }
```

### Implementation Steps

1. **Create proc macro server protocol** (`compiler/rustc_proc_macro_srv/`)
   - Define JSON-RPC protocol for proc macro operations
   - Message types: `expand_derive`, `expand_attr`, `expand_fn_like`
   - TokenStream serialization (reuse `proc_macro2` format)

2. **Implement proc macro server binary**
   - New crate that proc macros link against when compiled for WASM
   - Reads requests from stdin, writes responses to stdout
   - Loads proc macro dylib internally (or links statically)

3. **Modify rustc proc macro loader** (`compiler/rustc_metadata/src/creader.rs`)
   - WASI-specific branch that spawns subprocess instead of dlopen
   - Implement IPC client that sends/receives JSON messages
   - Cache subprocess handles to avoid repeated spawning

4. **Update proc macro compilation**
   - Add server main() function to proc macro crates when targeting WASM
   - Static linking mode: bundle proc macro into standalone binary

### Example Protocol

```json
// Request
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "expand_derive",
  "params": {
    "macro_name": "Serialize",
    "input": {
      "tokens": [
        {"kind": "Ident", "text": "struct"},
        {"kind": "Ident", "text": "Foo"},
        {"kind": "Group", "delimiter": "Brace", "stream": [...]}
      ]
    }
  }
}

// Response
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "tokens": [...]
  }
}
```

### Example Code Structure

```rust
// compiler/rustc_metadata/src/creader.rs

#[cfg(target_family = "wasm")]
fn dlsym_proc_macros_wasi(path: &Path) -> Result<DylibData, String> {
    // Spawn proc macro server process
    let mut child = std::process::Command::new("wasmtime")
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn proc macro: {}", e))?;

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    // List proc macros by sending "list" request
    let request = json!({
        "jsonrpc": "2.0",
        "id": 0,
        "method": "list",
        "params": {}
    });

    serde_json::to_writer(&mut stdin, &request)?;
    stdin.write_all(b"\n")?;
    stdin.flush()?;

    let mut response = String::new();
    stdout.read_line(&mut response)?;
    let list: ListResponse = serde_json::from_str(&response)?;

    Ok(DylibData {
        handle: ProcMacroHandle::Subprocess { child, stdin, stdout },
        macros: list.macros,
    })
}

// When expanding:
fn expand_proc_macro(handle: &ProcMacroHandle, input: TokenStream) -> TokenStream {
    match handle {
        ProcMacroHandle::Subprocess { stdin, stdout, .. } => {
            let request = json!({
                "method": "expand_derive",
                "params": { "input": serialize_tokens(input) }
            });

            serde_json::to_writer(stdin, &request)?;
            stdin.flush()?;

            let mut response = String::new();
            stdout.read_line(&mut response)?;
            let result: ExpandResponse = serde_json::from_str(&response)?;

            deserialize_tokens(result.tokens)
        }
    }
}
```

### Pros

- **Simple architecture**: Well-understood IPC pattern
- **Process isolation**: Each proc macro runs in separate sandbox
- **No runtime embedding**: Rustc binary stays smaller
- **Proven approach**: rust-analyzer uses similar technique successfully
- **Easier debugging**: Can inspect IPC messages
- **Flexible**: Could use different transports (sockets, pipes, etc.)

### Cons

- **Subprocess overhead**: Spawning processes is expensive (can be mitigated with caching)
- **IPC latency**: Serialization/deserialization adds overhead
- **TokenStream serialization**: Need stable format across versions
- **Process management**: Must handle crashes, timeouts, cleanup
- **Requires WASI runtime**: Must support process spawning (not all do)

### Technical Challenges

1. **Process spawning on WASI**: Ensure target WASI runtime supports spawning (Wasmtime does)
2. **TokenStream format**: Design stable, efficient serialization
3. **Error propagation**: Handle subprocess crashes gracefully
4. **Performance**: Minimize serialization overhead
5. **Caching strategy**: Keep subprocesses alive vs spawn per invocation

### Estimated Effort

**Medium** - 3-4 weeks for experienced developer
- 1 week: Protocol design and implementation
- 1 week: Server binary and proc macro integration
- 1 week: Rustc IPC client integration
- 1 week: Testing, error handling, optimization

---

## Option C: Dual-Compilation Strategy (Hybrid Approach)

### Overview

Compile proc macros for both native host (x86_64/aarch64) AND target (wasm32-wasi). During compilation, load native dylibs using standard FFI even when targeting WASM.

### Architecture

```
Build Process:
1. Compile proc_macro for native (x86_64)
2. Compile proc_macro for wasm32-wasi
3. During compilation:
   - rustc (any platform) loads native x86_64 dylib
   - Produces wasm32-wasi output

Self-hosting scenario:
1. rustc running as WASM can't load native dylibs
2. Falls back to precompiled native proc macros
3. Or uses Option B (RPC) as fallback
```

### Implementation Steps

1. **Extend build system** (Cargo/rustc integration)
   - When building for WASM target, also build proc macros for host
   - Store native dylibs in separate directory
   - Metadata tracks both versions

2. **Modify proc macro resolver** (`compiler/rustc_metadata/src/creader.rs`)
   - Check if both native and WASM versions exist
   - Prefer native dylib for loading
   - Use WASM version only if self-hosting

3. **Add proc macro cache/registry**
   - Precompile common proc macros (serde, tokio, etc.) for native
   - Ship with rustc distribution
   - Users can add their own to cache

4. **Fallback mechanism**
   - If native dylib not available and running on WASM, use Option B (RPC)
   - Graceful degradation path

### Example Build Configuration

```toml
# .cargo/config.toml
[target.wasm32-wasip1-threads]
# Always build proc macros for host too
proc-macro-native-build = true
proc-macro-cache-dir = ".proc-macros/native"
```

### Example Code Structure

```rust
// compiler/rustc_metadata/src/creader.rs

fn dlsym_proc_macros(path: &Path) -> Result<DylibData, String> {
    // Check for native version first
    let native_path = path.with_extension(env::consts::DLL_EXTENSION);

    if native_path.exists() && !cfg!(target_family = "wasm") {
        // We're on native platform, load native dylib
        return dlsym_proc_macros_native(&native_path);
    }

    #[cfg(not(target_family = "wasm"))]
    {
        // Native rustc compiling to WASM: load native dylib
        return dlsym_proc_macros_native(&native_path);
    }

    #[cfg(target_family = "wasm")]
    {
        // WASM rustc: try to load native dylib from cache
        if let Some(cached) = try_load_cached_native_dylib(path) {
            return Ok(cached);
        }

        // Fall back to RPC/subprocess approach
        return dlsym_proc_macros_wasi_rpc(path);
    }
}
```

### Pros

- **Immediate solution**: Works today for cross-compilation
- **Best performance**: Native FFI has zero overhead
- **Incremental path**: Can be combined with other options
- **Compatibility**: Existing proc macros work unchanged
- **No runtime changes**: Minimal rustc modifications

### Cons

- **Not true self-hosting**: WASM rustc can't compile code using WASM proc macros independently
- **Build complexity**: Must build proc macros twice
- **Storage overhead**: Two versions of every proc macro
- **Distribution issues**: Must ship native dylibs alongside WASM
- **Platform matrix**: N targets × M hosts = lots of combinations

### Technical Challenges

1. **Build system integration**: Cargo must build both versions
2. **Path management**: Track native and WASM versions separately
3. **Cache invalidation**: Ensure native/WASM versions stay in sync
4. **Distribution**: How to package and ship native dylibs
5. **Self-hosting gap**: Still need Option A or B for true self-hosting

### Estimated Effort

**Low-Medium** - 2-3 weeks for experienced developer
- 1 week: Build system changes (Cargo integration)
- 1 week: Path resolution and caching logic
- 1 week: Testing across platforms

---

## Comparison Matrix

| Aspect | Option A: Embedded Runtime | Option B: RPC/IPC | Option C: Dual-Compilation |
|--------|---------------------------|-------------------|---------------------------|
| **Self-hosting** | ✅ Full support | ✅ Full support | ⚠️ Partial (needs fallback) |
| **Performance** | ⚠️ Moderate (JIT overhead) | ⚠️ Moderate (IPC overhead) | ✅ Excellent (native FFI) |
| **Binary size** | ❌ +5-10MB | ✅ Minimal | ✅ Minimal |
| **Complexity** | ❌ High | ⚠️ Medium | ✅ Low |
| **Portability** | ✅ Any WASI runtime | ⚠️ Needs subprocess support | ⚠️ Needs native dylibs |
| **Debugging** | ❌ Difficult | ✅ Good (inspect IPC) | ✅ Good (native tools) |
| **Isolation** | ✅ WASM sandbox | ✅ Process isolation | ❌ Same process |
| **Development time** | ❌ 4-6 weeks | ⚠️ 3-4 weeks | ✅ 2-3 weeks |
| **Risk** | ⚠️ High (new tech) | ⚠️ Medium | ✅ Low |
| **Maintenance** | ❌ High | ⚠️ Medium | ✅ Low |

---

## Recommendations

### For Production Use: **Option B (RPC/IPC)** + **Option C (Dual-Compilation)**

**Rationale:**
1. Start with Option C for immediate cross-compilation support
2. Implement Option B for self-hosting scenarios
3. Rustc automatically selects best available method:
   - Native platform → Native FFI (Option C)
   - WASM cross-compiling → Native FFI (Option C)
   - WASM self-hosting → RPC/IPC (Option B)

**Benefits:**
- Covers all use cases
- Good performance in common case (cross-compilation)
- True self-hosting capability when needed
- Reasonable complexity and maintenance burden

### For Research/Experimentation: **Option A (Embedded Runtime)**

**Rationale:**
- Most architecturally "pure" solution
- Pushes WASM ecosystem forward
- Good for academic/research contexts

**Caveats:**
- Higher implementation complexity
- Performance may not meet production needs
- Tooling still maturing

### Phased Implementation Plan

**Phase 1: Foundation (2-3 weeks)**
- Implement Option C (dual-compilation)
- Get basic cross-compilation working
- Validate build system changes

**Phase 2: Self-Hosting (3-4 weeks)**
- Implement Option B (RPC/IPC)
- Design protocol and server
- Integration testing

**Phase 3: Optimization (2-3 weeks)**
- Proc macro caching strategies
- Subprocess pooling
- Performance tuning

**Phase 4: (Optional) Native Alternative (4-6 weeks)**
- Implement Option A if needed
- Benchmark against Option B
- Evaluate for specific use cases

---

## Related Files and Code Locations

### Key Files to Modify

1. **`compiler/rustc_metadata/src/creader.rs`**
   - Lines 720-722: Current WASI error
   - Lines 634-826: `dlsym_proc_macros()` function
   - Add WASI-specific loading logic here

2. **`compiler/rustc_target/src/spec/targets/wasm32_wasip1_threads.rs`**
   - Current WASI target configuration
   - May need adjustments for dynamic linking

3. **`compiler/rustc_codegen_ssa/src/back/link.rs`**
   - Lines 1000-1500: Binary linking logic
   - Ensure proc macro dylibs link correctly

4. **`compiler/rustc_codegen_ssa/src/back/linker.rs`**
   - Lines 1100-1400: WasmLd implementation
   - May need exports for component model (Option A)

### New Components to Create

1. **`compiler/rustc_proc_macro_srv/`** (Option B)
   - Protocol definition
   - Server implementation
   - Client integration

2. **`compiler/rustc_proc_macro_interface/`** (Option A)
   - Component model WIT definitions
   - TokenStream serialization
   - Runtime integration

3. **`compiler/rustc_proc_macro_cache/`** (Option C)
   - Native dylib cache management
   - Path resolution logic

---

## Testing Strategy

### Unit Tests
- TokenStream serialization/deserialization
- Protocol message handling
- Path resolution logic

### Integration Tests
- Simple derive macro (e.g., Debug, Clone)
- Attribute macro with modifications
- Function-like macro with complex expansion
- Error propagation and reporting

### Performance Benchmarks
- Proc macro expansion latency
- Memory usage
- Compilation time impact
- Compare against native baseline

### Self-Hosting Test
- Compile rustc to WASM using rustc-on-WASM
- Use serde proc macros in test crate
- Verify correctness of generated code

---

## Open Questions

1. **Component Model Adoption**: How many proc macros can easily adapt to component model exports?
2. **WASI Runtime Capabilities**: Which WASI runtimes support subprocess spawning reliably?
3. **Performance Targets**: What latency/overhead is acceptable for proc macro expansion?
4. **Distribution**: How should we distribute native proc macro dylibs?
5. **Versioning**: How do we ensure proc macro server protocol compatibility across rustc versions?

---

## References

- [WASI Preview 2 Specification](https://github.com/WebAssembly/WASI/blob/main/preview2/README.md)
- [Component Model Proposal](https://github.com/WebAssembly/component-model)
- [rust-analyzer proc macro server](https://github.com/rust-lang/rust-analyzer/tree/master/crates/proc-macro-srv)
- [Wasmtime Documentation](https://docs.wasmtime.dev/)
- [Rustc Dev Guide: Procedural Macros](https://rustc-dev-guide.rust-lang.org/backend/libs-and-metadata.html)

---

## Conclusion

Adding proc macro support to WASM rustc is achievable with several viable approaches. The recommended path is a combination of Option B (RPC/IPC) and Option C (dual-compilation), providing both immediate cross-compilation support and a path to true self-hosting. This hybrid approach balances implementation complexity, performance, and maintenance burden while supporting all anticipated use cases.

The key decision factors are:
- **Timeline**: How quickly do you need a solution?
- **Self-hosting priority**: Is true self-hosting critical?
- **Performance requirements**: What overhead is acceptable?
- **Maintenance capacity**: How much ongoing maintenance can you support?

Based on your requirements, you can select the most appropriate approach or combination of approaches.
