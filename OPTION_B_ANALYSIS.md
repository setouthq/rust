# Option B Analysis: Modify Watt Runtime for Standard Proc Macros

**Date:** November 10, 2025

## Current Architecture Understanding

### Standard Proc Macro Library (`library/proc_macro/src/`)

**Key findings:**
1. **No WASM-specific code** - The standard proc_macro library has no special handling for WASM targets
2. **Uses Bridge abstraction** - All proc macro operations go through `Bridge::with()` which accesses thread-local state
3. **RPC-based communication** - TokenStreams are serialized using `Encode`/`Decode` traits and passed via `dispatch.call(buf)`
4. **Client structure** - `Client<I, O>` contains `run: extern "C" fn(BridgeConfig) -> Buffer`

```rust
// How standard proc macros work:
struct Bridge<'a> {
    cached_buffer: Buffer,
    dispatch: closure::Closure<'a, Buffer, Buffer>,  // RPC mechanism
    globals: ExpnGlobals<Span>,
}

// All proc macro operations call into the bridge:
fn some_operation() {
    Bridge::with(|bridge| {
        // Serialize request
        api_tags::Method::SomeOp.encode(&mut buf, &mut ());
        // Make RPC call to host
        buf = bridge.dispatch.call(buf);
        // Deserialize response
        Result::decode(&mut &buf[..], &mut ())
    })
}
```

### Watt Interpreter (`compiler/rustc_watt_runtime/src/interpret.rs`)

**Architecture:**
1. **Handle-based approach** - Stores TokenStreams in host-side registry (`Data::tokenstream`)
2. **Expects WASM exports** - Requires modules to export `raw_to_token_stream` and `token_stream_into_raw`
3. **Calling convention:**
   ```rust
   // watt converts: TokenStream → i32 handle
   let raw = Value::I32(data.tokenstream.push(input));

   // watt expects WASM to export conversion function
   let tokenstream_obj = call(exports.raw_to_token_stream, vec![raw]);

   // watt calls proc macro with TokenStream object
   let output = call(exports.main, vec![tokenstream_obj]);

   // watt expects WASM to convert back to handle
   let raw_output = call(exports.token_stream_into_raw, vec![output]);
   ```

4. **Imports provided** - watt provides imports under the `"watt-0.5"` module name:
   - `token_stream_serialize`
   - `token_stream_deserialize`
   - `token_stream_parse`
   - etc.

## The Fundamental Incompatibility

### Problem 1: Different Calling Conventions

**Standard proc macro:**
- Expects: `fn simple_test(input: TokenStream) -> TokenStream`
- TokenStream is `proc_macro::TokenStream` which wraps `bridge::client::TokenStream`
- All operations go through the Bridge RPC mechanism
- Requires thread-local Bridge state to be set up before calling

**watt's expectation:**
- Expects WASM to export: `raw_to_token_stream` and `token_stream_into_raw`
- Uses i32 handles instead of serialized TokenStreams
- Direct function calls, no RPC mechanism

### Problem 2: Bridge Initialization

Standard proc macros expect:
1. Host sets up thread-local `Bridge` state via `state::set()`
2. Provides a `dispatch` closure for RPC calls
3. Calls the proc macro function
4. Proc macro functions internally call `Bridge::with()` to access the dispatch

But in the watt interpreter:
1. WASM modules are instantiated with imports
2. No mechanism to set up thread-local state in the WASM module before calling it
3. The interpreter directly calls exported functions

### Problem 3: WASM Module Exports

When we compile a standard Rust proc macro to WASM:
- The proc macro functions are compiled into WASM
- They reference the standard `proc_macro` library
- They do NOT export `raw_to_token_stream` or `token_stream_into_raw`
- They expect the Bridge to be set up by the host

## Possible Solutions for Option B

### Solution B1: Modify proc_macro Library to Support WASM Mode

**Approach:**
1. Add WASM-specific code to `library/proc_macro/src/bridge/client.rs`
2. When compiled for WASM, instead of using thread-local Bridge:
   - Import host functions from watt interpreter
   - Use handle-based approach for TokenStream operations
3. Compile proc macros with this modified library

**Pros:**
- Works with any Rust proc macro source code
- Clean separation of concerns

**Cons:**
- Requires modifying upstream Rust standard library
- Would need to maintain a fork or get changes upstreamed
- Complex implementation - need to dual-mode the entire bridge

### Solution B2: Modify Watt Interpreter to Bridge Standard Proc Macros

**Approach:**
1. Instead of expecting WASM to export conversion functions
2. Provide them as imports that match the standard Bridge API
3. Set up a shim that:
   - Implements the `dispatch` closure as a WASM import
   - Provides it to the WASM proc macro somehow
   - Handles RPC serialization/deserialization

**Problem:**
- WASM modules can't set up thread-local state from imports at instantiation time
- The Bridge expects to be in thread-local storage before the proc macro function is called
- No way to "inject" the dispatch closure into the proc macro's Bridge

### Solution B3: Generate Wrapper Exports

**Approach:**
1. When compiling a proc macro to WASM, generate additional code that:
   - Exports `raw_to_token_stream` and `token_stream_into_raw`
   - These functions set up the Bridge and call into the standard proc macro
2. Essentially create a watt-compatible wrapper around standard proc macros

**Problem:**
- Still requires modifying the proc macro build process
- Need to generate the glue code somehow
- Complex to implement

## Fundamental Issue

The core problem is that **standard proc macros assume a dynamic library model** where:
- The host loads the library
- The host sets up thread-local state
- The host calls exported functions
- The functions can access thread-local state

But **WASM modules use an import/export model** where:
- Imports are bound at instantiation time
- Exports are called by the host
- No mechanism for "setting up thread-local state before calling exports"

## Recommendation

**Option B is significantly more complex than initially thought** because:

1. It requires either:
   - Forking the standard library's proc_macro crate, OR
   - Implementing a complex bridging mechanism that doesn't have a clear solution

2. The architecture mismatch is fundamental, not superficial

3. The watt framework exists precisely because it provides a different model that DOES work with WASM

**Therefore, Option A (Use Watt Framework) is more practical:**

### Option A: Compile Proc Macros with Watt

**What's needed:**
1. Proc macros import from `watt` instead of `proc_macro`
2. Use `#[watt::proc_macro]` instead of `#[proc_macro]`
3. Compile with watt's build infrastructure

**Example:**
```rust
// Instead of:
use proc_macro::TokenStream;

#[proc_macro_derive(SimpleTest)]
pub fn simple_test(input: TokenStream) -> TokenStream {
    // ...
}

// Use:
use watt::WasmMacro;

static MACRO: WasmMacro = WasmMacro::new(WASM);
static WASM: &[u8] = include_bytes!("macro_impl.wasm");

#[proc_macro_derive(SimpleTest)]
pub fn simple_test(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    MACRO.proc_macro_derive("simple_test_impl", input)
}

// Then in a separate crate:
#[watt::proc_macro]
pub fn simple_test_impl(input: watt::TokenStream) -> watt::TokenStream {
    // Implementation using watt's TokenStream
}
```

**Pros:**
- Works with existing watt runtime (already integrated)
- Proven approach used by watt crate in production
- No compiler modifications needed

**Cons:**
- Proc macros must be written specifically for watt
- Can't use arbitrary existing proc macros without modification
- watt's TokenStream API may differ slightly from standard proc_macro

## Next Steps

We should discuss with the user whether:
1. They want to pursue Option A (watt framework), which is more practical
2. They want to continue investigating Option B, understanding it will be significantly more complex
3. They want to explore a hybrid approach

The slot-based registry (Option 2) we implemented is still valuable - it solves the "zero-sized closure" problem. The question now is how to actually invoke the WASM proc macros at runtime.
