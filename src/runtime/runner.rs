use crate::runtime::*;

pub struct WATERRunner<Host> {
    pub core: H2O<Host>, // core WASM runtime (engine, linker, instance, store, module)
}

impl WATERRunner<Host> {
    /// Run the entry function
    pub fn run(&mut self, conf: &Config) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERRunner running...");

        let fnc = self.core.instance.get_func(&mut self.core.store, &conf.entry_fn).unwrap();
        match fnc.call(&mut self.core.store, &[], &mut []) {
            Ok(_) => {},
            Err(e) => return Err(anyhow::Error::msg(format!("run function failed: {}", e))),
        }
        
        Ok(())
    }
    
    pub fn init(conf: &Config) -> Result<Self, anyhow::Error> {
        info!("[HOST] WATERRunner init...");

        let mut core = H2O::init(conf)?;
        core._prepare(conf)?;
        
        let runtime = WATERRunner {
            core: core,
        };

        Ok(runtime)
    }
}