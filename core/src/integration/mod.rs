//! Integration tests for Nethercore core framework
//!
//! Tests full game lifecycle, rollback simulation, input synchronization,
//! and resource limit enforcement.

#[cfg(test)]
mod full_integration_tests;
#[cfg(test)]
mod input_tests;
#[cfg(test)]
mod lifecycle_tests;
#[cfg(test)]
mod rollback_tests;

#[cfg(test)]
pub(crate) mod test_utils {
    use wasmtime::Linker;

    use crate::{
        console::Console,
        test_utils::{TestConsole, TestInput},
        wasm::{GameInstance, WasmEngine, WasmGameContext},
    };

    /// Create a test engine with common FFI registered
    pub fn create_test_engine() -> (WasmEngine, Linker<WasmGameContext<TestInput, (), ()>>) {
        let engine = WasmEngine::new().unwrap();
        let mut linker = Linker::new(engine.engine());
        crate::ffi::register_common_ffi(&mut linker).unwrap();
        (engine, linker)
    }

    pub fn test_ram_limit() -> usize {
        TestConsole::specs().ram_limit
    }

    pub fn new_test_game_instance(
        engine: &WasmEngine,
        module: &wasmtime::Module,
        linker: &Linker<WasmGameContext<TestInput, (), ()>>,
    ) -> GameInstance<TestInput, ()> {
        GameInstance::with_ram_limit(engine, module, linker, test_ram_limit()).unwrap()
    }
}
