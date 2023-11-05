use std::fmt;
use std::str::FromStr;

use crate::runtime::v0::config::{Config, V0Config};
use crate::runtime::*;

pub enum Version {
    Unknown,
    V0(Option<V0Config>),
    V1,
    V2,
}

impl Version {
    pub fn parse(s: &str) -> Option<Version> {
        match Version::from_str(s) {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }

    // Current API v0 needs some configurations at the beginning
    pub fn config_v0(&mut self, conf: &WATERConfig) -> Result<Version, anyhow::Error> {
        info!("[HOST] WATERCore configuring for V0");

        let wasm_config = Config::from(&conf.config_wasm)?;

        let v = match conf.client_type {
            WaterBinType::Dial => {
                let v0_conf = V0Config::init(
                    "CONNECT".into(),
                    wasm_config.remote_address.clone(),
                    wasm_config.remote_port,
                )?;
                Version::V0(Some(v0_conf))
            }
            WaterBinType::Listen => {
                let v0_conf = V0Config::init(
                    "LISTEN".into(),
                    wasm_config.local_address.clone(),
                    wasm_config.local_port,
                )?;
                Version::V0(Some(v0_conf))
            }
            WaterBinType::Unknown => {
                Version::Unknown // WATER is setting up?
            }
            _ => {
                unimplemented!("This client type is not supported yet")
            }
        };

        Ok(v)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Version::Unknown => "_water_setting_up",
            Version::V0(_v0_conf) => "_water_v0",
            Version::V1 => "_water_v1",
            Version::V2 => "_water_v2",
        }
    }
}

impl FromStr for Version {
    type Err = ();

    fn from_str(s: &str) -> Result<Version, ()> {
        match s {
            "_water_v0" => Ok(Version::V0(None)),
            "_water_v1" => Ok(Version::V1),
            "_water_v2" => Ok(Version::V2),
            _ => Err(()),
        }
    }
}

impl From<&Version> for &'static str {
    fn from(v: &Version) -> &'static str {
        match v {
            Version::Unknown => "_water_setting_up",
            Version::V0(_v0_conf) => "_water_v0",
            Version::V1 => "_water_v1",
            Version::V2 => "_water_v2",
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.into())
    }
}
