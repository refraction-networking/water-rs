use crate::runtime::v0::config::V0Config;
use crate::runtime::*;
use std::sync::{Arc, Mutex};

pub fn export_tcp_connect(
    linker: &mut Linker<Host>,
    config: Arc<Mutex<V0Config>>,
) -> Result<(), anyhow::Error> {
    linker
        .func_wrap(
            "env",
            "host_dial",
            move |mut caller: Caller<'_, Host>| -> i32 {
                info!("[WASM] invoking Host exported Dial func connect_tcp...");

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

pub fn export_accept(
    linker: &mut Linker<Host>,
    config: Arc<Mutex<V0Config>>,
) -> Result<(), anyhow::Error> {
    linker
        .func_wrap(
            "env",
            "host_accept",
            move |mut caller: Caller<'_, Host>| -> i32 {
                info!("[WASM] invoking Host exported host_accept func...");

                let mut config = config.lock().unwrap();

                let tcp = config
                    .accept()
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
        .context("Failed to export TcpListener create function to WASM")?;
    Ok(())
}

// TODO: implement this
pub fn export_defer(
    linker: &mut Linker<Host>,
    config: Arc<Mutex<V0Config>>,
) -> Result<(), anyhow::Error> {
    linker
        .func_wrap("env", "host_defer", move |_caller: Caller<'_, Host>| {
            info!("[WASM] invoking Host exported host_defer func...");

            let mut config = config.lock().unwrap();

            config.defer();
        })
        .context("Failed to export defer function to WASM")?;
    Ok(())
}
