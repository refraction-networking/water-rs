# WebAssembly Transport Module (WATM) APIs -- `water_wasm-v0`

---
A set of Universal APIs in WASM when integrate with Rust / Go Host
---


## WASM -> Host

Every valid WASM Transport Module is required to export a set of functions.&#x20;

Currently this documentation contains both `generic` (version-independent) specs and `v0` specs. Unless otherwise specified, all APIs are mandatory.

### Generic: version independent exports

```rust
/// init is for the WASM module to setup its internal states with or without a 
/// configuration specified by the Host.
///
/// A configurable WATM should call pull_config() and parse 
/// the config file.
#[export_name = "_water_init"]
pub fn _init() -> i32
```

### v0: water draft version 0

```rust
/// _v0 showing up in the export list tells the runtime it should interpret 
/// this WATM in v0 spec and talk to it with v0 APIs only.
///
/// The literal name/type of this export does not matter.
#[export_name = "_water_v0"]
pub static VERSION: i32 = v0plus::VERSION;

/// _cancel_with specifies a file descriptor for the cancellation channel. 
/// The WATM will select on this channel, and if successfully read any 
/// bytes, abort and exit with error "Aborted".
#[export_name = "_water_cancel_with"]
pub fn _cancel_with(fd: i32) -> i32

/// _worker is the entry point for the WATM. It is used to spin up 
/// a blocking thread that runs indefinitely, in which the WATM 
/// do its tasks asynchronously. 
#[export_name = "_water_worker"]
pub fn _worker() -> i32
```

#### v0-Dialer: dialer in water draft version 0

All dialer-compliant WATM must also implement

```rust
// in _dial, a dialer WATM opens the file indicated by 
// caller_conn_fd as caller_conn, calls back to the host 
// via host_dial() and open the file indicated by dst_conn_fd 
// (returned by host_dial) as dst_conn. The dialer UPGRADEs
// the caller_conn and sends upgraded payload to dst_conn.
#[export_name = "_water_dial"]
pub fn _dial(caller_conn_fd: i32) -> i32 // caller_conn_fd -> dst_conn_fd
```

#### v0-Listener: listener in water draft version 0

All listener-compliant WATM must also implement

```rust
// in _accept, a listener WATM opens the file indicated by 
// caller_conn_fd as caller_conn, calls back to the host via 
// host_accept() and open the file indicated by src_conn_fd 
// (returned by host_accept) as src_conn. The listener 
// UPGRADEs the caller_conn and sends upgraded payload to src_conn. 
#[export_name = "_water_accept"]
pub fn _accept(caller_conn_fd: i32) -> i32 // caller_conn_fd -> src_conn_fd
```

#### v0-Relay: relay in water draft version 0

All relay-compliant WATM must also implement

```rust
// in _associate, a relay calls back to the host
// via host_accept() to accept src_conn, and 
// calls back to the host via host_dial() to 
// dial dst_conn. Then, relay UPGRADEs the 
// src_conn and sends upgraded payload 
// to dst_conn.
#[export_name = "_water_associate"]
pub fn _associate() -> i32
```

## Host -> WASM

Functions that the host MUST import/link to every WASM Transport Module

```rust
fn host_accept() -> i32 // -> src_conn_fd
fn host_dial() -> i32 // -> dst_conn_fd
fn host_defer() // notify the host the WASM instance is winding down

// If no config is available, INVALID_FD will be returned.
// A config-optional WATM SHOULD proceed, and a config-mandatory
// WATM MUST fail.
//
// If config file is set but other error (e.g., I/O Error) happened, 
// a trap will be triggered. 
fn pull_config() -> i32 // fetch config from the host. -> config_fd
```

## Host internal

```
// Validate WASMBin to ensure that it implements a compatible transport.
// 1. Ensure using the Module that the binary exposes the required functions
//    with the correct signatures.
// 2. On launch, call version and ensure that the version is compatible with
//    the current Host library version.
// 3. Check for presense and correctness of any version specific function
//    signatures.
fn validate() -> Error<()>
func validate() error

// TODO: research how to cleanly implement this on Go. 
// currently it seems we will need to dump exports one by one and cast/test them into
// FuncType, then pre-build the signatures to compare them to. This doesn't look clean
// enough. 
```

