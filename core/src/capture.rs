//! Screenshot and GIF recording functionality.
//!
//! Provides console-agnostic screen capture capabilities for gameplay recording.
//! - Screenshots saved as PNG to `~/.nethercore/screenshots/`
//! - GIFs saved to `~/.nethercore/gifs/`
//!
//! Screenshots are signed with HMAC to verify they came from the Nethercore player
//! when uploaded to the platform.

use anyhow::{Context, Result};
use nethercore_shared::screenshot::{
    SCREENSHOT_SIGNATURE_KEYWORD, ScreenshotPayload, compute_pixel_hash, sign_screenshot,
};
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

/// Trait for graphics backends that support screen capture.
///
/// Implement this trait to enable screenshot and GIF recording functionality.
pub trait CaptureSupport {
    /// Get the wgpu device for staging buffer creation.
    fn device(&self) -> &wgpu::Device;

    /// Get the wgpu queue for command submission.
    fn queue(&self) -> &wgpu::Queue;

    /// Get the render target texture (game resolution, not window).
    fn render_target_texture(&self) -> &wgpu::Texture;

    /// Get the render target dimensions (width, height).
    fn render_target_dimensions(&self) -> (u32, u32);
}

/// Screen capture state manager.
///
/// Handles screenshot requests and GIF recording state.
pub struct ScreenCapture {
    /// Whether a screenshot has been requested for this frame
    screenshot_pending: bool,
    /// Active GIF recorder, if recording
    gif_recorder: Option<GifRecorder>,
    /// Channel for receiving save completion notifications
    save_receiver: Option<mpsc::Receiver<SaveResult>>,
    /// GIF settings
    gif_fps: u32,
    gif_max_seconds: u32,
    /// Game name for filename prefixes
    game_name: String,
    /// Console type for screenshot signing (e.g., "zx", "chroma")
    console_type: String,
}

/// GIF recorder state.
struct GifRecorder {
    /// Accumulated frames (RGBA pixels)
    frames: Vec<Vec<u8>>,
    /// Frame dimensions
    width: u32,
    height: u32,
    /// Recording start time
    start_time: std::time::Instant,
    /// Frame skip counter (to achieve target FPS)
    frame_skip_counter: u32,
    /// Target FPS for recording
    target_fps: u32,
    /// Game name for filename prefix
    game_name: String,
}

/// Result of a save operation.
pub enum SaveResult {
    Screenshot(Result<PathBuf>),
    Gif(Result<PathBuf>),
}

impl ScreenCapture {
    /// Create a new screen capture manager.
    ///
    /// # Arguments
    /// * `gif_fps` - Target FPS for GIF recording
    /// * `gif_max_seconds` - Maximum GIF recording duration
    /// * `game_name` - Used as a prefix in saved filenames
    /// * `console_type` - Console identifier for screenshot signing (e.g., "zx", "chroma")
    pub fn new(
        gif_fps: u32,
        gif_max_seconds: u32,
        game_name: String,
        console_type: String,
    ) -> Self {
        Self {
            screenshot_pending: false,
            gif_recorder: None,
            save_receiver: None,
            gif_fps,
            gif_max_seconds,
            game_name,
            console_type,
        }
    }

    /// Update the game name (e.g., after loading a new game).
    pub fn set_game_name(&mut self, name: String) {
        self.game_name = name;
    }

    /// Update the console type (e.g., after loading a new game).
    pub fn set_console_type(&mut self, console_type: String) {
        self.console_type = console_type;
    }

    /// Request a screenshot to be taken on the next frame.
    pub fn request_screenshot(&mut self) {
        self.screenshot_pending = true;
    }

    /// Toggle GIF recording on/off.
    ///
    /// If not recording, starts recording.
    /// If recording, stops and saves the GIF.
    pub fn toggle_recording(&mut self, width: u32, height: u32) {
        if self.gif_recorder.is_some() {
            self.stop_recording();
        } else {
            self.start_recording(width, height);
        }
    }

    /// Start GIF recording.
    fn start_recording(&mut self, width: u32, height: u32) {
        tracing::info!(
            "GIF recording started ({}x{}, {}fps)",
            width,
            height,
            self.gif_fps
        );
        self.gif_recorder = Some(GifRecorder {
            frames: Vec::new(),
            width,
            height,
            start_time: std::time::Instant::now(),
            frame_skip_counter: 0,
            target_fps: self.gif_fps,
            game_name: self.game_name.clone(),
        });
    }

    /// Stop GIF recording and save the file.
    fn stop_recording(&mut self) {
        if let Some(recorder) = self.gif_recorder.take() {
            let frame_count = recorder.frames.len();
            let duration = recorder.start_time.elapsed();
            tracing::info!(
                "GIF recording stopped: {} frames, {:.1}s",
                frame_count,
                duration.as_secs_f32()
            );

            if frame_count > 0 {
                // Save in background thread
                let (tx, rx) = mpsc::channel();
                self.save_receiver = Some(rx);

                thread::spawn(move || {
                    let result = save_gif(recorder);
                    let _ = tx.send(SaveResult::Gif(result));
                });
            }
        }
    }

    /// Check if currently recording.
    pub fn is_recording(&self) -> bool {
        self.gif_recorder.is_some()
    }

    /// Check if a capture (screenshot or GIF frame) is needed this frame.
    pub fn needs_capture(&self) -> bool {
        if self.screenshot_pending {
            return true;
        }
        if let Some(ref recorder) = self.gif_recorder {
            // Check if we need to capture a frame based on target FPS
            // Assuming game runs at 60fps, we capture every (60/target_fps) frames
            return recorder.frame_skip_counter == 0;
        }
        false
    }

    /// Process a captured frame.
    ///
    /// Call this with pixel data after rendering when `needs_capture()` returns true.
    pub fn process_frame(&mut self, pixels: Vec<u8>, width: u32, height: u32) {
        // Handle screenshot
        if self.screenshot_pending {
            self.screenshot_pending = false;

            // Save in background thread
            let screenshot_pixels = pixels.clone();
            let game_name = self.game_name.clone();
            let console_type = self.console_type.clone();
            let (tx, rx) = mpsc::channel();
            self.save_receiver = Some(rx);

            thread::spawn(move || {
                let result =
                    save_screenshot(screenshot_pixels, width, height, &game_name, &console_type);
                let _ = tx.send(SaveResult::Screenshot(result));
            });
        }

        // Handle GIF recording
        if let Some(ref mut recorder) = self.gif_recorder {
            // Frame skip logic for target FPS (assuming 60fps game)
            if recorder.frame_skip_counter == 0 {
                recorder.frames.push(pixels);

                // Check max duration
                let max_frames = (recorder.target_fps * self.gif_max_seconds) as usize;
                if recorder.frames.len() >= max_frames {
                    tracing::info!("GIF max duration reached, stopping recording");
                    self.stop_recording();
                    return;
                }
            }

            // Update frame skip counter (60fps game -> 30fps GIF = skip every other frame)
            let skip_interval = 60 / recorder.target_fps;
            recorder.frame_skip_counter = (recorder.frame_skip_counter + 1) % skip_interval;
        }
    }

    /// Poll for save completion results.
    ///
    /// Returns the result if a save operation has completed.
    pub fn poll_save_result(&mut self) -> Option<SaveResult> {
        if let Some(ref receiver) = self.save_receiver {
            match receiver.try_recv() {
                Ok(result) => {
                    self.save_receiver = None;
                    Some(result)
                }
                Err(mpsc::TryRecvError::Empty) => None,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.save_receiver = None;
                    None
                }
            }
        } else {
            None
        }
    }
}

/// Read pixels from a render target texture.
///
/// Copies the texture to a staging buffer and reads the pixel data.
pub fn read_render_target_pixels(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
) -> Vec<u8> {
    // Calculate buffer size with alignment
    // wgpu requires rows to be aligned to COPY_BYTES_PER_ROW_ALIGNMENT (256 bytes)
    let bytes_per_pixel = 4u32;
    let unpadded_bytes_per_row = width * bytes_per_pixel;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded_bytes_per_row = (unpadded_bytes_per_row + align - 1) / align * align;
    let buffer_size = (padded_bytes_per_row * height) as u64;

    // Create staging buffer
    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Screenshot Staging Buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    // Copy texture to buffer
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenshot Copy Encoder"),
    });

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    queue.submit(std::iter::once(encoder.finish()));

    // Map and read the buffer
    let slice = staging_buffer.slice(..);

    // Use pollster to block on the async map operation
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });

    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("Failed to poll device");

    // Wait for mapping to complete
    rx.recv()
        .expect("Failed to receive map result")
        .expect("Failed to map buffer");

    // Read data and remove row padding
    let mapped = slice.get_mapped_range();
    let mut pixels = Vec::with_capacity((width * height * bytes_per_pixel) as usize);

    for row in 0..height {
        let start = (row * padded_bytes_per_row) as usize;
        let end = start + (width * bytes_per_pixel) as usize;
        pixels.extend_from_slice(&mapped[start..end]);
    }

    drop(mapped);
    staging_buffer.unmap();

    pixels
}

/// Convenience function to read pixels from a graphics backend that implements CaptureSupport.
pub fn read_capture_pixels<G: CaptureSupport>(graphics: &G) -> Vec<u8> {
    let (width, height) = graphics.render_target_dimensions();
    read_render_target_pixels(
        graphics.device(),
        graphics.queue(),
        graphics.render_target_texture(),
        width,
        height,
    )
}

/// Get the screenshots directory, creating it if needed.
fn screenshots_dir() -> Result<PathBuf> {
    let dir = directories::ProjectDirs::from("io.nethercore", "", "Nethercore")
        .context("Failed to get project directories")?
        .data_dir()
        .join("screenshots");

    std::fs::create_dir_all(&dir).context("Failed to create screenshots directory")?;

    Ok(dir)
}

/// Get the GIFs directory, creating it if needed.
fn gifs_dir() -> Result<PathBuf> {
    let dir = directories::ProjectDirs::from("io.nethercore", "", "Nethercore")
        .context("Failed to get project directories")?
        .data_dir()
        .join("gifs");

    std::fs::create_dir_all(&dir).context("Failed to create gifs directory")?;

    Ok(dir)
}

/// Sanitize a game name for use in filenames.
///
/// Replaces spaces with underscores, removes special characters,
/// and converts to lowercase for consistent filenames.
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else if c == ' ' || c == '-' {
                '_'
            } else {
                '_'
            }
        })
        .collect::<String>()
        // Remove consecutive underscores
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

/// Generate a timestamped filename with game name prefix.
fn timestamped_filename(game_name: &str, suffix: &str, extension: &str) -> String {
    let now = chrono::Local::now();
    let sanitized = sanitize_filename(game_name);
    format!(
        "{}_{}_{}.{}",
        sanitized,
        suffix,
        now.format("%Y-%m-%d_%H-%M-%S"),
        extension
    )
}

/// Save screenshot as PNG with embedded signature for origin verification.
///
/// The signature is embedded as a PNG iTXt chunk and can be verified by the
/// platform backend to ensure the screenshot came from the Nethercore player.
fn save_screenshot(
    pixels: Vec<u8>,
    width: u32,
    height: u32,
    game_name: &str,
    console_type: &str,
) -> Result<PathBuf> {
    let dir = screenshots_dir()?;
    let filename = timestamped_filename(game_name, "screenshot", "png");
    let path = dir.join(&filename);

    // Compute pixel hash for the signature
    let pixel_hash = compute_pixel_hash(&pixels);

    // Create and sign the payload
    let payload = ScreenshotPayload::new(&pixel_hash, console_type, width, height);
    let signed = sign_screenshot(&payload).context("Failed to sign screenshot")?;
    let signed_json = signed.to_json().context("Failed to serialize signature")?;

    // Create PNG file with signature embedded as iTXt chunk
    let file = File::create(&path).context("Failed to create screenshot file")?;
    let ref mut writer = BufWriter::new(file);

    let mut encoder = png::Encoder::new(writer, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    // Add signature as iTXt chunk
    encoder
        .add_itxt_chunk(SCREENSHOT_SIGNATURE_KEYWORD.to_string(), signed_json)
        .context("Failed to add signature chunk")?;

    let mut png_writer = encoder
        .write_header()
        .context("Failed to write PNG header")?;
    png_writer
        .write_image_data(&pixels)
        .context("Failed to write PNG data")?;

    tracing::info!("Screenshot saved: {}", path.display());

    Ok(path)
}

/// Save GIF recording.
fn save_gif(recorder: GifRecorder) -> Result<PathBuf> {
    let dir = gifs_dir()?;
    let filename = timestamped_filename(&recorder.game_name, "recording", "gif");
    let path = dir.join(&filename);

    let file = std::fs::File::create(&path).context("Failed to create GIF file")?;

    let mut encoder = gif::Encoder::new(file, recorder.width as u16, recorder.height as u16, &[])
        .context("Failed to create GIF encoder")?;

    // Set repeat count (0 = infinite loop)
    encoder
        .set_repeat(gif::Repeat::Infinite)
        .context("Failed to set GIF repeat")?;

    // Calculate frame delay (in centiseconds, 100ths of a second)
    // For 30fps: 1000ms / 30 = 33.33ms per frame = 3.33 centiseconds ≈ 3
    let frame_delay = (100 / recorder.target_fps) as u16;

    for frame_pixels in recorder.frames {
        // Convert RGBA to RGB for GIF (GIF doesn't support alpha well)
        let mut rgb_pixels: Vec<u8> = Vec::with_capacity(frame_pixels.len() * 3 / 4);
        for chunk in frame_pixels.chunks(4) {
            rgb_pixels.push(chunk[0]); // R
            rgb_pixels.push(chunk[1]); // G
            rgb_pixels.push(chunk[2]); // B
        }

        let mut frame =
            gif::Frame::from_rgb(recorder.width as u16, recorder.height as u16, &rgb_pixels);
        frame.delay = frame_delay;

        encoder
            .write_frame(&frame)
            .context("Failed to write GIF frame")?;
    }

    tracing::info!("GIF saved: {}", path.display());

    Ok(path)
}
