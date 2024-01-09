//! Exported functions implementation for v0 WATM module from the Host

use crate::runtime::v0::config::V0Config;
use crate::runtime::*;
use std::sync::{Arc, Mutex};

/// This function is exporting the `host_dial() -> i32`
/// to the WATM where it is used to create a tcp connection and returns the fd of the connection used by Dialer & Relay.
pub fn export_tcp_connect(
    linker: &mut Linker<Host>,
    config: Arc<Mutex<V0Config>>,
) -> Result<(), anyhow::Error> {
    linker
        .func_wrap(
            "env",
            "host_dial",
            move |mut caller: Caller<'_, Host>| -> i32 {
                info!("[WASM] invoking host_dial v0 ...");

                let mut config = config.lock().unwrap();

                let tcp = config
                    .connect()
                    .map(TcpStream::from_std)
                    .context("failed to connect to endpoint")
                    .unwrap();

                // Connecting Tcp
                let socket_file: Box<dyn WasiFile> = wasmtime_wasi::net::Socket::from(tcp).into();

                // Get the WasiCtx of the caller(WASM), then insert_file into it
                let ctx: &mut WasiCtx = caller
                    .data_mut()
                    .preview1_ctx
                    .as_mut()
                    .context("preview1_ctx in Store is None")
                    .unwrap();
                ctx.push_file(socket_file, FileAccessMode::all())
                    .context("Failed to push file into WASM")
                    .unwrap() as i32
            },
        )
        .context("Failed to export Dial function to WASM")?;
    Ok(())
}

/// This function is exporting the `host_accept() -> i32`
/// to the WATM where it is used to accept a incoming connection from the listener and returns the fd of the connection used by Listener & Relay.
pub fn export_accept(
    linker: &mut Linker<Host>,
    config: Arc<Mutex<V0Config>>,
) -> Result<(), anyhow::Error> {
    linker
        .func_wrap(
            "env",
            "host_accept",
            move |mut caller: Caller<'_, Host>| -> i32 {
                info!("[WASM] invoking host_accept v0 ...");

                let mut config = config.lock().unwrap();

                let tcp = config
                    .accept()
                    .map(TcpStream::from_std)
                    .context("failed to accept")
                    .unwrap();

                // Connecting Tcp
                let socket_file: Box<dyn WasiFile> = wasmtime_wasi::net::Socket::from(tcp).into();

                // Get the WasiCtx of the caller(WASM), then insert_file into it
                let ctx: &mut WasiCtx = caller
                    .data_mut()
                    .preview1_ctx
                    .as_mut()
                    .context("preview1_ctx in Store is None")
                    .unwrap();
                ctx.push_file(socket_file, FileAccessMode::all())
                    .context("Failed to push file into WASM")
                    .unwrap() as i32
            },
        )
        .context("Failed to export TcpListener create function to WASM")?;
    Ok(())
}

/// This function is exporting the `host_defer()` to the WATM where it is used to close the connection.
pub fn export_defer(
    linker: &mut Linker<Host>,
    config: Arc<Mutex<V0Config>>,
) -> Result<(), anyhow::Error> {
    linker
        .func_wrap("env", "host_defer", move |_caller: Caller<'_, Host>| {
            info!("[WASM] invoking host_defer v0 ...");

            let mut config = config.lock().unwrap();

            config.defer();
        })
        .context("Failed to export defer function to WASM")?;
    Ok(())
}
