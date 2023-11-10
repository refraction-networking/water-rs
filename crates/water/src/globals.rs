#![allow(dead_code)]

pub const WASM_PATH: &str = "./proxy.wasm";
pub const CONFIG_WASM_PATH: &str = "./conf.json";

pub const MAIN: &str = "main";
pub const VERSION_FN: &str = "_water_version";
pub const INIT_FN: &str = "_water_init";
pub const CONFIG_FN: &str = "_water_config";
pub const WATER_BRIDGING_FN: &str = "_water_set_inbound";
pub const READER_FN: &str = "_water_read";
pub const WRITER_FN: &str = "_water_write";
pub const ACCEPT_FN: &str = "_water_accept";
pub const DIAL_FN: &str = "_water_dial";
pub const ASSOCIATE_FN: &str = "_water_associate";
pub const CANCEL_FN: &str = "_water_cancel_with";

pub const RUNTIME_VERSION_MAJOR: i32 = 0x001aaaaa;
pub const RUNTIME_VERSION: &str = "v0.1-alpha";
