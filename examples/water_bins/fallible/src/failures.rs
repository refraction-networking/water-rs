use std::str::FromStr;

use water_wasm::{Decoder, DefaultDecoder, DefaultEncoder, Encoder};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Failures {
    #[default]
    Success,
    ConfigError,
    ConfigPanic,
    DialError,
    DialPanic,
    ReadError,
    ReadPanic,
    ReadTimeout,
    ReadHang,
    CloseOnRead,
    WriteError,
    WritePanic,
    WriteTimeout,
    WriteHang,
    CloseOnWrite,
    HandshakeError,
}

impl FromStr for Failures {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let out = match s {
            "ConfigError" => Failures::ConfigError,
            "ConfigPanic" => Failures::ConfigPanic,
            "DialError" => Failures::DialError,
            "DialPanic" => Failures::DialPanic,
            "ReadError" => Failures::ReadError,
            "ReadPanic" => Failures::ReadPanic,
            "ReadTimeout" => Failures::ReadTimeout,
            "ReadHang" => Failures::ReadHang,
            "CloseOnRead" => Failures::CloseOnRead,
            "WriteError" => Failures::WriteError,
            "WritePanic" => Failures::WritePanic,
            "WriteTimeout" => Failures::WriteTimeout,
            "WriteHang" => Failures::WriteHang,
            "CloseOnWrite" => Failures::CloseOnWrite,
            "HandshakeError" => Failures::HandshakeError,
            _ => Failures::Success,
        };
        Ok(out)
    }
}

pub trait Configurable {
    fn with_config(self, config_str: String) -> anyhow::Result<Self>
    where
        Self: Sized;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Config {
    throw: Failures,
}

impl FromStr for Config {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Config {
            throw: Failures::from_str(s)?,
        })
    }
}

impl TryFrom<&str> for Config {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(Config {
            throw: Failures::from_str(s)?,
        })
    }
}

pub struct IdentityTransport {
    config: Config,
    decoder: DefaultDecoder,
    encoder: DefaultEncoder,
    n_encodes: i32,
    n_decodes: i32,
}

impl Default for IdentityTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl IdentityTransport {
    pub fn new() -> Self {
        IdentityTransport {
            config: Config::default(),
            decoder: DefaultDecoder,
            encoder: DefaultEncoder,
            n_encodes: 0,
            n_decodes: 0,
        }
    }

    fn should_throw(&self, failure: Failures) -> bool {
        self.config.throw == failure
    }
}

impl Configurable for IdentityTransport {
    fn with_config(mut self, config_str: String) -> anyhow::Result<Self> {
        self.config = Config::from_str(&config_str)?;
        if self.should_throw(Failures::ConfigError) {
            return Err(anyhow::anyhow!("throw ConfigError"));
        } else if self.should_throw(Failures::ConfigPanic) {
            panic!("throw ConfigPanic");
        }
        Ok(self)
    }
}

impl Encoder for IdentityTransport {
    fn encode(&mut self, input: &[u8], output: &mut [u8]) -> anyhow::Result<u32, anyhow::Error> {
        if self.n_encodes == 0 && self.should_throw(Failures::HandshakeError) {
            return Err(anyhow::anyhow!("throw HandshakeError"));
        }

        self.n_encodes += 1;
        self.encoder.encode(input, output)
    }
}

impl Decoder for IdentityTransport {
    fn decode(&mut self, input: &[u8], output: &mut [u8]) -> anyhow::Result<u32, anyhow::Error> {
        self.n_decodes += 1;
        self.decoder.decode(input, output)
    }
}
