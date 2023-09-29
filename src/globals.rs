pub const WASM_PATH: &str = "./proxy.wasm";
pub const CONFIF_WASM_PATH: &str = "./conf.json";

const ALLOC_FN: &str = "alloc";
const MEMORY: &str = "memory";
const DEALLOC_FN: &str = "dealloc";

pub const MAIN: &str = "main";
pub const VERSION_FN: &str = "_version";
pub const INIT_FN: &str = "_init";
pub const CONFIG_FN: &str = "_process_config";
pub const USER_READ_FN: &str = "_user_will_read";
pub const WRITE_DONE_FN: &str = "_user_write_done";
pub const WATER_BRIDGING_FN: &str = "_water_bridging";
pub const READER_FN: &str = "_read_from_net";
pub const WRITER_FN: &str = "_write_2_net";

pub const RUNTIME_VERSION_MAJOR: i32 = 0x001aaaaa;
pub const RUNTIME_VERSION: &str = "v0.1-alpha";
