use super::*;

use bytes::{BufMut, BytesMut};

#[export_name = "_init"]
pub fn _init(debug: bool) {
    if debug {
        tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    }

    info!("[WASM] running in _init");
}

#[export_name = "_config"]
pub fn _process_config(fd: i32) {
    info!("[WASM] running in _process_config");

    let mut config_file = unsafe { std::fs::File::from_raw_fd(fd) };
    let mut config = String::new();
    match config_file.read_to_string(&mut config) {
        Ok(_) => {
            let config: Config = match serde_json::from_str(&config) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("[WASM] > _process_config ERROR: {}", e);
                    return;
                }
            };

            let mut global_dialer = match DIALER.lock() {
                Ok(dialer) => dialer,
                Err(e) => {
                    eprintln!("[WASM] > ERROR: {}", e);
                    return;
                }
            };

            // global_dialer.file_conn.config = config.clone();
            global_dialer.config = config;
        }
        Err(e) => {
            eprintln!(
                "[WASM] > WASM _process_config falied reading path ERROR: {}",
                e
            );
        }
    };
}

/// WASM Entry point here
#[export_name = "v1_listen"]
fn client_start() {
    _start_listen().unwrap();
}

#[tokio::main(flavor = "current_thread")]
async fn _start_listen() -> std::io::Result<()> {
    let fd = _listener_creation().unwrap();

    // Set up pre-established listening socket.
    let standard = unsafe { std::net::TcpListener::from_raw_fd(fd) };
    standard.set_nonblocking(true).unwrap();

    // Convert to tokio TcpListener.
    let listener = TcpListener::from_std(standard)?;

    info!("[WASM] Starting to listen...");

    loop {
        // Accept new sockets in a loop.
        let socket = match listener.accept().await {
            Ok(s) => s.0,
            Err(e) => {
                eprintln!("[WASM] > ERROR: {}", e);
                continue;
            }
        };

        // Spawn a background task for each new connection.
        tokio::spawn(async move {
            eprintln!("[WASM] > CONNECTED");
            match _handle_connection(socket).await {
                Ok(()) => eprintln!("[WASM] > DISCONNECTED"),
                Err(e) => eprintln!("[WASM] > ERROR: {}", e),
            }
        });
    }
}

// SS handle incoming connections
async fn _handle_connection(stream: TcpStream) -> std::io::Result<()> {
    let mut inbound_con = Socks5Handler::new(stream);
    inbound_con.socks5_greet().await.expect("Failed to greet");

    let target_addr = inbound_con
        .socks5_get_target()
        .await
        .expect("Failed to get target address");
    let server_stream = _dial_server().expect("Failed to dial to SS-Server");

    // FIXME: hardcoded server ip:address for now + only support connection with ip:port
    let server_addr = Address::SocketAddress(SocketAddr::from(([127, 0, 0, 1], 8388)));

    // Constructing the response header
    let mut buf = BytesMut::with_capacity(server_addr.serialized_len());
    buf.put_slice(&[consts::SOCKS5_VERSION, consts::SOCKS5_REPLY_SUCCEEDED, 0x00]);
    server_addr.write_to_buf(&mut buf);

    inbound_con.socks5_response(&mut buf).await;

    // FIXME: hardcoded the key which derived from the password: "Test!23"
    let key = [
        128, 218, 128, 160, 125, 72, 115, 9, 187, 165, 163, 169, 92, 177, 35, 201, 49, 245, 92,
        203, 57, 152, 63, 149, 108, 132, 60, 128, 201, 206, 82, 226,
    ];
    // creating the client proxystream -- contains cryptostream with both AsyncRead and AsyncWrite implemented
    let mut proxy = ProxyClientStream::from_stream(server_stream, target_addr, CIPHER_METHOD, &key);

    match copy_encrypted_bidirectional(CIPHER_METHOD, &mut proxy, &mut inbound_con.stream).await {
        Ok((wn, rn)) => {
            info!(
                "tcp tunnel (proxied) closed, L2R {} bytes, R2L {} bytes",
                rn, wn
            );
        }
        Err(err) => {
            eprintln!("tcp tunnel (proxied) closed with error: {}", err);
        }
    }

    Ok(())
}

pub fn _dial_server() -> Result<TcpStream, std::io::Error> {
    // NOTE: dial to SS-Server
    let mut tcp_dialer = Dialer::new();

    // FIXME: Hardcoded server ip:port for now
    tcp_dialer.config.remote_address = "127.0.0.1".to_string();
    tcp_dialer.config.remote_port = 8388;

    let _tcp_fd = tcp_dialer.dial().expect("Failed to dial");

    let server_stream = match tcp_dialer.file_conn.outbound_conn.file.unwrap() {
        ConnStream::TcpStream(s) => s,
        _ => {
            eprintln!("Failed to get outbound tcp stream");
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Failed to get outbound tcp stream",
            ));
        }
    };

    // NOTE: can convert to a async tokio TcpStream if wanted / needed
    server_stream
        .set_nonblocking(true)
        .expect("Failed to set non-blocking");
    let server_stream =
        TcpStream::from_std(server_stream).expect("Failed to convert to tokio stream");

    info!("[Connected] to SS-Server");

    Ok(server_stream)
}

#[cfg(feature = "direct_connect")]
pub fn _direct_connect() {
    // create a new Dialer to dial any target address as it wants to
    // Add more features later -- connect to target thru rules (direct / server)
    // Connect to target address directly
    {
        let mut tcp_dialer = Dialer::new();
        tcp_dialer.config.remote_address = addr.ip().to_string();
        tcp_dialer.config.remote_port = addr.port() as u32;

        let tcp_fd = tcp_dialer.dial().expect("Failed to dial");

        let server_stream = match tcp_dialer.file_conn.outbound_conn.file.unwrap() {
            ConnStream::TcpStream(s) => s,
            _ => {
                eprintln!("Failed to get outbound tcp stream");
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Failed to get outbound tcp stream",
                ));
            }
        };

        server_stream
            .set_nonblocking(true)
            .expect("Failed to set non-blocking");

        let server_stream =
            TcpStream::from_std(server_stream).expect("Failed to convert to tokio stream");
    }
}

pub fn _listener_creation() -> Result<i32, std::io::Error> {
    let global_conn = match CONN.lock() {
        Ok(conf) => conf,
        Err(e) => {
            eprintln!("[WASM] > ERROR: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "failed to lock config",
            ));
        }
    };

    // FIXME: hardcoded the filename for now, make it a config later
    let stream = StreamConfigV1::init(
        global_conn.config.local_address.clone(),
        global_conn.config.local_port,
        "LISTEN".to_string(),
    );

    let encoded: Vec<u8> = bincode::serialize(&stream).expect("Failed to serialize");

    let address = encoded.as_ptr() as u32;
    let size = encoded.len() as u32;

    let fd = unsafe {
        create_listen(address, size)
    };

    if fd < 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "failed to create listener",
        ));
    }

    info!(
        "[WASM] ready to start listening at {}:{}",
        global_conn.config.local_address, global_conn.config.local_port
    );

    Ok(fd)
}
