//! This file contains the config related function that will be the same across all versions of WATM

use crate::runtime::*;

/// exportint a function `pull_config() -> i32` that will be used
/// for WATM to get the config file from the host
pub fn export_config(linker: &mut Linker<Host>, config_file: String) -> Result<(), anyhow::Error> {
    linker
        .func_wrap(
            "env",
            "pull_config",
            move |mut caller: Caller<'_, Host>| -> i32 {
                info!("[WASM] invoking Host exported request_config ...");

                // open the config file and insert to WASM
                let dir = Dir::open_ambient_dir(".", ambient_authority())
                    .expect("Error now able to open ambient dir"); // Open the root directory
                let wasi_file = dir
                    .open_with(&config_file, OpenOptions::new().read(true).write(true))
                    .expect("Error now able to open config file");
                let wasi_file = wasmtime_wasi::sync::file::File::from_cap_std(wasi_file);

                let ctx: &mut WasiCtx = caller.data_mut().preview1_ctx.as_mut().unwrap();
                ctx.push_file(Box::new(wasi_file), FileAccessMode::all())
                    .expect("Error with pushing file") as i32
            },
        )
        .context("Failed to export config function to WASM")?;
    Ok(())
}
