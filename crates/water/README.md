# Rust APIs -- `water`

For the latest API docs see: [here](https://app.gitbook.com/o/KHlQypYtIQKkb8YeZ6Hx/s/lVX5MqollomuX6vW80T6/rust-apis).

---

## External (Caller Facing)

<figure><img src="docs/images/water_rust_lib_draft1.png" alt=""><figcaption><p>design diagram draft1</p></figcaption></figure>

#### Sync:

<pre class="language-rust"><code class="lang-rust"><strong>pub struct WATERClient {
</strong>    pub config: Config,
    debug: bool,
}

impl WATERClient {
    pub fn new(conf: Config) -> Result&#x3C;Self, anyhow::Error> 
    pub fn set_debug(&#x26;mut self, debug: bool)
    
    // create a WATERStream which is a context wrapper for WASM instance
    // and (maybe later?) start listening to inbound connections 
    pub fn connect(&#x26;mut self, addr: &#x26;str) -> Result&#x3C;WATERStream&#x3C;Host>, anyhow::Error>
}

pub trait WATERStreamOps {
    fn user_write_done(&#x26;mut self, n: i32) -> Result&#x3C;i32, anyhow::Error>;
    fn user_will_read(&#x26;mut self) -> Result&#x3C;i32, anyhow::Error>;
}
-
pub struct WATERStream {
    // wasmtime runtime essentials
    pub instance: wasmtime::instance,
    ...
      
    // NOTE: Maybe is a better Rust idiom to put these into a trait instead of here
    //user_write_done: Box&#x3C;dyn Fn(i32) -> Result&#x3C;i32, anyhow::Error>>
}

// Should it to be called as WATERStream 
impl WATERStream {
    pub fn read(&#x26;self, buf: &#x26;mut [u8]) -> Result&#x3C;u32, anyhow::Error>;
    pub fn write(&#x26;self, buf: &#x26;mut [u8]) -> Result&#x3C;u32, anyhow::Error>;
    
    pub fn set_nonblocking(&#x26;self, nonblocking: bool)    
}

impl WATERStreamOps for WATERStream { /*...*/ }

impl Read for &#x26;WATERStream {
    pub fn read(&#x26;mut self, buf: &#x26;mut [u8]) -> Result&#x3C;u32>
}

impl Write for &#x26;WATERStream {
    pub fn write(&#x26;mut self, buf: &#x26;[u8]) -> Result&#x3C;u32>
    pub fn flush(&#x26;mut self) -> Result&#x3C;()>
}
</code></pre>

#### Async + Multiple connections

```rust
impl AsyncRead for &WATERStream {
    pub fn poll_read(&mut self, buf: &mut [u8]) -> Result<u32>
}

impl AsyncWrite for &WATERStream {
    pub fn poll_write(&mut self, buf: &[u8]) -> Result<u32>
    pub fn poll_flush(&mut self) -> Result<()>
}

pub fn split(stream: &mut WATERStream) -> (ReadHalf<'_>, WriteHalf<'_>)
```







Draft:

<pre class="language-rust"><code class="lang-rust">/// Match TcpListener idioms
pub struct WATERListener {
    // wasmtime runtime essentials
    pub instance: wasmtime::instance,
    ...
}

impl WATERListener {
    // convention for listen
    pub fn bind() 
    
    pub fn incoming()
}


type NetStream = u32;

// Define DialFn and DialFnLocal
type DialFn = fn(&#x26;str) -> Result&#x3C;NetStream>;
type DialFnLocal = fn(&#x26;str) -> Result&#x3C;NetStream>;

pub fn dial(addr: &#x26;str) -> Result&#x3C;NetStream> {}

pub fn dial_context(addr: &#x26;str) -> Result&#x3C;NetStream> {}

struct Config {
    water_path: String,
    // In Rust, we'll use Cursor from the standard library as an equivalent to Go's bytes.Reader
    water_bin: Cursor&#x3C;Vec&#x3C;u8>>,
    water_config_blob: Vec&#x3C;u8>,
    features: Vec&#x3C;String>,
    
    // The entry point of WASM module after initializations
    entry_fn: String,
    
    // To be exported to WASM for dial with a specific protocol
    dial_fns: HashMap&#x3C;String, DialFn>,
<strong>}
</strong>
pub struct Dialer {
    config: Config,
    internal_dial_fn: DialFn,
    internal_dial_fn_local: DialFnLocal,
}

impl Dialer {
    pub fn dial(&#x26;self, network: &#x26;str, addr: &#x26;str) -> Result&#x3C;NetStream> {}
    pub fn dial_context(&#x26;self, addr: &#x26;str) -> Result&#x3C;NetStream> {}
}
</code></pre>

## Internal (WASM Facing)



#### Multiple connection (Listener)

```
```

## Examples

<details>

<summary>Example 1 simple HTTP client</summary>

```rust
/// Minimal example
let t = TcpStream::New("127.0.0.1:443")

/// With a normal Rust program - how to do HTTP request low-level
let water_con = WATERStreamConnector::init(Config { water_path: "./proxy.wasm" });

let mut stream: WATERStream = water_con.connect("127.0.0.1:443");
let req = "GET ...\r\n\r\n";

stream.write_all(req.as_bytes());

let mut response = String::new();
stream.read_to_string(&mut response);

/// Some higher-level possible ways for WATER
let water_path = "./proxy.wasm";
let water = WATERStream::new(Config { water_path: water_path } );

let req = "GET ...";
// OR
let req = tool_generate_http_packet.generate("www.twitter.com")
                .insert_header((...));

let res = water.send()?;
```

</details>

<details>

<summary>Example 2 simple proxy</summary>

```rust
```

</details>

<details>

<summary>Example 3 async proxy</summary>



</details>

<details>

<summary>Example 4 Multi-client Server</summary>



</details>


## Designs
**execute**: 
1. wasmtime runtime creation
2. Setup env:
    1. memory initialiation & limitation
    2. (`v1_preview` feature) wasm_config sharing to WASM
    3. export helper functions (e.g. creation of TCP, TLS, crypto, etc)
3. (`v1` feature) setup multi-threading
4. Run the `entry_fn` or execute as the Role (`Dial`, `Listen`, `Relay`)
