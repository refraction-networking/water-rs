use water::*;

// use rand;
// use pprof::protos::Message;
// use tracing::info;
use futures::future::{self, Either};
use shadowsocks_rust::{EXIT_CODE_SERVER_ABORTED, EXIT_CODE_SERVER_EXIT_UNEXPECTEDLY};
use tracing::Level;

use std::process::ExitCode;
use std::thread;
use std::{
    net::{SocketAddr, ToSocketAddrs},
    str,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    runtime::Builder,
    sync::oneshot,
    time::{self, Duration},
};

use shadowsocks_service::{
    config::{
        Config, ConfigType, LocalConfig, LocalInstanceConfig, ProtocolType, ServerInstanceConfig,
    },
    local::socks::client::socks5::Socks5TcpClient,
    run_local, run_server,
    shadowsocks::{
        config::{Mode, ServerAddr, ServerConfig},
        crypto::CipherKind,
        relay::socks5::Address,
    },
};

// use std::net::{TcpListener, TcpStream};
// use std::sync::atomic::{AtomicBool, Ordering};
// use std::sync::Arc;
// use std::time::Instant;
// use std::io::{Read, Write, ErrorKind};
// use std::thread::sleep;
// use std::time::Duration;

#[tokio::test]
async fn socks5_relay_aead() {
    const SERVER_ADDR: &str = "127.0.0.1:8110";
    const LOCAL_ADDR: &str = "127.0.0.1:8210";

    const PASSWORD: &str = "test-password";
    const METHOD: CipherKind = CipherKind::AES_256_GCM;

    let svr = Socks5TestServer::new(SERVER_ADDR, LOCAL_ADDR, PASSWORD, METHOD, false);
    svr.run().await;

    let mut c = Socks5TcpClient::connect(
        Address::DomainNameAddress("detectportal.firefox.com".to_owned(), 80),
        svr.client_addr(),
    )
    .await
    .unwrap();

    let req = b"GET /success.txt HTTP/1.0\r\nHost: detectportal.firefox.com\r\nAccept: */*\r\n\r\n";
    c.write_all(req).await.unwrap();
    c.flush().await.unwrap();

    let mut r = BufReader::new(c);

    let mut buf = Vec::new();
    r.read_until(b'\n', &mut buf).await.unwrap();

    let http_status = b"HTTP/1.0 200 OK\r\n";
    assert!(buf.starts_with(http_status));
}

pub struct Socks5TestServer {
    local_addr: SocketAddr,
    svr_config: Config,
    cli_config: Config,
}

impl Socks5TestServer {
    pub fn new<S, L>(
        svr_addr: S,
        local_addr: L,
        pwd: &str,
        method: CipherKind,
        enable_udp: bool,
    ) -> Socks5TestServer
    where
        S: ToSocketAddrs,
        L: ToSocketAddrs,
    {
        let svr_addr = svr_addr.to_socket_addrs().unwrap().next().unwrap();
        let local_addr = local_addr.to_socket_addrs().unwrap().next().unwrap();

        Socks5TestServer {
            local_addr,
            svr_config: {
                let mut cfg = Config::new(ConfigType::Server);
                cfg.server = vec![ServerInstanceConfig::with_server_config(ServerConfig::new(
                    svr_addr,
                    pwd.to_owned(),
                    method,
                ))];
                cfg.server[0].config.set_mode(if enable_udp {
                    Mode::TcpAndUdp
                } else {
                    Mode::TcpOnly
                });
                cfg
            },
            cli_config: {
                let mut cfg = Config::new(ConfigType::Local);
                cfg.local = vec![LocalInstanceConfig::with_local_config(
                    LocalConfig::new_with_addr(ServerAddr::from(local_addr), ProtocolType::Socks),
                )];
                cfg.local[0].config.mode = if enable_udp {
                    Mode::TcpAndUdp
                } else {
                    Mode::TcpOnly
                };
                cfg.server = vec![ServerInstanceConfig::with_server_config(ServerConfig::new(
                    svr_addr,
                    pwd.to_owned(),
                    method,
                ))];
                cfg
            },
        }
    }

    pub fn client_addr(&self) -> &SocketAddr {
        &self.local_addr
    }

    pub async fn run(&self) {
        let svr_cfg = self.svr_config.clone();
        tokio::spawn(run_server(svr_cfg));

        let client_cfg = self.cli_config.clone();
        tokio::spawn(run_local(client_cfg));

        time::sleep(Duration::from_secs(1)).await;
    }
}

const SERVER_CONF_STR: &str = r#"
{
    "server": "127.0.0.1",
    "server_port": 8388,
    "password": "Test!23",
    "method": "chacha20-ietf-poly1305",
}
"#;

#[tokio::test]
async fn wasm_managed_shadowsocks_async() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // let (cancel, is_canceled) = oneshot::channel::<()>();
    // thread::spawn(move || {
    //     let server_config = Config::load_from_str(SERVER_CONF_STR, ConfigType::Server).unwrap();
    //     let mut rt_builder = Builder::new_current_thread();
    //     let rt = rt_builder
    //         .enable_all()
    //         .build()
    //         .expect("create tokio Runtime");
    //     rt.block_on(
    //         // async move {
    //         //     server::run(server_config).await.unwrap();
    //         // }
    //         async move {
    //             let server = run_server(server_config);
    //             tokio::pin!(is_canceled);
    //             tokio::pin!(server);

    //             match future::select(server, is_canceled).await {
    //                 // Server future resolved without an error. This should never happen.
    //                 Either::Left((Ok(..), ..)) => {
    //                     eprintln!("server exited unexpectedly");
    //                     EXIT_CODE_SERVER_EXIT_UNEXPECTEDLY.into()
    //                 }
    //                 // Server future resolved with error, which are listener errors in most cases
    //                 Either::Left((Err(err), ..)) => {
    //                     eprintln!("server aborted with {err}");
    //                     EXIT_CODE_SERVER_ABORTED.into()
    //                 }
    //                 // The abort signal future resolved. Means we should just exit.
    //                 Either::Right(_) => ExitCode::SUCCESS,
    //             }
    //         },
    //     );
    // });

    const SERVER_ADDR: &str = "127.0.0.1:8388";
    const LOCAL_ADDR: &str = "127.0.0.1:8080";

    const PASSWORD: &str = "Test!23";
    const METHOD: CipherKind = CipherKind::CHACHA20_POLY1305;

    let svr = Socks5TestServer::new(SERVER_ADDR, LOCAL_ADDR, PASSWORD, METHOD, false);
    svr.run().await;

    let conf = config::WATERConfig::init(
        String::from("./test_wasm/ss_client_wasm.wasm"),
        String::from("v1_listen"),
        String::from("./test_data/config.json"),
        2,
        true,
    )
    .unwrap();

    thread::spawn(move || {
        let mut water_client = runtime::WATERClient::new(conf).unwrap();
        water_client.execute().unwrap();
    });

    // let socket = SocketAddr::new("127.0.0.1".parse().unwrap(), 8080);

    let mut c = Socks5TcpClient::connect(
        Address::DomainNameAddress("detectportal.firefox.com".to_owned(), 80),
        svr.client_addr(),
    )
    .await
    .unwrap();

    let req = b"GET /success.txt HTTP/1.0\r\nHost: detectportal.firefox.com\r\nAccept: */*\r\n\r\n";
    c.write_all(req).await.unwrap();
    c.flush().await.unwrap();

    let mut r = BufReader::new(c);

    let mut buf = Vec::new();
    r.read_until(b'\n', &mut buf).await.unwrap();

    let http_status = b"HTTP/1.0 200 OK\r\n";
    assert!(buf.starts_with(http_status));
}

// #[test]
// fn SS_handler_testing() {
//     tracing_subscriber::fmt()
//         .with_max_level(Level::INFO)
//         .init();

//     let listener = TcpListener::bind("127.0.0.1:1080").expect("Failed to bind to address");
//     println!("Listening on {:?}", listener.local_addr().unwrap());
//     for stream in listener.incoming() {
//         match stream {
//             Ok(client) => {
//                 // handle onely 1 client
//                 handle_client(client);
//             }
//             Err(e) => {
//                 println!("Error accepting client: {}", e);
//             }
//         }
//     }
// }

// this is the test where SOCKS5 server + listener is at the Host -- V0
// #[test]
// fn SS_client_no_socks5() -> Result<(), anyhow::Error> {
//     tracing_subscriber::fmt()
//         .with_max_level(Level::INFO)
//         .init();

//     // --------- start to dial the listener ---------
//     let dial_handle = std::thread::spawn(|| -> Result<(), anyhow::Error> {
//         // Measure initialization time
//         let conf = config::WATERConfig::init(String::from("./tests/test_wasm/proxy.wasm"), String::from("_dial"), String::from("./tests/test_data/config.json"), 0, true)?;
//         let mut water_client = runtime::WATERClient::new(conf)?;
//         water_client.connect("", 0)?;

//         // let mut water_client = TcpStream::connect(("127.0.0.1", 8088))?;

//         // Not measuring the profiler guard initialization since it's unrelated to the read/write ops
//         let guard = pprof::ProfilerGuard::new(100).unwrap();

//         let single_data_size = 1024; // Bytes per iteration
//         let total_iterations = 1;

//         let random_data: Vec<u8> = (0..single_data_size).map(|_| rand::random::<u8>()).collect();

//         let start = Instant::now();
//         for _ in 0..total_iterations {
//             water_client.write(&random_data)?;

//             let mut buf = vec![0; single_data_size];
//             water_client.read(&mut buf)?;
//         }

//         let elapsed_time = start.elapsed().as_secs_f64();
//         let total_data_size_mb = (total_iterations * single_data_size) as f64;
//         let avg_bandwidth = total_data_size_mb / elapsed_time / 1024.0 / 1024.0;

//         info!("avg bandwidth: {:.2} MB/s (N={})", avg_bandwidth, total_iterations);

//         let single_data_size = 1024; // Bytes per iteration
//         let total_iterations = 100;

//         let random_data: Vec<u8> = (0..single_data_size).map(|_| rand::random::<u8>()).collect();

//         let start = Instant::now();
//         for _ in 0..total_iterations {
//             water_client.write(&random_data)?;

//             let mut buf = vec![0; single_data_size];
//             water_client.read(&mut buf)?;
//         }

//         let elapsed_time = start.elapsed().as_secs_f64();
//         let total_data_size_mb = (total_iterations * single_data_size) as f64;
//         let avg_bandwidth = total_data_size_mb / elapsed_time / 1024.0 / 1024.0;

//         info!("avg bandwidth: {:.2} MB/s (N={})", avg_bandwidth, total_iterations);

//         // Stop and report profiler data
//         if let Ok(report) = guard.report().build() {
//             // println!("{:?}", report);
//             // report.flamegraph(std::io::stdout())?;
//             let mut file = std::fs::File::create("flamegraph.svg")?;
//             report.flamegraph(file)?;

//             // let mut file = std::fs::File::create("profile.pb")?;
//             // report.pprof(file)?;
//             let mut file = std::fs::File::create("profile.pb").unwrap();
//             let profile = report.pprof().unwrap();

//             let mut content = Vec::new();
//             // profile.encode(&mut content).unwrap();
//             profile.write_to_vec(&mut content).unwrap();
//             file.write_all(&content).unwrap();
//         }

//         Ok(())
//     });

//     dial_handle.join().expect("Listener thread panicked")?;

//     // // Signal the listener thread to stop
//     // should_stop.store(true, Ordering::Relaxed);

//     Ok(())
// }
