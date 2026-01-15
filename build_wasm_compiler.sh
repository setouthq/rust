#!/bin/bash
set -e

# Configuration
WASI_SDK_MAJOR_VER=25
WASI_SDK_MINOR_VER=0
WASI_SDK_VER="${WASI_SDK_MAJOR_VER}.${WASI_SDK_MINOR_VER}"
PROJECT_ROOT="$(pwd)"

# Detect Host Architecture
HOST_ARCH=$(uname -m)
if [ "$HOST_ARCH" = "x86_64" ]; then
    SDK_ARCH="x86_64"
    SDK_PLATFORM="linux"
elif [ "$HOST_ARCH" = "aarch64" ]; then
    SDK_ARCH="arm64"
    SDK_PLATFORM="linux"
else
    echo "Error: Unsupported architecture $HOST_ARCH"
    exit 1
fi

WASI_SDK_DIR_NAME="wasi-sdk-${WASI_SDK_VER}-${SDK_ARCH}-${SDK_PLATFORM}"
WASI_SDK_PATH="${PROJECT_ROOT}/${WASI_SDK_DIR_NAME}"
WASI_SDK_TAR="${WASI_SDK_DIR_NAME}.tar.gz"
WASI_SDK_URL="https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-${WASI_SDK_MAJOR_VER}/${WASI_SDK_TAR}"

# Download WASI SDK if missing
if [ ! -d "$WASI_SDK_PATH" ]; then
    echo "Downloading WASI SDK ${WASI_SDK_VER}..."
    if [ ! -f "$WASI_SDK_TAR" ]; then
        curl -L -o "$WASI_SDK_TAR" "$WASI_SDK_URL"
    fi
    echo "Extracting WASI SDK..."
    tar xf "$WASI_SDK_TAR"
    # Remove tarball to save space (optional)
    # rm "$WASI_SDK_TAR"
else
    echo "WASI SDK found at $WASI_SDK_PATH"
fi

WASI_SYSROOT="${WASI_SDK_PATH}/share/wasi-sysroot"
WASI_CLANG="${WASI_SDK_PATH}/bin/clang"

# Initialize submodules (essential for the build)
echo "Initializing submodules..."
# We explicitly initialize llvm-project and rustc_watt_runtime as they are critical and might be missed
git submodule update --init --recursive

# Create a linker wrapper script
# This is necessary because we need to inject specific flags for the wasm32-wasip1-threads target
# and handle linker arguments that clang might not pass through correctly without -Wl,
WRAPPER_PATH="${PROJECT_ROOT}/wasi-wrapper.sh"
cat > "$WRAPPER_PATH" <<EOF
#!/bin/bash
# Wrapper for clang to act as a linker for Rust wasm32-wasip1-threads target

args=()
skip_next=false
prefix_next=""

for arg in "\$@"; do
    if [ "\$skip_next" = true ]; then
        skip_next=false
        continue
    fi
    if [ -n "\$prefix_next" ]; then
        args+=("-Wl,\$prefix_next,\$arg")
        prefix_next=""
        continue
    fi
    
    case "\$arg" in
        -flavor)
            # Rust passes -flavor wasm, but we are calling clang, not lld directly.
            skip_next=true
            ;;
        --export|--undefined)
            # Next arg is the symbol name
            prefix_next="\$arg"
            ;;
        --export=*|--undefined=*|--stack-first|--allow-undefined|--no-demangle|--import-memory|--export-memory|--shared-memory|--max-memory=*|--gc-sections|--strip-all|--no-entry|--import-table)
            args+=("-Wl,\$arg")
            ;;
        -O*)
            # Linker optimization level - pass through
            args+=("-Wl,\$arg")
            ;;
        *)
            args+=("\$arg")
            ;;
    esac
done

exec "${WASI_CLANG}" \\
    --target=wasm32-wasip1-threads \\
    --sysroot="${WASI_SYSROOT}" \\
    -lwasi-emulated-signal \\
    -lwasi-emulated-process-clocks \\
    -lwasi-emulated-mman \\
    "\${args[@]}"
EOF

chmod +x "$WRAPPER_PATH"
echo "Created linker wrapper at $WRAPPER_PATH"

# Create config.toml for the build
CONFIG_PATH="${PROJECT_ROOT}/config.wasm-compiler.toml"
cat > "$CONFIG_PATH" <<EOF
# Includes one of the default files in src/bootstrap/defaults
profile = "compiler"
change-id = 123456

[rust]
codegen-backends = ["llvm"]
# Disable debug assertions and info for speed/size if desired, but user config had them off/default
debug = false
debuginfo-level = 0
strip = true
channel = "nightly" 
# Use LLD if possible (WASI SDK uses lld)
lld = true

[llvm]
# IMPORTANT: Only build WebAssembly backend.
# This ensures the final binary does not contain native instruction support.
targets = "WebAssembly"
static-libstdcpp = true
link-shared = false
optimize = true
# Flags for WASI environment compatibility (needed when LLVM runs on Wasm)
cflags = "-D_WASI_EMULATED_SIGNAL -D_WASI_EMULATED_PROCESS_CLOCKS"
cxxflags = "-D_WASI_EMULATED_SIGNAL -D_WASI_EMULATED_PROCESS_CLOCKS"

[build]
# The host compiler (Stage 1) runs on the build machine (Linux).
# The final compiler (Stage 2) will run on Wasm.
# However, x.py 'host' setting usually dictates the targets to build compilers that RUN on.
# If we set host = ["wasm32-wasip1-threads"], x.py attempts to build a compiler that runs on that target.
host = ["wasm32-wasip1-threads"]

# The compiler we build should be able to compile for Wasm.
target = ["wasm32-wasip1-threads"]

# Use the local cargo/rustc for bootstrapping
cargo-native-static = true

[target.wasm32-wasip1-threads]
wasi-root = "${WASI_SYSROOT}"
linker = "${WRAPPER_PATH}"
codegen-backends = ["llvm"]

# Configure native target (needed for stage0/stage1 to run)
[target.${HOST_ARCH}-unknown-linux-gnu]
cc = "gcc"
cxx = "g++"
EOF

echo "Created build configuration at $CONFIG_PATH"
echo "Starting build..."

# Run the build
# We use -j $(nproc) to maximize parallelism
./x.py build compiler --config "$CONFIG_PATH"
