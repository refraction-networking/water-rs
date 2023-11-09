use crate::runtime::*;

pub trait WATERRelayTrait: Send {
    fn associate(&mut self, conf: &WATERConfig, _addr: &str, _port: u16)
        -> Result<(), anyhow::Error>;

    fn read(&mut self, _buf: &mut Vec<u8>) -> Result<i64, anyhow::Error> {
        Err(anyhow::anyhow!("Method not supported"))
    }

    fn write(&mut self, _buf: &[u8]) -> Result<(), anyhow::Error> {
        Err(anyhow::anyhow!("Method not supported"))
    }

    // v0 only
    fn cancel_with(&mut self, _conf: &WATERConfig) -> Result<(), anyhow::Error> {
        Err(anyhow::anyhow!("Method not supported"))
    }

    // v0 only
    fn cancel(&mut self, _conf: &WATERConfig) -> Result<(), anyhow::Error> {
        Err(anyhow::anyhow!("Method not supported"))
    }

    // v0 only
    fn run_entry_fn(
        &mut self,
        _conf: &WATERConfig,
    ) -> Result<std::thread::JoinHandle<Result<(), anyhow::Error>>, anyhow::Error> {
        Err(anyhow::anyhow!("Method not supported"))
    }
}
