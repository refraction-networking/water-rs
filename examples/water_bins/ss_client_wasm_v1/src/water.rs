use super::*;

use bytes::{BufMut, BytesMut};
use shadowsocks_crypto::v1::openssl_bytes_to_key;
use std::sync::Arc;

#[export_name = "_water_init"]
pub fn _init() {
    // default to have logging enabled
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("[WASM] running in _init");
}

#[export_name = "_water_config"]
pub fn _process_config(fd: i32) {
    info!("[WASM] running in _process_config");

    let mut config_file = unsafe { std::fs::File::from_raw_fd(fd) };
    let mut config = String::new();
    match config_file.read_to_string(&mut config) {
        Ok(_) => {
            let config: SSConfig = match serde_json::from_str(&config) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("[WASM] > _process_config ERROR: {}", e);
                    return;
                }
            };

            let mut global_conn = match CONN.lock() {
                Ok(conn) => conn,
                Err(e) => {
                    eprintln!("[WASM] > ERROR: {}", e);
                    return;
                }
            };

            global_conn.config = config;

            info!("[WASM] > _process_config: {:?}", global_conn.config);
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
    let bypass = match CONN.lock() {
        Ok(conn) => conn.config.bypass,
        Err(e) => {
            eprintln!("[WASM] > ERROR: {}", e);
            return;
        }
    };

    _start_listen(bypass).unwrap();
}

#[tokio::main(flavor = "current_thread")]
async fn _start_listen(bypass: bool) -> std::io::Result<()> {
    let fd = _listener_creation().unwrap();

    // Set up pre-established listening socket.
    let standard = unsafe { std::net::TcpListener::from_raw_fd(fd) };
    standard.set_nonblocking(true).unwrap();

    // Convert to tokio TcpListener.
    let listener = TcpListener::from_std(standard)?;

    // Initialize the variables for the server address and the encryption key
    let mut server_addr: Address = Address::SocketAddress(SocketAddr::from(([127, 0, 0, 1], 8088)));
    let mut enc_key = vec![0u8; CIPHER_METHOD.key_len()].into_boxed_slice();

    {
        let global_conn = match CONN.lock() {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!("[WASM] > ERROR: {}", e);
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "failed to lock CONN",
                ));
            }
        };

        // getting the server ip address
        match IpAddr::from_str(&global_conn.config.remote_address) {
            Ok(ip_addr) => {
                server_addr = Address::SocketAddress(SocketAddr::from((
                    ip_addr,
                    global_conn.config.remote_port as u16,
                )));
                println!("Server address: {}", server_addr);
            }
            Err(e) => {
                eprintln!("Failed to parse IP address: {}", e);
            }
        }

        // getting the enc_key derived from the password
        openssl_bytes_to_key(global_conn.config.password.as_bytes(), &mut enc_key);
    }

    // Create a Arc for the encryption key
    let enc_key = Arc::new(enc_key);

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

        // Clone server_addr(will be changed) for each iteration of the loop.
        let server_addr_clone = server_addr.clone();

        // Clone the Arc for enc_key to save resources.
        let enc_key_clone = Arc::clone(&enc_key);

        // let enc_key_clone = enc_key.clone();

        // Spawn a background task for each new connection.
        tokio::spawn(async move {
            eprintln!("[WASM] > CONNECTED");
            match _handle_connection(socket, server_addr_clone, &enc_key_clone, bypass).await {
                Ok(()) => eprintln!("[WASM] > DISCONNECTED"),
                Err(e) => eprintln!("[WASM] > ERROR: {}", e),
            }
        });
    }
}

// SS handle incoming connections
async fn _handle_connection(
    stream: TcpStream,
    server_addr: Address,
    key: &[u8],
    bypass: bool,
) -> std::io::Result<()> {
    let mut inbound_con = Socks5Handler::new(stream);
    inbound_con.socks5_greet().await.expect("Failed to greet");

    let target_addr = inbound_con
        .socks5_get_target()
        .await
        .expect("Failed to get target address");

    // if proxied {
    if bypass {
        _connect_bypass(&target_addr, &mut inbound_con).await?;
    } else {
        _connect(target_addr, server_addr, key, &mut inbound_con).await?;
    }

    Ok(())
}

async fn _connect(
    target_addr: Address,
    server_addr: Address,
    key: &[u8],
    inbound_con: &mut Socks5Handler,
) -> std::io::Result<()> {
    let server_stream = _dial_remote(&server_addr).expect("Failed to dial to SS-Server");

    // Constructing the response header
    let mut buf = BytesMut::with_capacity(server_addr.serialized_len());
    buf.put_slice(&[consts::SOCKS5_VERSION, consts::SOCKS5_REPLY_SUCCEEDED, 0x00]);
    server_addr.write_to_buf(&mut buf);

    inbound_con.socks5_response(&mut buf).await;

    // creating the client proxystream -- contains cryptostream with both AsyncRead and AsyncWrite implemented
    let mut proxy = ProxyClientStream::from_stream(server_stream, target_addr, CIPHER_METHOD, key);

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

async fn _connect_bypass(
    target_addr: &Address,
    inbound_con: &mut Socks5Handler,
) -> std::io::Result<()> {
    let mut target_stream = _dial_remote(target_addr).expect("Failed to dial to SS-Server");

    // Constructing the response header
    let mut buf = BytesMut::with_capacity(target_addr.serialized_len());
    buf.put_slice(&[consts::SOCKS5_VERSION, consts::SOCKS5_REPLY_SUCCEEDED, 0x00]);
    target_addr.write_to_buf(&mut buf);

    inbound_con.socks5_response(&mut buf).await;

    match establish_tcp_tunnel_bypassed(&mut inbound_con.stream, &mut target_stream, target_addr)
        .await
    {
        Ok(()) => {
            info!("tcp tunnel (bypassed) closed");
        }
        Err(err) => {
            eprintln!("tcp tunnel (proxied) closed with error: {}", err);
        }
    }

    Ok(())
}

pub fn _dial_remote(target: &Address) -> Result<TcpStream, std::io::Error> {
    let mut tcp_dialer = Dialer::new();

    // TODO: only support ip:port for now, add DNS resolver helper from Host later
    match target {
        Address::SocketAddress(addr) => {
            tcp_dialer.config.remote_address = addr.ip().to_string();
            tcp_dialer.config.remote_port = addr.port() as u32;
        }
        _ => {
            eprintln!("Failed to get target address");
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Failed to get target address",
            ));
        }
    }

    let _tcp_fd: i32 = tcp_dialer.dial().expect("Failed to dial");

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

    info!(
        "[WASM] creating listener at {}:{}",
        global_conn.config.local_address, global_conn.config.local_port
    );

    // NOTE: hardcoded the filename for now, make it a config later
    let stream = StreamConfigV1::init(
        global_conn.config.local_address.clone(),
        global_conn.config.local_port,
        "LISTEN".to_string(),
    );

    let encoded: Vec<u8> = bincode::serialize(&stream).expect("Failed to serialize");

    let address = encoded.as_ptr() as u32;
    let size = encoded.len() as u32;

    let fd = unsafe { create_listen(address, size) };

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
