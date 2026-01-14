//! Auto-growing GPU buffer implementation
//!
//! Provides dynamic buffer growth during initialization phase.

/// Initial buffer size (64KB)
const INITIAL_BUFFER_SIZE: u64 = 64 * 1024;

/// Growth factor when buffer needs to expand (2x)
const BUFFER_GROWTH_FACTOR: u64 = 2;

/// Auto-growing GPU buffer for vertex/index data
///
/// Grows dynamically during init phase when more data is needed.
/// Avoids frequent reallocation by doubling capacity on growth.
pub struct GrowableBuffer {
    /// The wgpu buffer
    buffer: wgpu::Buffer,
    /// Buffer usage flags
    usage: wgpu::BufferUsages,
    /// Current capacity in bytes
    capacity: u64,
    /// Current used size in bytes
    used: u64,
    /// Debug label
    label: String,
}

impl GrowableBuffer {
    /// Create a new growable buffer with initial capacity
    pub fn new(device: &wgpu::Device, usage: wgpu::BufferUsages, label: &str) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: INITIAL_BUFFER_SIZE,
            usage: usage | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            usage,
            capacity: INITIAL_BUFFER_SIZE,
            used: 0,
            label: label.to_string(),
        }
    }

    /// Ensure the buffer has enough capacity for additional bytes
    ///
    /// If the buffer needs to grow, creates a new larger buffer and copies existing data.
    /// Returns true if the buffer was grown, false otherwise.
    pub fn ensure_capacity(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        additional_bytes: u64,
    ) -> bool {
        let required = self.used + additional_bytes;
        if required <= self.capacity {
            return false;
        }

        // Calculate new capacity (at least double, or enough for required)
        let mut new_capacity = self.capacity * BUFFER_GROWTH_FACTOR;
        while new_capacity < required {
            new_capacity *= BUFFER_GROWTH_FACTOR;
        }

        tracing::debug!(
            "Growing buffer '{}': {} -> {} bytes (preserving {} bytes)",
            self.label,
            self.capacity,
            new_capacity,
            self.used
        );

        // Create new buffer with COPY_SRC for data preservation
        let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&self.label),
            size: new_capacity,
            usage: self.usage | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Copy existing data to new buffer
        if self.used > 0 {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Buffer Grow Copy"),
            });
            encoder.copy_buffer_to_buffer(&self.buffer, 0, &new_buffer, 0, self.used);
            queue.submit(std::iter::once(encoder.finish()));
        }

        self.buffer = new_buffer;
        self.capacity = new_capacity;
        // CRITICAL: Don't reset used! Data has been preserved.

        true
    }

    /// Write data to the buffer at the current position
    ///
    /// Returns the byte offset where data was written.
    /// Panics if there's not enough capacity (call ensure_capacity first).
    ///
    /// **Alignment**: After writing, `self.used` is aligned to the next 4-byte boundary
    /// to satisfy wgpu's COPY_BUFFER_ALIGNMENT requirement for subsequent writes.
    pub fn write(&mut self, queue: &wgpu::Queue, data: &[u8]) -> u64 {
        let offset = self.used;
        assert!(
            offset + data.len() as u64 <= self.capacity,
            "Buffer overflow: {} + {} > {}",
            offset,
            data.len(),
            self.capacity
        );

        queue.write_buffer(&self.buffer, offset, data);

        // Align to next 4-byte boundary for wgpu COPY_BUFFER_ALIGNMENT
        // This prevents misalignment when multiple meshes are packed together
        let bytes_written = data.len() as u64;
        let aligned_size = (bytes_written + 3) & !3;
        self.used += aligned_size;

        offset
    }

    /// Write data to the buffer at a specific offset
    ///
    /// Updates the used counter if this write extends past the current end.
    /// Panics if offset + data.len > capacity.
    pub fn write_at(&self, queue: &wgpu::Queue, offset: u64, data: &[u8]) {
        assert!(
            offset + data.len() as u64 <= self.capacity,
            "Buffer overflow: {} + {} > {}",
            offset,
            data.len(),
            self.capacity
        );
        queue.write_buffer(&self.buffer, offset, data);
    }

    /// Reset the used counter (for per-frame immediate mode buffers)
    pub fn reset(&mut self) {
        self.used = 0;
    }

    /// Get the underlying wgpu buffer, if it exists
    pub fn buffer(&self) -> Option<&wgpu::Buffer> {
        Some(&self.buffer)
    }

    /// Get current used bytes
    pub fn used(&self) -> u64 {
        self.used
    }

    /// Get current capacity in bytes
    pub fn capacity(&self) -> u64 {
        self.capacity
    }
}
