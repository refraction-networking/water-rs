//! Exported functions implementation for v1_preview WATM module from the Host

use anyhow::Ok;

use crate::config::wasm_shared_config::StreamConfig;
use crate::runtime::*;
use std::convert::TryInto;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

/// This function is exporting the `connect_tcp(ptr: u32, size:u32) -> i32`
/// to the WATM where it is used to create a tcp connection and returns the fd of the connection used by Dialer & Relay.
pub fn export_tcp_connect(linker: &mut Linker<Host>) -> Result<(), anyhow::Error> {
    linker
        .func_wrap(
            "env",
            "connect_tcp",
            move |mut caller: Caller<'_, Host>, ptr: u32, size: u32| -> i32 {
                info!("[WASM] invoking Host exported Dial func connect_tcp...");

                let memory = match caller.get_export("memory") {
                    Some(Extern::Memory(memory)) => memory,
                    _ => return -1,
                };

                // Get a slice of the memory.
                let mem_slice = memory.data_mut(&mut caller);

                // Use the offset and size to get the relevant part of the memory.
                let data = &mut mem_slice[ptr as usize..(ptr as usize + size as usize)];

                let config: StreamConfig =
                    bincode::deserialize(data).expect("Failed to deserialize");

                let connect_file = File::Connect(ConnectFile::Tcp {
                    name: Some(config.name.clone().try_into().unwrap()),
                    port: config.port as u16,
                    host: config.addr.clone(),
                });

                // Get the pair here addr:port
                let (host, port) = match connect_file {
                    File::Connect(listen_file) => match listen_file {
                        ConnectFile::Tcp { host, port, .. }
                        | ConnectFile::Tls { host, port, .. } => (host, port),
                    },
                    _ => ("Wrong".into(), 0),
                };

                let tcp = match (host.as_str(), port) {
                    ("localhost", port) => std::net::TcpStream::connect(SocketAddr::V4(
                        SocketAddrV4::new(Ipv4Addr::LOCALHOST, port),
                    )),
                    addr => std::net::TcpStream::connect(addr),
                }
                .map(TcpStream::from_std)
                .context(format!(
                    "Failed to connect to {}:{} in Host exported dial",
                    host, port
                ))
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

/// This function is exporting the `create_listen(ptr: u32, size: u32) -> i32`
/// to the WATM where it is used to create a tcp listener and returns the fd of the listener used by Listener & Relay.
pub fn export_tcplistener_create(linker: &mut Linker<Host>) -> Result<(), anyhow::Error> {
    linker
        .func_wrap(
            "env",
            "create_listen",
            move |mut caller: Caller<'_, Host>, ptr: u32, size: u32| -> i32 {
                info!("[WASM] invoking Host exported Dial func create_tcp_listener...");

                let memory = match caller.get_export("memory") {
                    Some(Extern::Memory(memory)) => memory,
                    _ => return -1,
                };

                // Get a slice of the memory.
                let mem_slice = memory.data_mut(&mut caller);

                // Use the offset and size to get the relevant part of the memory.
                let data = &mut mem_slice[ptr as usize..(ptr as usize + size as usize)];

                let config: StreamConfig =
                    bincode::deserialize(data).expect("Failed to deserialize");

                let listener_file = File::Listen(ListenFile::Tcp {
                    name: config.name.clone().try_into().unwrap(),
                    port: config.port as u16,
                    addr: config.addr.clone(),
                });

                // Get the pair here addr:port
                let (addr, port) = match listener_file {
                    File::Listen(listen_file) => match listen_file {
                        ListenFile::Tcp { addr, port, .. } | ListenFile::Tls { addr, port, .. } => {
                            (addr, port)
                        }
                    },
                    _ => ("Wrong".into(), 0),
                };

                // Creating Tcp Listener
                let tcp = std::net::TcpListener::bind((addr.as_str(), port)).unwrap();
                let tcp = TcpListener::from_std(tcp);
                // tcp.set_nonblocking(true);
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
