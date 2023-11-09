# W.A.T.E.R.: WebAssembly Transport Executable Runtime
[![License](https://img.shields.io/badge/License-Apache_2.0-yellowgreen.svg)](https://opensource.org/licenses/Apache-2.0) [![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Ferikziyunchi%2Fwater-rs.svg?type=shield&issueType=license)](https://app.fossa.com/projects/git%2Bgithub.com%2Ferikziyunchi%2Fwater-rs?ref=badge_shield&issueType=license) [![Build & Test Status](https://github.com/erikziyunchi/WASMable-Transport/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/erikziyunchi/WASMable-Transport/actions/workflows/rust.yml)

<div style="width: 100%; height = 160px">
    <div style="width: 75%; height: 150px; float: left;"> 
        WATER-rs provides a Rust runtime for WebAssembly Transport Modules(WATM) as a pluggable application-layer transport protocol provider. It is designed to be highly portable and lightweight, allowing for rapidly deployable pluggable transports. While other pluggable transport implementations require a fresh client deployment (and app-store review) to update their protocol WATER allows dynamic delivery of new transports in real time over the network.<br />
        <br />
    </div>
    <div style="margin-left: 80%; height: 150px;"> 
        <img src=".github/assets/logo_v0.svg" alt="WATER wasm transport" align="right">
    </div>
</div>

Information about the Golang Engine can be found in the [water-go](https://github.com/gaukas/water) library, and another [repo](https://github.com/erikziyunchi/wasm_proxy) contains all the other WASM PoC examples has explored.

## Contents

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