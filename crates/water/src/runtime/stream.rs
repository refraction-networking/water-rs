use crate::runtime::*;

pub trait WATERStreamTrait: Send {
    fn connect(&mut self, conf: &WATERConfig, _addr: &str, _port: u16)
        -> Result<(), anyhow::Error>;
    fn cancel_with(&mut self, conf: &WATERConfig) -> Result<(), anyhow::Error> {
        Err(anyhow::anyhow!("Method not supported"))
    }
    fn cancel(&mut self, conf: &WATERConfig) -> Result<(), anyhow::Error> {
        Err(anyhow::anyhow!("Method not supported"))
    }
    fn run_entry_fn(
        &mut self,
        conf: &WATERConfig,
    ) -> Result<std::thread::JoinHandle<Result<(), anyhow::Error>>, anyhow::Error> {
        Err(anyhow::anyhow!("Method not supported"))
    }
    fn read(&mut self, buf: &mut Vec<u8>) -> Result<i64, anyhow::Error> {
        Err(anyhow::anyhow!("Method not supported"))
    }
    fn write(&mut self, buf: &[u8]) -> Result<(), anyhow::Error> {
        Err(anyhow::anyhow!("Method not supported"))
    }
}
