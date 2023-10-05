# W.A.T.E.R.: WebAssembly Transport Executable Reactor -- Rust
[![License](https://img.shields.io/badge/License-Apache_2.0-yellowgreen.svg)](https://opensource.org/licenses/Apache-2.0) [![Build Status](https://github.com/erikziyunchi/WASMable-Transport/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/erikziyunchi/WASMable-Transport/actions/workflows/rust.yml)

WASMable Transport: Yet another more Pluggable Pluggable Transport
> Here is the [repo](https://github.com/erikziyunchi/wasm_proxy) contains all the PoC examples, and this framework is the conprehensive result after all the research.

The repo will contain 2 parts of purposes:
1. A cli tool where can directly load a `.wasm` binary proxy and run it (with our WASM development guidence).
2. A library can be used directly to integrate / coporate `.wasm` binary proxy.

## Designs

### What is needed for the library / cli
#### The Host side:
**Config**: calls main program / some library entry func -> use claps to get args (making use of some `Args` struct) -> then convert it to a `WATERConfig` struct.

**execute**: 
1. wasmtime runtime creation
2. Setup env:
    1. memory initialiation & limitation
    2. (optional for now) wasm_config sharing to WASM
    3. export helper functions (e.g. creation of TCP, TLS, crypto, etc)
3. (optional) setup multi-threading
4. Run the `entry_fn`

#### The WASM side:
1. get version
2. load config_wasm
3. start netwroking with imported funcions from Host

## How to run?
To run the Host program + WASM:
```shell
cargo run --bin wasmable_transport -- --wasm-path <./proxy.wasm> --entry-fn <main> --config-wasm <conf.json>
```

Then you can netcat into the connection, for now, I included a `proxy.wasm` as a multiple conneciton echo server, test with several terminals:
```shell
nc 127.0.0.1 9005
```
you should see `> CONNECTED` in the terminal of running WASM, then you can connect a bunch like this and input anything to see how it echos.

## TODOs
- [ ] wasm_config sharing implementation 
- [ ] Generalize Host export TCP listener helper function
- [ ] Host export TCP connect helper function
- [ ] Host export TLS helper function (with decoupled connection & packaging)
