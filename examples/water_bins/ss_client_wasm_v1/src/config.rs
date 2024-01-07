//! Specific config for the ss client, with more fields like password,
//! and others like cipher method(adding later)

use serde::Deserialize;

// A Config currently contains the local + remote ip & port
#[derive(Debug, Deserialize, Clone)]
pub struct SSConfig {
    pub remote_address: String,
    pub remote_port: u32,
    pub local_address: String,
    pub local_port: u32,
    pub password: String,
    pub bypass: bool,
    // NOTE: will add the config for ciphter method later
    // pub method: CipherKind,
}

impl Default for SSConfig {
    fn default() -> Self {
        Self::new()
    }
}

// implement a constructor for the config
impl SSConfig {
    pub fn new() -> Self {
        SSConfig {
            remote_address: String::from("example.com"),
            remote_port: 8082,
            local_address: String::from("127.0.0.1"),
            local_port: 8080,
            password: String::from("Test!23"),
            bypass: false,
        }
    }
}
