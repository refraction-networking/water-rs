use super::*;

// A Config currently contains the local + remote ip & port
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub remote_address: String,
    pub remote_port: u32,
    pub local_address: String,
    pub local_port: u32,
    pub bypass: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

// implement a constructor for the config
impl Config {
    pub fn new() -> Self {
        Config {
            remote_address: String::from("example.com"),
            remote_port: 8082,
            local_address: String::from("127.0.0.1"),
            local_port: 8080,
            bypass: false,
        }
    }
}

// ============ Some implementation for V1 ============
// A config struct that shares between your host & wasm to establish a connection
// #[cfg(feature = "v1")]
#[derive(Serialize, Deserialize)]
pub struct StreamConfigV1 {
    pub addr: String,
    pub port: u32,
    pub name: String,
}

// #[cfg(feature = "v1")]
impl StreamConfigV1 {
    pub fn init(addr: String, port: u32, name: String) -> Self {
        StreamConfigV1 { addr, port, name }
    }
}
