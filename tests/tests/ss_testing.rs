//! This is the test file for testing the ss_client_wasm.wasm which is a v1_preview ShadowSocks WATM module,
//! program procedures here can also be treat as examples of using the WATER client, with the ShadowSocks protocol WATM.

#![allow(dead_code)]

use tempfile::tempdir;
use tracing::Level;
use water::*;

use std::thread;
use std::{
    fs::File,
    io::Write,
    net::{IpAddr, SocketAddr, ToSocketAddrs},
    str,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
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

// const SERVER_CONF_STR: &str = r#"
// {
//     "server": "127.0.0.1",
//     "server_port": 8388,
//     "password": "Test!23",
//     "method": "chacha20-ietf-poly1305",
// }
// "#;

/// A test for a normal Shadowsocks client
#[tokio::test]
async fn wasm_managed_shadowsocks_async() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // ==== setup official Shadowsocks server ====
    const SERVER_ADDR: &str = "127.0.0.1:8088";
    const LOCAL_ADDR: &str = "127.0.0.1:8081";

    const PASSWORD: &str = "WATERisAwesome!23";
    const METHOD: CipherKind = CipherKind::CHACHA20_POLY1305;

    let cfg_str = r#"
	{
        "remote_address": "127.0.0.1",
        "remote_port": 8088,
        "local_address": "127.0.0.1",
        "local_port": 8080,
        "password": "WATERisAwesome!23",
        "bypass": false
    }
	"#;
    // Create a directory inside of `std::env::temp_dir()`.
    let dir = tempdir()?;
    let file_path = dir.path().join("temp-config.txt");
    let mut file = File::create(&file_path)?;
    writeln!(file, "{}", cfg_str)?;

    let svr = Socks5TestServer::new(SERVER_ADDR, LOCAL_ADDR, PASSWORD, METHOD, false);
    svr.run().await;

    // ==== setup WASM Shadowsocks client ====
    let conf = config::WATERConfig::init(
        // Source code of ss_client_wasm.wasm:
        // https://github.com/refraction-networking/water-rs/tree/main/examples/water_bins/ss_client_wasm_v1
        //
        String::from("./test_wasm/ss_client_wasm.wasm"),
        String::from("v1_listen"),
        // Currently using a temp file to pass config to WASM client
        // can be easily configed here -- but can also use config.json
        String::from(file_path.to_string_lossy()),
        // String::from("./test_data/config.json"),
        config::WaterBinType::Runner,
        true,
    )
    .unwrap();

    let mut water_client = runtime::client::WATERClient::new(conf).unwrap();

    // ==== spawn a thread to run WASM Shadowsocks client ====
    thread::spawn(move || {
        water_client.execute().unwrap();
    });

    let wasm_ss_client_addr = SocketAddr::new("127.0.0.1".parse().unwrap(), 8080);

    // Give some time for the WASM client to start
    thread::sleep(Duration::from_millis(100));

    // ==== test WASM Shadowsocks client ====
    let mut c = Socks5TcpClient::connect(
        Address::DomainNameAddress("detectportal.firefox.com".to_owned(), 80),
        wasm_ss_client_addr,
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

    Ok(())
}

/// A test for a normal Shadowsocks client but set bypass to true (which means it won't go through a Shadowsocks server act as a plain proxy)
#[tokio::test]
async fn wasm_managed_shadowsocks_bypass_async() -> Result<(), Box<dyn std::error::Error>> {
    let cfg_str = r#"
	{
		"remote_address": "127.0.0.1",
		"remote_port": 10085,
		"local_address": "127.0.0.1",
		"local_port": 10086,
        "password": "Test!23",
        "bypass": true
	}
	"#;
    // Create a directory inside of `std::env::temp_dir()`.
    let dir = tempdir()?;
    let file_path = dir.path().join("temp-config.txt");
    let mut file = File::create(&file_path)?;
    writeln!(file, "{}", cfg_str)?;

    // ==== setup WASM Shadowsocks client ====
    let conf = config::WATERConfig::init(
        String::from("./test_wasm/ss_client_wasm.wasm"),
        String::from("v1_listen"),
        String::from(file_path.to_string_lossy()),
        config::WaterBinType::Runner,
        true,
    )
    .unwrap();

    let mut water_client = runtime::client::WATERClient::new(conf).unwrap();

    // spawn a thread to run WASM Shadowsocks client
    thread::spawn(move || {
        water_client.execute().unwrap();
    });

    // Give some time for the WASM client to start
    thread::sleep(Duration::from_millis(1000));

    let wasm_ss_client_addr = SocketAddr::new("127.0.0.1".parse().unwrap(), 10086);

    // ==== test WASM Shadowsocks client ====
    // currently only support connect by ip,
    // get the ip of detectportal.firefox.com
    let mut addrs = "detectportal.firefox.com:80".to_socket_addrs()?;
    let ip = addrs
        .find(|addr| addr.is_ipv4())
        .ok_or("No IPv4 address found for detectportal.firefox.com")?;

    let ip: IpAddr = ip.ip().to_string().parse().unwrap();
    let port = 80;

    let mut c = Socks5TcpClient::connect(
        Address::SocketAddress(SocketAddr::new(ip, port)),
        wasm_ss_client_addr,
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

    Ok(())
}
