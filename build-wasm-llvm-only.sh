#!/usr/bin/env bash
#
# Build stage1 with native LLVM targets, then rebuild LLVM with only
# WebAssembly enabled for the final stage2 rustc.
#
# This keeps the native LLVM targets available for bootstrap while ensuring
# the final compiler binary links against a wasm-only LLVM.

set -Euo pipefail

ROOT_DIR="$(git rev-parse --show-toplevel)"
BASE_CONFIG="${ROOT_DIR}/config.llvm.toml"
WASM_ONLY_CONFIG="${ROOT_DIR}/config.llvm-wasm-only.toml"

if [[ ! -f "${BASE_CONFIG}" ]]; then
    echo "Missing ${BASE_CONFIG}"
    exit 1
fi

if [[ ! -f "${WASM_ONLY_CONFIG}" ]]; then
    echo "Missing ${WASM_ONLY_CONFIG}"
    exit 1
fi

echo "==> Stage 1: build rustc with native LLVM targets"
"${ROOT_DIR}/x.py" build --config "${BASE_CONFIG}" --stage 1 compiler/rustc

echo "==> Stage 2: rebuild LLVM with wasm-only targets and build final rustc"
"${ROOT_DIR}/x.py" build \
    --config "${WASM_ONLY_CONFIG}" \
    --keep-stage 1 \
    --keep-stage-std 1 \
    --stage 2 \
    compiler/rustc
