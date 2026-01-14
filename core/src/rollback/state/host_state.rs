//! Host-side rollback state that lives outside WASM memory

/// Size of HostRollbackState in bytes (for inline storage)
pub const HOST_STATE_SIZE: usize = std::mem::size_of::<HostRollbackState>();

/// Host-side state that must be rolled back for determinism
///
/// This state lives on the host (not in WASM memory) but affects game
/// simulation and must be restored during rollback.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
#[repr(C)]
pub struct HostRollbackState {
    /// RNG state for deterministic random numbers
    pub rng_state: u64,
    /// Current tick count
    pub tick_count: u64,
    /// Elapsed time in seconds (f32 stored as bits for Pod compatibility)
    pub elapsed_time_bits: u32,
    /// Padding for alignment
    _padding: u32,
}

// SAFETY: HostRollbackState is #[repr(C)] with only primitive types
unsafe impl bytemuck::Zeroable for HostRollbackState {}
unsafe impl bytemuck::Pod for HostRollbackState {}

impl HostRollbackState {
    /// Create from game state values
    pub fn new(rng_state: u64, tick_count: u64, elapsed_time: f32) -> Self {
        Self {
            rng_state,
            tick_count,
            elapsed_time_bits: elapsed_time.to_bits(),
            _padding: 0,
        }
    }

    /// Get elapsed time as f32
    pub fn elapsed_time(&self) -> f32 {
        f32::from_bits(self.elapsed_time_bits)
    }
}
