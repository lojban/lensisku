use std::sync::OnceLock;
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;
use wasmtime_wasi::WasiCtx;

static ENGINE: OnceLock<Engine> = OnceLock::new();
static MODULE: OnceLock<Module> = OnceLock::new();

fn get_engine() -> &'static Engine {
    ENGINE.get_or_init(|| {
        let config = Config::new();
        // config.wasm_component_model(false);
        Engine::new(&config).expect("Failed to create WASM engine")
    })
}

fn get_module() -> &'static Module {
    MODULE.get_or_init(|| {
        let engine = get_engine();
        // assets/wasm/tersmu.wasm
        let wasm_bytes = include_bytes!("./assets/wasm/tersmu.wasm");
        Module::from_binary(engine, wasm_bytes).expect("Failed to load tersmu.wasm")
    })
}

pub struct TersmuParser {
    store: Store<WasiCtx>,
    instance: Instance,
}

impl TersmuParser {
    pub fn new() -> Result<Self, anyhow::Error> {
        let engine = get_engine();
        let module = get_module();
        let mut linker = Linker::new(engine);

        // Link WASI preview1
        wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

        // Setup WASI context
        let wasi = WasiCtxBuilder::new()
            .inherit_stdout()
            .inherit_stderr()
            .build();

        let mut store = Store::new(engine, wasi);
        let instance = linker.instantiate(&mut store, module)?;

        // Initialize Haskell RTS if exported
        if let Ok(hs_init) = instance.get_typed_func::<(i32, i32), ()>(&mut store, "hs_init") {
            let _ = hs_init.call(&mut store, (0, 0));
        }

        // Initialize Tersmu if exported
        if let Ok(init_tersmu) = instance.get_typed_func::<(), ()>(&mut store, "initTersmu") {
            let _ = init_tersmu.call(&mut store, ());
        }

        Ok(Self { store, instance })
    }

    pub fn parse(&mut self, text: &str) -> Result<String, anyhow::Error> {
        let memory = self
            .instance
            .get_memory(&mut self.store, "memory")
            .ok_or_else(|| anyhow::anyhow!("Memory export not found"))?;

        // Allocate memory for input string
        let malloc = self
            .instance
            .get_typed_func::<i32, i32>(&mut self.store, "malloc")?;
        let free = self
            .instance
            .get_typed_func::<i32, ()>(&mut self.store, "free")?;

        let bytes = text.as_bytes();
        let len = bytes.len();
        // Allocate len + 1 for null terminator
        let ptr = malloc.call(&mut self.store, (len as i32) + 1)?;

        // Write string to memory
        memory.write(&mut self.store, ptr as usize, bytes)?;
        memory.write(&mut self.store, ptr as usize + len, &[0])?; // null terminator

        // Call parseLojban
        let parse_lojban = self
            .instance
            .get_typed_func::<i32, i32>(&mut self.store, "parseLojban")
            .or_else(|_| {
                self.instance
                    .get_typed_func::<i32, i32>(&mut self.store, "parse_lojban")
            })?;

        let result_ptr = parse_lojban.call(&mut self.store, ptr)?;

        // Read result string
        let result = self.read_string(memory, result_ptr)?;

        // Free memory
        free.call(&mut self.store, ptr)?;
        free.call(&mut self.store, result_ptr)?;

        Ok(result)
    }

    fn read_string(&mut self, memory: Memory, ptr: i32) -> Result<String, anyhow::Error> {
        let mut buffer = Vec::new();
        let mut offset = 0;

        loop {
            let read_ptr = (ptr as usize) + offset;
            // This is a bit inefficient reading byte by byte or chunking and searching,
            // but safe against memory bounds if done carefully.
            // Wasmtime memory access checks bounds.

            // Let's read byte by byte for simplicity first to find null terminator
            // Optimization: read chunk, find 0, then read exact.

            // Simple implementation:
            let byte = memory.data(&self.store)[read_ptr];
            if byte == 0 {
                break;
            }
            buffer.push(byte);
            offset += 1;
        }

        Ok(String::from_utf8(buffer)?)
    }
}

pub fn parse_lojban(text: &str) -> Result<String, anyhow::Error> {
    let mut parser = TersmuParser::new()?;
    parser.parse(text)
}

pub fn get_canonical_form(text: &str) -> Option<String> {
    if let Ok(json_str) = parse_lojban(text) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str) {
            return val
                .get("canonical")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
        }
    }
    None
}
