//! A general runner that don't have any pre-defined roles where it all depends on the configuration passed in.
//! Currently used for v1_preview's relay mode, running shadowsocks.wasm.

use crate::runtime::*;

pub struct WATERRunner<Host> {
    pub core: H2O<Host>, // core WASM runtime (engine, linker, instance, store, module)
}

impl WATERRunner<Host> {
    /// Run the entry function
    pub fn run(&mut self, conf: &WATERConfig) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERRunner running...");

        let store_lock_result = self.core.store.lock();

        let mut store = match store_lock_result {
            Ok(store) => store,
            Err(e) => return Err(anyhow::Error::msg(format!("Failed to lock store: {}", e))),
        };

        let fnc = self
            .core
            .instance
            .get_func(&mut *store, &conf.entry_fn)
            .context(format!(
                "failed to find declared entry function: {}",
                &conf.entry_fn
            ))?;
        match fnc.call(&mut *store, &[], &mut []) {
            Ok(_) => {}
            Err(e) => return Err(anyhow::Error::msg(format!("run function failed: {}", e))),
        }

        Ok(())
    }

    pub fn init(_conf: &WATERConfig, core: H2O<Host>) -> Result<Self, anyhow::Error> {
        info!("[HOST] WATERRunner init...");

        let runtime = WATERRunner { core };

        Ok(runtime)
    }
}
