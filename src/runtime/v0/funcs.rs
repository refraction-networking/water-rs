use crate::runtime::*;
use crate::config::wasm_shared_config::StreamConfig;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

// TODO: rename this to dial_v1, since it has the ability to let WASM choose ip:port
pub fn export_tcp_connect(linker: &mut Linker<Host>) {
    linker.func_wrap("env", "connect_tcp", move |mut caller: Caller<'_, Host>, ptr: u32, size: u32| -> i32{

        info!("[WASM] invoking Host exported Dial func connect_tcp...");
        
        let memory = match caller.get_export("memory") {
            Some(Extern::Memory(memory)) => memory,
            _ => return -1,
        };

        // Get a slice of the memory.
        let mem_slice = memory.data_mut(&mut caller);

        // Use the offset and size to get the relevant part of the memory.
        let data = &mut mem_slice[ptr as usize..(ptr as usize + size as usize)];
        
        let mut config: StreamConfig;
        unsafe {
            config = bincode::deserialize(&data).expect("Failed to deserialize");
        }

        let connect_file = File::Connect(ConnectFile::Tcp {
            name: Some(config.name.clone().try_into().unwrap()),
            port: config.port as u16,
            host: config.addr.clone().into()
        });

        // Get the pair here addr:port
        let (host, port) = match connect_file {
            File::Connect(listen_file) => match listen_file {
                ConnectFile::Tcp { host, port, .. } | ConnectFile::Tls { host, port, .. } => (host, port),
            },
            _ => { ("Wrong".into(), 0) }
        };

        let tcp = match (host.as_str(), port) {
                ("localhost", port) => std::net::TcpStream::connect(SocketAddr::V4(SocketAddrV4::new(
                    Ipv4Addr::LOCALHOST,
                    port,
                ))),
                addr => std::net::TcpStream::connect(addr),
            }
            .map(TcpStream::from_std)
            .context("failed to connect to endpoint").unwrap();
    
        // Connecting Tcp
        let socket_file: Box<dyn WasiFile>  = wasmtime_wasi::net::Socket::from(tcp).into();

        // Get the WasiCtx of the caller(WASM), then insert_file into it
        let ctx: &mut WasiCtx = caller.data_mut().preview1_ctx.as_mut().unwrap();
        ctx.push_file(socket_file, FileAccessMode::all()).unwrap() as i32
    }).unwrap();
}

// TODO: rename this to dial_v1, since it has the ability to let WASM listen on a TcpListener
pub fn export_tcplistener_create(linker: &mut Linker<Host>) {
    linker.func_wrap("env", "create_listen", move |mut caller: Caller<'_, Host>, ptr: u32, size: u32| -> i32{

        info!("[WASM] invoking Host exported Dial func create_tcp_listener...");
        
        let memory = match caller.get_export("memory") {
            Some(Extern::Memory(memory)) => memory,
            _ => return -1,
        };

        // Get a slice of the memory.
        let mem_slice = memory.data_mut(&mut caller);

        // Use the offset and size to get the relevant part of the memory.
        let data = &mut mem_slice[ptr as usize..(ptr as usize + size as usize)];
        
        let mut config: StreamConfig;
        unsafe {
            config = bincode::deserialize(&data).expect("Failed to deserialize");
        }

        let listener_file = File::Listen(ListenFile::Tcp {
            name: config.name.clone().try_into().unwrap(),
            port: config.port as u16,
            addr: config.addr.clone().into()
        });

        // Get the pair here addr:port
        let (addr, port) = match listener_file {
            File::Listen(listen_file) => match listen_file {
                ListenFile::Tcp { addr, port, .. } | ListenFile::Tls { addr, port, .. } => (addr, port),
            },
            _ => { ("Wrong".into(), 0) }
        };

        // Creating Tcp Listener
        let tcp = std::net::TcpListener::bind((addr.as_str(), port)).unwrap();
        let tcp = TcpListener::from_std(tcp);
        tcp.set_nonblocking(true);
        let socket_file: Box<dyn WasiFile> = wasmtime_wasi::net::Socket::from(tcp).into();

        // Get the WasiCtx of the caller(WASM), then insert_file into it
        let ctx: &mut WasiCtx = caller.data_mut().preview1_ctx.as_mut().unwrap();
        ctx.push_file(socket_file, FileAccessMode::all()).unwrap() as i32
    }).unwrap();
}

// Generically link dial functions
// pub fn linkDialFuns(linker: &mut Linker<Host>) {
//     let network = vec!["tcplistener", "tlslistener", "udp"];

//     for net in &network {
//         match linker.func_wrap("env", &format!("connect_{}", net), move |mut caller: Caller<'_, Host>, ptr: u32, size: u32| -> i32{
//             // TODO: get addr from WASM

//             let socket_fd = dialer.Dial(net, addr).unwrap();
//             socket_fd
//         }) {
//             Ok(_) => {},
//             Err(e) => { eprintln!("Failed to define function: {}", e) },
//         };
//     }
// }