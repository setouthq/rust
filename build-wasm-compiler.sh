#!/usr/bin/env bash
# ==============================================================================
# Build Rust Compiler with WebAssembly-Only LLVM Backend
# ==============================================================================
#
# This script builds a Rust compiler (rustc) that includes ONLY the WebAssembly
# LLVM target backend in the final binary. This produces a minimal compiler
# that can only compile Rust to WebAssembly targets.
#
# HOW IT WORKS:
# -------------
# The Rust compiler build process involves multiple stages:
#
#   Stage 0: Uses the pre-existing "bootstrap" compiler (downloaded or from rustup)
#            to build the stage 1 compiler.
#
#   Stage 1: The newly built compiler from stage 0. This compiler is built BY
#            the bootstrap compiler but builds WITH the LLVM we configure.
#
#   Stage 2: The final compiler built BY stage 1. This is the "production" compiler.
#            Both stage 1 and stage 2 will include only the LLVM targets we specify.
#
# KEY INSIGHT:
# ------------
# The `llvm.targets` configuration controls what LLVM backends are included in
# the compiler's LLVM library. This affects what the compiler can TARGET (compile TO),
# not what platform it RUNS ON.
#
# For example, with `llvm.targets = "WebAssembly"`:
#   - The resulting rustc can compile Rust to wasm32-* targets
#   - But rustc itself still runs on your native platform (x86_64, aarch64, etc.)
#   - The bootstrap/stage0 process uses your system's native code
#
# USAGE:
# ------
#   ./build-wasm-compiler.sh [OPTIONS]
#
# OPTIONS:
#   --help, -h           Show this help message
#   --stage STAGE        Build stage (1 or 2, default: 2)
#   --clean              Clean previous build artifacts before building
#   --wasi-sdk PATH      Path to WASI SDK (for wasm32-wasip1 targets)
#   --jobs N             Number of parallel jobs (default: auto)
#   --dry-run            Show what would be done without executing
#   --verbose            Enable verbose output
#   --config PATH        Use a custom config file (default: config.wasm.toml)
#
# PREREQUISITES:
# --------------
#   - A working Rust toolchain (rustup)
#   - CMake, Python 3, and a C/C++ compiler
#   - (Optional) WASI SDK for wasm32-wasip1 targets
#   - (Optional) Ninja for faster builds
#
# EXAMPLES:
# ---------
#   # Basic build (stage 2 compiler)
#   ./build-wasm-compiler.sh
#
#   # Build with WASI SDK
#   ./build-wasm-compiler.sh --wasi-sdk /opt/wasi-sdk-24.0
#
#   # Quick stage 1 build for testing
#   ./build-wasm-compiler.sh --stage 1
#
#   # Clean build with verbose output
#   ./build-wasm-compiler.sh --clean --verbose
#
# OUTPUT:
# -------
# The built compiler will be in: build/<host-triple>/stage<N>/bin/rustc
# ==============================================================================

set -euo pipefail

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Default values
STAGE=2
CLEAN=false
WASI_SDK=""
JOBS=""
DRY_RUN=false
VERBOSE=false
CONFIG_FILE="config.wasm.toml"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

show_help() {
    head -n 60 "$0" | tail -n +2 | sed 's/^# \?//'
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --help|-h)
            show_help
            exit 0
            ;;
        --stage)
            STAGE="$2"
            shift 2
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        --wasi-sdk)
            WASI_SDK="$2"
            shift 2
            ;;
        --jobs)
            JOBS="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --config)
            CONFIG_FILE="$2"
            shift 2
            ;;
        *)
            log_error "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Validate stage
if [[ "$STAGE" != "1" && "$STAGE" != "2" ]]; then
    log_error "Stage must be 1 or 2, got: $STAGE"
    exit 1
fi

# Check we're in the rust repository
if [[ ! -f "$SCRIPT_DIR/x.py" ]]; then
    log_error "This script must be run from the rust repository root"
    exit 1
fi

cd "$SCRIPT_DIR"

# Check for required config file
if [[ ! -f "$CONFIG_FILE" ]]; then
    log_error "Configuration file not found: $CONFIG_FILE"
    log_info "Available config files:"
    ls -1 config*.toml 2>/dev/null || echo "  (none found)"
    exit 1
fi

log_info "Building Rust compiler with WebAssembly-only LLVM backend"
log_info "Using configuration: $CONFIG_FILE"
log_info "Target stage: $STAGE"

# Check for prerequisites
log_info "Checking prerequisites..."

check_command() {
    if ! command -v "$1" &> /dev/null; then
        log_error "Required command not found: $1"
        return 1
    fi
    return 0
}

MISSING_PREREQS=false
check_command python3 || MISSING_PREREQS=true
check_command cmake || MISSING_PREREQS=true
check_command rustc || MISSING_PREREQS=true
check_command cargo || MISSING_PREREQS=true

if [[ "$MISSING_PREREQS" == "true" ]]; then
    log_error "Please install missing prerequisites and try again"
    exit 1
fi

# Check for C/C++ compiler
if ! (check_command cc || check_command gcc || check_command clang); then
    log_error "No C compiler found (cc, gcc, or clang)"
    exit 1
fi

if ! (check_command c++ || check_command g++ || check_command clang++); then
    log_error "No C++ compiler found (c++, g++, or clang++)"
    exit 1
fi

log_success "Prerequisites check passed"

# Show current LLVM targets setting
LLVM_TARGETS=$(grep -E "^targets\s*=" "$CONFIG_FILE" | head -1 | cut -d'"' -f2 || echo "not found")
log_info "LLVM targets to be built: $LLVM_TARGETS"

# Setup config.toml
if [[ -f "config.toml" ]]; then
    log_warn "Existing config.toml found, backing up to config.toml.bak"
    cp config.toml config.toml.bak
fi

log_info "Copying $CONFIG_FILE to config.toml..."
if [[ "$DRY_RUN" == "false" ]]; then
    cp "$CONFIG_FILE" config.toml
fi

# Handle WASI SDK configuration if provided
if [[ -n "$WASI_SDK" ]]; then
    if [[ ! -d "$WASI_SDK" ]]; then
        log_error "WASI SDK directory not found: $WASI_SDK"
        exit 1
    fi

    log_info "Configuring WASI SDK from: $WASI_SDK"
    
    WASI_SYSROOT="$WASI_SDK/share/wasi-sysroot"
    WASI_CLANG="$WASI_SDK/bin/clang"

    if [[ ! -d "$WASI_SYSROOT" ]]; then
        log_error "WASI sysroot not found at: $WASI_SYSROOT"
        exit 1
    fi

    if [[ ! -x "$WASI_CLANG" ]]; then
        log_error "WASI clang not found at: $WASI_CLANG"
        exit 1
    fi

    if [[ "$DRY_RUN" == "false" ]]; then
        # Update config.toml with WASI SDK paths
        sed -i "s|#wasi-root = \"/path/to/wasi-sdk/share/wasi-sysroot\"|wasi-root = \"$WASI_SYSROOT\"|g" config.toml
        sed -i "s|#linker = \"/path/to/wasi-sdk/bin/clang\"|linker = \"$WASI_CLANG\"|g" config.toml
    fi

    log_success "WASI SDK configured"
fi

# Build options
BUILD_OPTS=""

if [[ -n "$JOBS" ]]; then
    BUILD_OPTS="$BUILD_OPTS --jobs $JOBS"
fi

if [[ "$VERBOSE" == "true" ]]; then
    BUILD_OPTS="$BUILD_OPTS -v"
fi

# Clean if requested
if [[ "$CLEAN" == "true" ]]; then
    log_info "Cleaning previous build artifacts..."
    if [[ "$DRY_RUN" == "false" ]]; then
        ./x.py clean
    else
        log_info "[DRY-RUN] Would run: ./x.py clean"
    fi
fi

# Perform the build
log_info "Starting build (this may take a while)..."
log_info "Build command: ./x.py build --stage $STAGE compiler $BUILD_OPTS"

if [[ "$DRY_RUN" == "true" ]]; then
    log_info "[DRY-RUN] Would run: ./x.py build --stage $STAGE compiler $BUILD_OPTS"
    log_info "[DRY-RUN] Build completed (dry run mode)"
    exit 0
fi

# Run the actual build
if ./x.py build --stage "$STAGE" compiler $BUILD_OPTS; then
    log_success "Build completed successfully!"
else
    log_error "Build failed"
    exit 1
fi

# Find and display the built compiler
HOST_TRIPLE=$(rustc -vV | grep host | cut -d' ' -f2)
RUSTC_PATH="build/$HOST_TRIPLE/stage$STAGE/bin/rustc"

if [[ -f "$RUSTC_PATH" ]]; then
    log_success "Compiler built at: $RUSTC_PATH"
    
    # Show compiler info
    log_info "Compiler version:"
    "$RUSTC_PATH" --version
    
    # Check binary size
    RUSTC_SIZE=$(du -h "$RUSTC_PATH" | cut -f1)
    log_info "Binary size: $RUSTC_SIZE"
    
    # Show available targets
    log_info "Available targets in built compiler:"
    "$RUSTC_PATH" --print target-list 2>/dev/null | grep -E "^wasm" || log_warn "No wasm targets found in target-list"
else
    log_warn "Expected compiler not found at: $RUSTC_PATH"
    log_info "Searching for built rustc..."
    find build -name 'rustc' -type f -executable 2>/dev/null | head -5
fi

log_info ""
log_info "To use the new compiler, add to your PATH:"
log_info "  export PATH=\"$SCRIPT_DIR/build/$HOST_TRIPLE/stage$STAGE/bin:\$PATH\""
log_info ""
log_info "Or compile directly with:"
log_info "  $RUSTC_PATH --target wasm32-unknown-unknown your_file.rs"
