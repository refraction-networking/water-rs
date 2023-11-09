use crate::runtime::{*, transport::WATERTransportTrait};

pub trait WATERListenerTrait: WATERTransportTrait {
    fn accept(&mut self, conf: &WATERConfig)
        -> Result<(), anyhow::Error>;
    
    fn listen(&mut self, conf: &WATERConfig) -> Result<(), anyhow::Error> {
        Err(anyhow::anyhow!("Method not supported"))
    }
}