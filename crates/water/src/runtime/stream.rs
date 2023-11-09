use crate::runtime::{*, transport::WATERTransportTrait};

pub trait WATERStreamTrait: WATERTransportTrait {
    fn connect(&mut self, conf: &WATERConfig)
        -> Result<(), anyhow::Error>;
}
