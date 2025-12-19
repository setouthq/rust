#!/bin/bash
# Filter out -flavor wasm arguments
args=()
skip_next=false
for arg in "$@"; do
    if [ "$skip_next" = true ]; then
        skip_next=false
        continue
    fi
    if [ "$arg" = "-flavor" ]; then
        skip_next=true
        continue
    fi
    args+=("$arg")
done

# Put emulated libraries at the END so they resolve symbols from the main objects
exec /home/ubuntu/1.92/rust/wasi-sdk-28.0-arm64-linux/bin/wasm-ld \
    "${args[@]}" \
    -L/home/ubuntu/1.92/rust/wasi-sdk-28.0-arm64-linux/share/wasi-sysroot/lib/wasm32-wasip1-threads \
    /home/ubuntu/1.92/rust/wasi-sdk-28.0-arm64-linux/share/wasi-sysroot/lib/wasm32-wasip1-threads/libwasi-emulated-signal.a \
    /home/ubuntu/1.92/rust/wasi-sdk-28.0-arm64-linux/share/wasi-sysroot/lib/wasm32-wasip1-threads/libwasi-emulated-process-clocks.a \
    /home/ubuntu/1.92/rust/wasi-sdk-28.0-arm64-linux/share/wasi-sysroot/lib/wasm32-wasip1-threads/libwasi-emulated-mman.a
