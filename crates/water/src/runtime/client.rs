use crate::runtime::*;
use stream::WATERStreamTrait;

// =================== WATERClient Definition ===================
pub enum WATERClientType {
    Dialer(Box<dyn WATERStreamTrait>),
    Listener(WATERListener<Host>),
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
                let stream = WATERListener::init(&conf, core)?;
                WATERClientType::Listener(stream)
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

    pub fn connect(&mut self, addr: &str, port: u16) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERClient connecting ...");

        match &mut self.stream {
            WATERClientType::Dialer(dialer) => {
                dialer.connect(&self.config, addr, port)?;
            }
            _ => {
                return Err(anyhow::anyhow!("[HOST] This client is not a Dialer"));
            }
        }
        Ok(())
    }

    pub fn listen(&mut self, addr: &str, port: u16) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERClient listening ...");

        match &mut self.stream {
            WATERClientType::Listener(listener) => {
                listener.listen(&self.config, addr, port)?;
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
            _ => {
                return Err(anyhow::anyhow!("This client is not a Runner"));
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
            _ => {
                // for now this is only implemented for v0 dialer
                return Err(anyhow::anyhow!("This client is not a v0 Dialer"));
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
