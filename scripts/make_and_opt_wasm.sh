#!/bin/bash

# Check if the required command line arguments are provided
if [ $# -ne 2 ]; then
  echo "Usage: $0 <examples/water_bins/<???>> <wasm_name>"
  exit 1
fi

# Change directory to the source directory
cd "./examples/water_bins/$1" || exit 1

# Build the project using cargo
cargo build --target wasm32-wasi || exit 1

# Optimize the generated wasm file
wasm-opt --strip-debug "../../../target/wasm32-wasi/debug/$2.wasm" -o "./$2.wasm" || exit 1

# Copy the optimized wasm file to the destination directory
cp "./$2.wasm" "../../../tests/test_wasm/" || exit 1

# Change directory back to the beginning directory
cd - || exit 1

echo "Wasm file $2.wasm successfully built, optimized, and copied to ./tests/test_wasm/"
