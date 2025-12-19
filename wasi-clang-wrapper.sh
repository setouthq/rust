#!/bin/bash
# Filter and translate arguments for clang
args=()
skip_next=false
prefix_next=""

for arg in "$@"; do
    if [ "$skip_next" = true ]; then
        skip_next=false
        continue
    fi
    if [ -n "$prefix_next" ]; then
        args+=("-Wl,$prefix_next,$arg")
        prefix_next=""
        continue
    fi
    
    case "$arg" in
        -flavor)
            skip_next=true
            ;;
        --export|--undefined)
            # Next arg is the symbol name
            prefix_next="$arg"
            ;;
        --export=*|--undefined=*|--stack-first|--allow-undefined|--no-demangle|--import-memory|--export-memory|--shared-memory|--max-memory=*|--gc-sections|--strip-all|--no-entry|--import-table)
            args+=("-Wl,$arg")
            ;;
        -O*)
            # Linker optimization level - pass through
            args+=("-Wl,$arg")
            ;;
        *)
            args+=("$arg")
            ;;
    esac
done

exec /home/ubuntu/1.92/rust/wasi-sdk-28.0-arm64-linux/bin/clang \
    --target=wasm32-wasip1-threads \
    --sysroot=/home/ubuntu/1.92/rust/wasi-sdk-28.0-arm64-linux/share/wasi-sysroot \
    -lwasi-emulated-signal \
    -lwasi-emulated-process-clocks \
    -lwasi-emulated-mman \
    "${args[@]}"
