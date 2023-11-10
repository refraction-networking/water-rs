use crate::runtime::{transport::WATERTransportTrait, *};

pub trait WATERRelayTrait: WATERTransportTrait {
    fn associate(&mut self, conf: &WATERConfig) -> Result<(), anyhow::Error>;

    fn relay(&mut self, conf: &WATERConfig) -> Result<(), anyhow::Error>;
}
