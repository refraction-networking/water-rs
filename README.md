# W.A.T.E.R.: WebAssembly Transport Executable Runtime
[![License](https://img.shields.io/badge/License-Apache_2.0-yellowgreen.svg)](https://opensource.org/licenses/Apache-2.0) [![Build Status](https://github.com/erikziyunchi/WASMable-Transport/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/erikziyunchi/WASMable-Transport/actions/workflows/rust.yml)

> Here is the [repo](https://github.com/erikziyunchi/wasm_proxy) contains all the other WASM PoC examples has explored, and this library is the conprehensive result after all the research.

The repo contains 2 main parts for the library:
1. A Rust crate [`water`](https://github.com/erikziyunchi/water-rs/tree/main/crates/water) for Host-development where developers can use to interact with their `.wasm` binary
2. A Rust crate [`water-wasm-crate`](https://github.com/erikziyunchi/water-rs/tree/main/crates/wasm) for WATM-development where developers can make their own `.wasm` binary easier.

Also include examples for demonstration of usage:
1. A cli tool where can directly load a `.wasm` binary and run it, see [here](https://github.com/erikziyunchi/water-rs/tree/main/examples/clients/cli).
2. Some WATM examples implemented using our `water-wasm-crate`, see [here](https://github.com/erikziyunchi/water-rs/tree/main/examples/water_bins).
3. Examples of using the above WATM examples with our `water` library, see [tests](https://github.com/erikziyunchi/water-rs/tree/main/tests/tests) for usage.

## Running tests

```sh
# runs ALL tests
cargo test --workspace --verbose

# run tests for a single crate
cargo test -p <crate_name> --verbose

# run a single test (or test matching name prefix) in a single crate
cargo test -p <crate_name> --verbose -- <test_name>
```