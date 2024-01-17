#!/bin/bash
set -e  # Exit on any error

members=(
    "crates/watm"
    "crates/watm_v0"
    "examples/water_bins/ss_client_wasm_v1"
    "examples/water_bins/echo_client"
    "examples/water_bins/plain_v0"
    "examples/water_bins/reverse_v0"
)

for member in "${members[@]}"; do
    pushd $member
    cargo build --verbose --target wasm32-wasi
    popd
done