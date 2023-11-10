use crate::runtime::*;
use listener::WATERListenerTrait;
use relay::WATERRelayTrait;
use stream::WATERStreamTrait;

// =================== WATERClient Definition ===================
pub enum WATERClientType {
    Dialer(Box<dyn WATERStreamTrait>),
    Listener(Box<dyn WATERListenerTrait>),
    Relay(Box<dyn WATERRelayTrait>),
    Runner(WATERRunner<Host>), // This is a customized runner -- not like any stream
}

pub struct WATERClient {
    debug: bool,

    pub config: WATERConfig,
    pub stream: WATERClientType,
}

impl WATERClient {
    pub fn new(conf: WATERConfig) -> Result<Self, anyhow::Error> {
        info!("[HOST] WATERClient initializing ...");

        let mut core = H2O::init(&conf)?;
        core._prepare(&conf)?;

        // client_type: 0 -> Dialer, 1 -> Listener, 2 -> Runner
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

    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

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

    pub fn listen(&mut self) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERClient listening ...");

        match &mut self.stream {
            WATERClientType::Listener(listener) => {
                listener.listen(&self.config)?;
            }
            _ => {
                return Err(anyhow::anyhow!("[HOST] This client is not a Listener"));
            }
        }
        Ok(())
    }

    pub fn relay(&mut self) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERClient relaying ...");

        match &mut self.stream {
            WATERClientType::Relay(relay) => {
                relay.relay(&self.config)?;
            }
            _ => {
                return Err(anyhow::anyhow!("[HOST] This client is not a Relay"));
            }
        }
        Ok(())
    }

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

    // this will start a worker(WATM) in a separate thread -- returns a JoinHandle
    pub fn run_worker(
        &mut self,
    ) -> Result<std::thread::JoinHandle<Result<(), anyhow::Error>>, anyhow::Error> {
        info!("[HOST] WATERClient run_worker ...");

        match &mut self.stream {
            WATERClientType::Dialer(dialer) => dialer.run_entry_fn(&self.config),
            WATERClientType::Listener(listener) => {
                // TODO: clone listener here, since we are doing one WATM instance / accept
                listener.run_entry_fn(&self.config)
            }
            WATERClientType::Relay(relay) => relay.run_entry_fn(&self.config),
            _ => Err(anyhow::anyhow!("This client is not a Runner")),
        }
    }

    // this will run the extry_fn(WATM) in the current thread -- replace Host when running
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

    // v0 func for Host to set pipe for canceling later
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

    // v0 func for Host to terminate the separate thread running worker(WATM)
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
