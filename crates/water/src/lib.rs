extern crate anyhow;
extern crate cap_std;
extern crate serde;
extern crate tracing;
extern crate wasi_common;
extern crate wasmtime;
extern crate wasmtime_wasi;
extern crate wasmtime_wasi_threads;

pub mod config;
pub mod errors;
pub mod globals;
pub mod runtime;
pub mod utils;

#[cfg(test)]
mod tests {
    #[test]
    fn water_runtime_test() {
        assert_eq!(1, 1);
    }
}
