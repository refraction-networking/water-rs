//! Configurations for the v0 runtime

use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd};

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

/// Constructor for the config
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

/// A enum to store the role of the connection for v0 as well as the fd for the connection
/// Listener and Relay will have multiple fds for bi-directional connections.
#[derive(Debug, Clone)]
pub enum V0CRole {
    Unknown,
    Dialer(i32),

    /// listener_fd, accepted_fd
    Listener(i32, i32),

    /// listener_fd, accepted_fd, dialer_fd
    Relay(i32, i32, i32),
}

/// V0 specific configurations with the V0Role stored
#[derive(Debug, Clone)]
pub struct V0Config {
    pub name: String,
    pub loc_addr: String,
    pub loc_port: u32,

    pub remote_addr: String,
    pub remote_port: u32,

    pub conn: V0CRole,
}

impl V0Config {
    pub fn init(
        name: String,
        loc_addr: String,
        loc_port: u32,
        remote_addr: String,
        remote_port: u32,
    ) -> Result<Self, anyhow::Error> {
        Ok(V0Config {
            name,
            loc_addr,
            loc_port,
            remote_addr,
            remote_port,
            conn: V0CRole::Unknown,
        })
    }

    /// It will connect to the remote addr and set the fd in the V0Config
    pub fn connect(&mut self) -> Result<std::net::TcpStream, anyhow::Error> {
        let addr = format!("{}:{}", self.remote_addr, self.remote_port);

        info!("[HOST] WATERCore V0 connecting to {}", addr);

        match &mut self.conn {
            // if the V0CRole is Relay, then it will remain as Relay
            V0CRole::Relay(_, _, ref mut conn_fd) => {
                // now relay has been built, need to dial
                if *conn_fd != -1 {
                    return Err(anyhow::Error::msg("Relay already connected"));
                }

                let conn = std::net::TcpStream::connect(addr)?;
                *conn_fd = conn.as_raw_fd();
                Ok(conn)
            }
            // if the V0CRole has not been set, and connect() was called, then it should be a dialer
            V0CRole::Unknown => {
                let conn = std::net::TcpStream::connect(addr)?;
                self.conn = V0CRole::Dialer(conn.as_raw_fd());
                Ok(conn)
            }
            _ => Err(anyhow::Error::msg("not a dialer")),
        }
    }

    /// It will create a listener and set the fd in the V0Config (for either listener or relay)
    pub fn create_listener(&mut self, is_relay: bool) -> Result<(), anyhow::Error> {
        let addr = format!("{}:{}", self.loc_addr, self.loc_port);

        info!("[HOST] WATERCore V0 creating listener on {}", addr);

        let listener = std::net::TcpListener::bind(addr)?;

        if is_relay {
            self.conn = V0CRole::Relay(listener.into_raw_fd(), -1, -1);
        } else {
            self.conn = V0CRole::Listener(listener.into_raw_fd(), -1);
        }
        Ok(())
    }

    /// It will accept a connection and set the fd in the V0Config (for either listener or relay)
    pub fn accept(&mut self) -> Result<std::net::TcpStream, anyhow::Error> {
        info!("[HOST] WATERCore V0 accept with conn {:?} ...", self.conn);

        match self.conn {
            V0CRole::Listener(ref mut listener_fd, ref mut accepted_fd) => {
                if *accepted_fd != -1 {
                    return Err(anyhow::Error::msg("Listener already accepted"));
                }

                let listener = unsafe { std::net::TcpListener::from_raw_fd(*listener_fd) };

                let (stream, _) = listener.accept()?;

                *listener_fd = listener.into_raw_fd(); // made sure the listener is not closed after scope
                *accepted_fd = stream.as_raw_fd();

                Ok(stream)
            }
            V0CRole::Relay(ref mut listener_fd, ref mut accepted_fd, _) => {
                if *accepted_fd != -1 {
                    return Err(anyhow::Error::msg("Relay already accepted"));
                }

                let listener = unsafe { std::net::TcpListener::from_raw_fd(*listener_fd) };
                let (stream, _) = listener.accept()?;
                *listener_fd = listener.into_raw_fd(); // made sure the listener is not closed after scope
                *accepted_fd = stream.as_raw_fd();
                Ok(stream)
            }
            _ => Err(anyhow::Error::msg("not a listener")),
        }
    }

    /// It will close the connection to remote / accepted connection listened and exit gracefully
    pub fn defer(&mut self) {
        info!("[HOST] WATERCore V0 defer with conn {:?} ...", self.conn);

        match self.conn {
            V0CRole::Listener(_, ref mut accepted_fd) => {
                // The accepted stream should be defered, not the listener
                let accepted_conn = unsafe { std::net::TcpStream::from_raw_fd(*accepted_fd) };
                drop(accepted_conn);
                *accepted_fd = -1; // set it back to default
            }
            V0CRole::Dialer(conn_fd) => {
                let conn = unsafe { std::net::TcpStream::from_raw_fd(conn_fd) };
                drop(conn);
            }
            V0CRole::Relay(_, ref mut accepted_fd, ref mut conn_fd) => {
                let accepted_conn = unsafe { std::net::TcpStream::from_raw_fd(*accepted_fd) };
                drop(accepted_conn);
                *accepted_fd = -1; // set it back to default

                let conn = unsafe { std::net::TcpStream::from_raw_fd(*conn_fd) };
                drop(conn);
                *conn_fd = -1; // set it back to default
            }
            _ => {}
        }
    }

    /// It is used for listener and relay only, to reset the accepted connection in the migrated listener / relay
    pub fn reset_listener_or_relay(&mut self) {
        info!(
            "[HOST] WATERCore v0 reset lisener / relay with conn {:?} ...",
            self.conn
        );

        match self.conn {
            V0CRole::Listener(_, ref mut accepted_fd) => {
                if *accepted_fd != -1 {
                    *accepted_fd = -1; // set it back to default
                }
            }
            V0CRole::Relay(_, ref mut accepted_fd, ref mut conn_fd) => {
                if *accepted_fd != -1 {
                    *accepted_fd = -1; // set it back to default
                }

                if *conn_fd != -1 {
                    *conn_fd = -1; // set it back to default
                }
            }
            _ => {}
        }
    }
}
