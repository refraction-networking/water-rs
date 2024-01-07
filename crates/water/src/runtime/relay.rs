//! Relay trait for WATER runtime (currently used for v0 only, v1 is using Runner for relay(ShadowSocks)).

use crate::runtime::{transport::WATERTransportTrait, *};

pub trait WATERRelayTrait: WATERTransportTrait {
    fn associate(&mut self, conf: &WATERConfig) -> Result<(), anyhow::Error>;

    fn relay(&mut self, conf: &WATERConfig) -> Result<(), anyhow::Error>;
}
