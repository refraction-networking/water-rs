# WATM Examples

This folder is for all the WATM examples that developed using the [WATM library crate](https://github.com/erikziyunchi/water-rs/tree/main/crates/wasm), and runnable with the [WATER library engine](https://github.com/erikziyunchi/water-rs/tree/main/crates/water).

One can find details of these in each examples' README.

---

These WATM examples can be compiled to WASM and optimized with the script I've provided in `./scripts/make_and_opt_wasm.sh` as following:

For example, if you want to make the `ss_client_wasm`, you can run this command in the root directory of this repo:
```shell
sh ./script/make_and_opt_wasm.sh ss_client_wasm_v1 ss_client_wasm
```
which is:
```shell
sh ./script/make_and_opt_wasm.sh ./examples/water_bins/<folder-name> <the-wasm-binary-name.wasm>
```