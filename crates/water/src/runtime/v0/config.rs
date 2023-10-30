use std::os::fd::{AsRawFd, FromRawFd};

use anyhow::Context;
use serde::Deserialize;
use tracing::info;

// A Config currently contains the local + remote ip & port
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub local_address: String,
    pub local_port: u32,
    pub remote_address: String,
    pub remote_port: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

// implement a constructor for the config
impl Config {
    pub fn new() -> Self {
        Config {
            local_address: String::from("127.0.0.1"),
            local_port: 8080,
            remote_address: String::from("example.com"),
            remote_port: 8082,
        }
    }

    pub fn from(config_file: &str) -> Result<Self, anyhow::Error> {
        let config_file =
            std::fs::read_to_string(config_file).context("failed to read config file")?;
        // let config: Config = json::from_str(&config_file).context("failed to parse config file")?;

        let config: Config = match serde_json::from_str(&config_file) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("[WASM] > _process_config ERROR: {}", e);
                return Err(anyhow::Error::msg("failed to parse config file"));
            }
        };

        Ok(config)
    }
}

#[derive(Debug, Clone)]
pub enum V0CRole {
    Unknown,
    Dialer(i32),
    Listener(i32),
}

// V0 specific configurations
// The addr:port pair will either be local / remote depend on the client_type
#[derive(Debug, Clone)]
pub struct V0Config {
    pub name: String,
    pub addr: String,
    pub port: u32,

    pub conn: V0CRole,
}

impl V0Config {
    pub fn init(name: String, addr: String, port: u32) -> Result<Self, anyhow::Error> {
        Ok(V0Config {
            name,
            addr,
            port,
            conn: V0CRole::Unknown,
        })
    }

    pub fn connect(&mut self) -> Result<std::net::TcpStream, anyhow::Error> {
        let addr = format!("{}:{}", self.addr, self.port);

        info!("[HOST] WATERCore V0 connecting to {}", addr);

        let conn = std::net::TcpStream::connect(addr)?;
        self.conn = V0CRole::Dialer(conn.as_raw_fd());
        Ok(conn)
    }

    pub fn create_listener(&mut self) -> Result<std::net::TcpListener, anyhow::Error> {
        let addr = format!("{}:{}", self.addr, self.port);

        info!("[HOST] WATERCore V0 creating listener on {}", addr);

        let listener = std::net::TcpListener::bind(addr)?;
        self.conn = V0CRole::Listener(listener.as_raw_fd());
        Ok(listener)
    }

    pub fn accept(&mut self) -> Result<std::net::TcpStream, anyhow::Error> {
        match &self.conn {
            V0CRole::Listener(listener) => {
                let listener = unsafe { std::net::TcpListener::from_raw_fd(*listener) };
                let (stream, _) = listener.accept()?;
                self.conn = V0CRole::Listener(listener.as_raw_fd()); // makde sure it is not closed after scope
                Ok(stream)
            }
            _ => Err(anyhow::Error::msg("not a listener")),
        }
    }

    pub fn defer(&mut self) {
        match &self.conn {
            V0CRole::Listener(listener) => {
                let listener = unsafe { std::net::TcpListener::from_raw_fd(*listener) };
                drop(listener);
            }
            V0CRole::Dialer(conn) => {
                let conn = unsafe { std::net::TcpStream::from_raw_fd(*conn) };
                drop(conn);
            }
            _ => {}
        }
    }
}
