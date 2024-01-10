//! This module is to define the `WATERClient` struct and its methods
//!
//! `WATERClient` is the main struct that holds the WATERClientType and WATERConfig; used as the entry point of using the WATER runtime
//!
//! `WATERClientType` is an enum type that holds different types of clients

use crate::runtime::*;
use listener::WATERListenerTrait;
use relay::WATERRelayTrait;
use stream::WATERStreamTrait;

/// `WATERClientType` Definition: A enum type to hold different types of clients
pub enum WATERClientType {
    /// `Dialer`: create 1 WATM instance with the given `.wasm` binary to connect to a remote address
    Dialer(Box<dyn WATERStreamTrait>),

    /// `Listener`: create 1 WATM instance with the given `.wasm` binary to listen on a local address, and accept 1 connection (v0) or multiple connections asynchronizely (v1)
    Listener(Box<dyn WATERListenerTrait>),

    /// `Relay`: create 1 WATM instance with the given `.wasm` binary to listen on a local address, and connect to a remote address
    Relay(Box<dyn WATERRelayTrait>),

    /// `Runner`: create 1 WATM instance with the given `.wasm` binary to run the `entry_fn`
    Runner(WATERRunner<Host>), // This is a customized runner -- not like any stream; currently can run v1 relay (shadowsocks client)
}

/// `WATERClient` is used as the object for entering and managing the WASM runtime
pub struct WATERClient {
    debug: bool,

    pub config: WATERConfig,
    pub stream: WATERClientType,
}

impl WATERClient {
    /// `new` is the constructor of `WATERClient`
    /// it checks the client type and the version to create the corresponding `WATERClientType`
    pub fn new(conf: WATERConfig) -> Result<Self, anyhow::Error> {
        info!("[HOST] WATERClient initializing ...");

        let mut core = H2O::init_core(&conf)?;
        core._prepare(&conf)?;

        let water = match conf.client_type {
            WaterBinType::Dial => {
                let stream = match core.version {
                    Version::V0(_) => Box::new(v0::stream::WATERStream::init(&conf, core)?)
                        as Box<dyn WATERStreamTrait>,
                    Version::V1 => Box::new(v1::stream::WATERStream::init(&conf, core)?)
                        as Box<dyn WATERStreamTrait>,
                    _ => {
                        return Err(anyhow::anyhow!("Invalid version"));
                    }
                };

                WATERClientType::Dialer(stream)
            }
            WaterBinType::Listen => {
                let listener = match core.version {
                    Version::V0(_) => Box::new(v0::listener::WATERListener::init(&conf, core)?)
                        as Box<dyn WATERListenerTrait>,
                    Version::V1 => Box::new(v1::listener::WATERListener::init(&conf, core)?)
                        as Box<dyn WATERListenerTrait>,
                    _ => {
                        return Err(anyhow::anyhow!("Invalid version"));
                    }
                };

                WATERClientType::Listener(listener)
            }
            WaterBinType::Relay => {
                // host managed relay is only implemented for v0
                let relay = match core.version {
                    Version::V0(_) => Box::new(v0::relay::WATERRelay::init(&conf, core)?)
                        as Box<dyn WATERRelayTrait>,
                    _ => {
                        return Err(anyhow::anyhow!("Invalid version"));
                    }
                };

                WATERClientType::Relay(relay)
            }
            WaterBinType::Runner => {
                let runner = WATERRunner::init(&conf, core)?;
                WATERClientType::Runner(runner)
            }
            _ => {
                return Err(anyhow::anyhow!("Invalid client type"));
            }
        };

        Ok(WATERClient {
            config: conf,
            debug: false,
            stream: water,
        })
    }

    /// keep_listen is the function that is called when user wants to accept a newly income connection,
    /// it creates a new WASM instance and migrate the previous listener to it. -- v0_plus listener and relay for now.
    pub fn keep_listen(&mut self) -> Result<Self, anyhow::Error> {
        info!("[HOST] WATERClient keep listening...",);

        let water = match &mut self.stream {
            WATERClientType::Listener(ref mut listener) => WATERClientType::Listener(Box::new(
                v0::listener::WATERListener::migrate_listener(&self.config, listener.get_core())?,
            )
                as Box<dyn WATERListenerTrait>),
            WATERClientType::Relay(ref mut relay) => WATERClientType::Relay(Box::new(
                v0::relay::WATERRelay::migrate_listener(&self.config, relay.get_core())?,
            )
                as Box<dyn WATERRelayTrait>),
            _ => {
                return Err(anyhow::anyhow!(
                    "[HOST] This client is neither a Listener nor a Relay"
                ));
            }
        };

        Ok(WATERClient {
            config: self.config.clone(),
            debug: self.debug,
            stream: water,
        })
    }

    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    /// `connect` is the function for `Dialer` to connect to a remote address
    pub fn connect(&mut self) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERClient connecting ...");

        match &mut self.stream {
            WATERClientType::Dialer(dialer) => {
                dialer.connect(&self.config)?;
            }
            _ => {
                return Err(anyhow::anyhow!("[HOST] This client is not a Dialer"));
            }
        }
        Ok(())
    }

    /// `listen` is the function for `Listener` and `Relay` to create the Listener and listen on a local addr
    pub fn listen(&mut self) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERClient creating listener ...");

        match &mut self.stream {
            WATERClientType::Listener(listener) => {
                listener.listen(&self.config)?;
            }
            WATERClientType::Relay(relay) => {
                relay.listen(&self.config)?;
            }
            _ => {
                return Err(anyhow::anyhow!("[HOST] This client is not a Listener"));
            }
        }
        Ok(())
    }

    /// `associate` is the function for `Relay` to associate a remote connection
    pub fn associate(&mut self) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERClient relaying ...");

        match &mut self.stream {
            WATERClientType::Relay(relay) => {
                relay.associate(&self.config)?;
            }
            _ => {
                return Err(anyhow::anyhow!("[HOST] This client is not a Relay"));
            }
        }
        Ok(())
    }

    /// `accept` is the function for `Listener` to accept a connection
    /// called after `listen()`
    pub fn accept(&mut self) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERClient accepting ...");

        match &mut self.stream {
            WATERClientType::Listener(listener) => {
                listener.accept(&self.config)?;
            }
            _ => {
                return Err(anyhow::anyhow!("[HOST] This client is not a Listener"));
            }
        }
        Ok(())
    }

    /// `run_worker` is the function to run the entry_fn(a worker in WATM) in a separate thread and return the thread handle
    /// it will return a `JoinHandle` for the caller to manage the thread -- used by v0_plus
    pub fn run_worker(
        &mut self,
    ) -> Result<std::thread::JoinHandle<Result<(), anyhow::Error>>, anyhow::Error> {
        info!("[HOST] WATERClient run_worker ...");

        match &mut self.stream {
            WATERClientType::Dialer(dialer) => dialer.run_entry_fn(&self.config),
            WATERClientType::Listener(listener) => listener.run_entry_fn(&self.config),
            WATERClientType::Relay(relay) => relay.run_entry_fn(&self.config),
            _ => Err(anyhow::anyhow!("This client is not a Runner")),
        }
    }

    /// `execute` is the function to run the entry_fn(a worker in WATM) in the current thread
    /// -- replace the thread running Host when running it <- used by v1 currently
    pub fn execute(&mut self) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERClient Executing ...");

        match &mut self.stream {
            WATERClientType::Runner(runner) => {
                runner.run(&self.config)?;
            }
            WATERClientType::Dialer(dialer) => {
                dialer.run_entry_fn(&self.config)?;
            }
            WATERClientType::Listener(listener) => {
                listener.run_entry_fn(&self.config)?;
            }
            WATERClientType::Relay(relay) => {
                relay.run_entry_fn(&self.config)?;
            }
        }
        Ok(())
    }

    /// `cancel_with` is the function to set the cancel pipe for exiting later -- v0_plus
    pub fn cancel_with(&mut self) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERClient cancel_with ...");

        match &mut self.stream {
            WATERClientType::Dialer(dialer) => {
                dialer.cancel_with(&self.config)?;
            }
            WATERClientType::Listener(listener) => {
                listener.cancel_with(&self.config)?;
            }
            WATERClientType::Relay(relay) => {
                relay.cancel_with(&self.config)?;
            }
            _ => {
                // for now this is only implemented for v0 dialer
                return Err(anyhow::anyhow!("This client is not a v0 supported client"));
            }
        }
        Ok(())
    }

    /// `cancel` is the function to send thru the cancel_pipe and let the thread running the worker to exit -- v0_plus
    pub fn cancel(&mut self) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERClient canceling ...");

        match &mut self.stream {
            WATERClientType::Dialer(dialer) => {
                dialer.cancel(&self.config)?;
            }
            WATERClientType::Listener(listener) => {
                listener.cancel(&self.config)?;
            }
            WATERClientType::Relay(relay) => {
                relay.cancel(&self.config)?;
            }
            _ => {
                // for now this is only implemented for v0 dialer
                return Err(anyhow::anyhow!("This client is not a v0 Dialer"));
            }
        }
        Ok(())
    }

    /// `read` is the function to read from the stream
    pub fn read(&mut self, buf: &mut Vec<u8>) -> Result<i64, anyhow::Error> {
        info!("[HOST] WATERClient reading ...");

        let read_bytes = match &mut self.stream {
            WATERClientType::Dialer(dialer) => dialer.read(buf)?,
            WATERClientType::Listener(listener) => listener.read(buf)?,
            _ => {
                return Err(anyhow::anyhow!("This client is not supporting read"));
            }
        };

        Ok(read_bytes)
    }

    /// `write` is the function to write to the stream
    pub fn write(&mut self, buf: &[u8]) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERClient writing ...");

        match &mut self.stream {
            WATERClientType::Dialer(dialer) => {
                dialer.write(buf)?;
            }
            WATERClientType::Listener(listener) => {
                listener.write(buf)?;
            }
            _ => {
                return Err(anyhow::anyhow!("This client is not supporting write"));
            }
        }
        Ok(())
    }
}
