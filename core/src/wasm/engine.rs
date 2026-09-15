//! WASM engine wrapper for loading and compiling modules

use std::sync::{Arc, mpsc};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use wasmtime::{Cache, Config, Engine, ExternType, Module, OptLevel};

/// Shared WASM engine (one per application)
pub struct WasmEngine {
    engine: Engine,
    pub(crate) epoch_ticker: Option<Arc<EpochTicker>>,
}

pub(crate) struct EpochTicker {
    cancel: mpsc::Sender<()>,
    thread: Option<JoinHandle<()>>,
}

impl Drop for EpochTicker {
    fn drop(&mut self) {
        let _ = self.cancel.send(());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl WasmEngine {
    /// Create a new WASM engine with default configuration
    pub fn new() -> Result<Self> {
        Self::configured(false)
    }

    /// Ordinary player watchdog: one timer per engine, not one thread per callback.
    /// Epochs are wall-clock safety deadlines, not deterministic instruction budgets.
    pub fn new_bounded() -> Result<Self> {
        let mut engine = Self::configured(true)?;
        let clock = engine.engine.clone();
        let (cancel, receive) = mpsc::channel();
        let thread = std::thread::Builder::new()
            .name("wasm-watchdog".into())
            .spawn(move || {
                let started = Instant::now();
                let mut ticks = 0;
                while matches!(
                    receive.recv_timeout(Duration::from_millis(10)),
                    Err(mpsc::RecvTimeoutError::Timeout)
                ) {
                    // OS timer granularity can exceed 10 ms; count elapsed time, not wakeups.
                    let due = started.elapsed().as_millis() / 10;
                    for _ in ticks..due {
                        clock.increment_epoch();
                    }
                    ticks = due;
                }
            })
            .context("Failed to start WASM watchdog")?;
        engine.epoch_ticker = Some(Arc::new(EpochTicker {
            cancel,
            thread: Some(thread),
        }));
        Ok(engine)
    }

    /// Create an engine whose stores can be interrupted by epoch deadlines.
    pub fn new_interruptible() -> Result<Self> {
        Self::configured(true)
    }

    fn configured(epoch_interruption: bool) -> Result<Self> {
        let mut config = Config::new();
        config.epoch_interruption(epoch_interruption);

        // Debug builds prioritize fast startup (WASM compilation) over peak runtime perf.
        // This is particularly noticeable when iterating on games and restarting often.
        let opt_level = parse_opt_level_env().unwrap_or(if cfg!(debug_assertions) {
            OptLevel::None
        } else {
            OptLevel::Speed
        });
        config.cranelift_opt_level(opt_level);

        // Enable Wasmtime's on-disk compilation cache when available.
        // This can drastically reduce subsequent startup times for unchanged ROMs.
        match Cache::from_file(None) {
            Ok(cache) => {
                config.cache(Some(cache));
                tracing::debug!("Wasmtime compilation cache enabled");
            }
            Err(e) => {
                tracing::debug!("Wasmtime compilation cache disabled: {e}");
            }
        }

        let engine = Engine::new(&config).context("Failed to create WASM engine")?;
        Ok(Self {
            engine,
            epoch_ticker: None,
        })
    }

    /// Get a reference to the underlying wasmtime engine
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Load a WASM module from bytes
    pub fn load_module(&self, bytes: &[u8]) -> Result<Module> {
        Module::new(&self.engine, bytes).context("Failed to compile WASM module")
    }

    /// Validate that a WASM module's memory requirements fit console constraints
    ///
    /// Call this before instantiating a module to ensure it doesn't declare
    /// more memory than the console allows. This provides a clear error message
    /// rather than failing during instantiation.
    ///
    /// # Arguments
    /// * `module` - The compiled WASM module to validate
    /// * `ram_limit` - Maximum allowed memory in bytes (from ConsoleSpecs::ram_limit)
    ///
    /// # Returns
    /// * `Ok(())` if the module's memory requirements fit within the limit
    /// * `Err(...)` if the module requires too much memory
    pub fn validate_module_memory(module: &Module, ram_limit: usize) -> Result<()> {
        for export in module.exports() {
            if let ExternType::Memory(mem_type) = export.ty() {
                let min_pages = mem_type.minimum();
                let min_bytes = min_pages as usize * 65536; // WASM pages are 64KB

                if min_bytes > ram_limit {
                    anyhow::bail!(
                        "Module '{}' requires {} bytes ({} pages) minimum memory, \
                         but console only allows {} bytes. \
                         Reduce your game's memory usage or embedded assets.",
                        export.name(),
                        min_bytes,
                        min_pages,
                        ram_limit
                    );
                }

                // Warn if module declares no maximum (will be limited by host)
                if mem_type.maximum().is_none() {
                    tracing::debug!(
                        "Module memory '{}' has no maximum declared; \
                         host will limit to {} bytes",
                        export.name(),
                        ram_limit
                    );
                }
            }
        }
        Ok(())
    }
}

// NOTE: WasmEngine intentionally does not implement Default.
// The WASM engine initialization is fallible (wasmtime::Engine::default() can fail
// on unsupported platforms or with invalid configuration). Using WasmEngine::new()
// returns Result<Self> which properly propagates initialization errors.

fn parse_opt_level_env() -> Option<OptLevel> {
    let value = std::env::var("NETHERCORE_WASM_OPT_LEVEL").ok()?;
    match value.trim().to_ascii_lowercase().as_str() {
        "none" | "0" => Some(OptLevel::None),
        "speed" | "1" => Some(OptLevel::Speed),
        "speed_and_size" | "speedandsize" | "2" => Some(OptLevel::SpeedAndSize),
        _ => {
            tracing::warn!(
                "Invalid NETHERCORE_WASM_OPT_LEVEL value '{}'; expected none/speed/speed_and_size",
                value
            );
            None
        }
    }
}
