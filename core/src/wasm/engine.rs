//! WASM engine wrapper for loading and compiling modules

use anyhow::{Context, Result};
use wasmtime::{Engine, ExternType, Module};

/// Shared WASM engine (one per application)
pub struct WasmEngine {
    engine: Engine,
}

impl WasmEngine {
    /// Create a new WASM engine with default configuration
    pub fn new() -> Result<Self> {
        let engine = Engine::default();
        Ok(Self { engine })
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
