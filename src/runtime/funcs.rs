use crate::runtime::*;

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

pub fn export_tcplistener_create(linker: &mut Linker<Host>) {
    linker.func_wrap("env", "create_listen", move |mut caller: Caller<'_, Host>, ptr: u32, size: u32| -> i32{
        
        let memory = match caller.get_export("memory") {
            Some(Extern::Memory(memory)) => memory,
            _ => return -1,
        };

        // Get a slice of the memory.
        let mem_slice = memory.data_mut(&mut caller);

        // Use the offset and size to get the relevant part of the memory.
        // TODO: use data here to get the filename, ip:port for creating config
        let data = &mut mem_slice[ptr as usize..(ptr as usize + size as usize)];
        // println!("Data: {:?}", data);

        // FIXME: currently hardcoded config here
        let test_file = File::Listen(ListenFile::Tcp {
            name: "LISTEN".try_into().unwrap(),
            port: 9005,
            addr: "127.0.0.1".into()
        });

        // Get the pair here addr:port
        let (addr, port) = match test_file {
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

        // FIXME: currently hardcoded the fd mapped into WASM -- need to configed by WASM if there are multiple connections later
        let wanted_fd = 3;
        
        let socket_fd: usize = (wanted_fd).try_into().unwrap();

        // Get the WasiCtx of the caller(WASM), then insert_file into it
        let mut ctx: &mut WasiCtx = caller.data_mut().preview1_ctx.as_mut().unwrap();
        ctx.insert_file(socket_fd as u32, socket_file, FileAccessMode::all());

        socket_fd as i32
    }).unwrap();
}