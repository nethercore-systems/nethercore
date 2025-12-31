//! Generic asset preview application for any console
//!
//! Provides a console-agnostic preview application that can browse ROM assets
//! without executing WASM code. Implements ConsoleApp trait for integration
//! with the existing event loop infrastructure.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

use crate::capture::CaptureSupport;
use crate::console::{Console, Graphics};

use super::event_loop::ConsoleApp;
use super::standalone::StandaloneGraphicsSupport;
use super::types::RuntimeError;

/// Configuration for preview mode.
#[derive(Clone)]
pub struct PreviewConfig {
    /// ROM file path
    pub rom_path: PathBuf,
    /// Initial asset to focus (optional)
    pub initial_asset: Option<String>,
}

/// Metadata extracted from ROM for preview.
#[derive(Clone, Default)]
pub struct PreviewMetadata {
    pub id: String,
    pub title: String,
    pub author: String,
    pub version: String,
}

/// Loaded preview data (assets only, no WASM).
pub struct PreviewData<D> {
    /// Data pack containing all assets
    pub data_pack: D,
    /// Game metadata
    pub metadata: PreviewMetadata,
}

/// Trait for preview-specific ROM loading.
///
/// Unlike RomLoader which prepares for WASM execution, PreviewRomLoader
/// only extracts the data pack for asset browsing.
pub trait PreviewRomLoader: Sized + 'static {
    /// Console type this loader works with.
    type Console: Console + Clone;

    /// Data pack type containing assets.
    type DataPack: Send + 'static;

    /// Viewer type for this console's assets.
    type Viewer: AssetViewer<Self::Console, Self::DataPack>;

    /// Load ROM and extract data pack for preview (skip WASM init).
    fn load_for_preview(path: &Path) -> Result<PreviewData<Self::DataPack>>;
}

/// Asset category for navigation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AssetCategory {
    Textures,
    Meshes,
    Sounds,
    Trackers,
    Animations,
    Fonts,
    Data,
}

impl AssetCategory {
    pub fn all() -> &'static [AssetCategory] {
        &[
            AssetCategory::Textures,
            AssetCategory::Meshes,
            AssetCategory::Sounds,
            AssetCategory::Trackers,
            AssetCategory::Animations,
            AssetCategory::Fonts,
            AssetCategory::Data,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            AssetCategory::Textures => "Textures",
            AssetCategory::Meshes => "Meshes",
            AssetCategory::Sounds => "Sounds",
            AssetCategory::Trackers => "Trackers",
            AssetCategory::Animations => "Animations",
            AssetCategory::Fonts => "Fonts",
            AssetCategory::Data => "Data",
        }
    }
}

/// Trait for console-specific asset viewers.
pub trait AssetViewer<C: Console, D>: Sized + 'static {
    /// Create a new viewer for the given data pack.
    fn new(data: &PreviewData<D>) -> Self;

    /// Get list of asset IDs for a category.
    fn asset_ids(&self, category: AssetCategory) -> Vec<String>;

    /// Get count of assets in a category.
    fn asset_count(&self, category: AssetCategory) -> usize;

    /// Select an asset for viewing.
    fn select_asset(&mut self, category: AssetCategory, id: &str);

    /// Get currently selected asset info string.
    fn selected_info(&self) -> Option<String>;

    /// Render the viewer UI.
    fn render_ui(&mut self, ctx: &egui::Context, graphics: &mut C::Graphics);

    /// Update viewer state (called each frame for animations, audio, etc.)
    fn update(&mut self, dt: f32);
}

/// Generic preview application.
pub struct PreviewApp<C, L>
where
    C: Console + Clone,
    C::Graphics: StandaloneGraphicsSupport,
    L: PreviewRomLoader<Console = C>,
{
    config: PreviewConfig,
    window: Option<Arc<Window>>,
    graphics: Option<C::Graphics>,
    viewer: Option<L::Viewer>,
    #[allow(dead_code)]
    data_pack: Option<L::DataPack>,
    metadata: PreviewMetadata,

    // UI state
    egui_ctx: egui::Context,
    egui_state: Option<egui_winit::State>,
    egui_renderer: Option<egui_wgpu::Renderer>,

    // Navigation
    selected_category: AssetCategory,
    selected_asset: Option<String>,
    search_filter: String,

    // Event loop state
    needs_redraw: bool,
    should_exit: bool,
    last_update: Instant,

    _loader_marker: std::marker::PhantomData<L>,
}

impl<C, L> PreviewApp<C, L>
where
    C: Console + Clone,
    C::Graphics: StandaloneGraphicsSupport,
    L: PreviewRomLoader<Console = C>,
{
    /// Create a new preview app with the given configuration.
    pub fn new(config: PreviewConfig) -> Self {
        Self {
            config,
            window: None,
            graphics: None,
            viewer: None,
            data_pack: None,
            metadata: PreviewMetadata::default(),
            egui_ctx: egui::Context::default(),
            egui_state: None,
            egui_renderer: None,
            selected_category: AssetCategory::Textures,
            selected_asset: None,
            search_filter: String::new(),
            needs_redraw: true,
            should_exit: false,
            last_update: Instant::now(),
            _loader_marker: std::marker::PhantomData,
        }
    }

    fn handle_key_input(&mut self, event: KeyEvent) {
        if event.state == ElementState::Pressed
            && let PhysicalKey::Code(KeyCode::Escape) = event.physical_key
        {
            self.should_exit = true;
        }
    }

    fn render_ui(&mut self) {
        let Some(window) = &self.window else { return };
        let Some(graphics) = &mut self.graphics else {
            return;
        };
        let Some(egui_state) = &mut self.egui_state else {
            return;
        };
        let Some(egui_renderer) = &mut self.egui_renderer else {
            return;
        };

        let raw_input = egui_state.take_egui_input(window);

        // Capture state for closure
        let mut new_category = self.selected_category;
        let mut new_asset = self.selected_asset.clone();
        let mut exit_requested = false;
        let search_filter = self.search_filter.clone();
        let metadata = self.metadata.clone();

        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            // Top panel with category tabs
            egui::TopBottomPanel::top("categories").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for cat in AssetCategory::all() {
                        let count = self
                            .viewer
                            .as_ref()
                            .map(|v| v.asset_count(*cat))
                            .unwrap_or(0);
                        let label = format!("{} ({})", cat.label(), count);
                        if ui
                            .selectable_label(self.selected_category == *cat, &label)
                            .clicked()
                        {
                            new_category = *cat;
                            new_asset = None;
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("X").clicked() {
                            exit_requested = true;
                        }
                    });
                });
            });

            // Bottom status bar
            egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("ROM: {}", metadata.title));
                    ui.separator();
                    if let Some(v) = self.viewer.as_ref() {
                        let total: usize =
                            AssetCategory::all().iter().map(|c| v.asset_count(*c)).sum();
                        ui.label(format!("{} assets", total));
                    }
                });
            });

            // Left panel with asset list
            egui::SidePanel::left("assets")
                .default_width(200.0)
                .show(ctx, |ui| {
                    let mut filter = search_filter.clone();
                    ui.add(
                        egui::TextEdit::singleline(&mut filter)
                            .hint_text("Search...")
                            .desired_width(f32::INFINITY),
                    );
                    self.search_filter = filter;
                    ui.separator();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if let Some(v) = self.viewer.as_ref() {
                            for id in v.asset_ids(self.selected_category) {
                                if !self.search_filter.is_empty()
                                    && !id
                                        .to_lowercase()
                                        .contains(&self.search_filter.to_lowercase())
                                {
                                    continue;
                                }
                                let is_selected = self.selected_asset.as_ref() == Some(&id);
                                if ui.selectable_label(is_selected, &id).clicked() {
                                    new_asset = Some(id.clone());
                                }
                            }
                        }
                    });
                });

            // Central panel with preview
            egui::CentralPanel::default().show(ctx, |_ui| {
                if let Some(v) = self.viewer.as_mut() {
                    v.render_ui(ctx, graphics);
                }
            });
        });

        // Apply UI state changes
        if new_category != self.selected_category {
            self.selected_category = new_category;
            self.selected_asset = None;
        }
        if new_asset != self.selected_asset {
            if let Some(ref id) = new_asset
                && let Some(viewer) = &mut self.viewer
            {
                viewer.select_asset(self.selected_category, id);
            }
            self.selected_asset = new_asset;
        }
        if exit_requested {
            self.should_exit = true;
        }

        egui_state.handle_platform_output(window, full_output.platform_output);

        // Render egui
        let surface_texture = match graphics.get_current_texture() {
            Ok(tex) => tex,
            Err(_) => return,
        };

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            graphics
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Preview Encoder"),
                });

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [graphics.width(), graphics.height()],
            pixels_per_point: window.scale_factor() as f32,
        };

        let tris = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        for (id, delta) in &full_output.textures_delta.set {
            egui_renderer.update_texture(graphics.device(), graphics.queue(), *id, delta);
        }

        egui_renderer.update_buffers(
            graphics.device(),
            graphics.queue(),
            &mut encoder,
            &tris,
            &screen_descriptor,
        );

        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Preview Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            let mut render_pass_static = render_pass.forget_lifetime();
            egui_renderer.render(&mut render_pass_static, &tris, &screen_descriptor);
        }

        for id in &full_output.textures_delta.free {
            egui_renderer.free_texture(id);
        }

        graphics.queue().submit(std::iter::once(encoder.finish()));
        surface_texture.present();
    }
}

impl<C, L> ConsoleApp<C> for PreviewApp<C, L>
where
    C: Console + Clone + Default,
    C::Graphics: StandaloneGraphicsSupport,
    L: PreviewRomLoader<Console = C>,
{
    fn on_window_created(
        &mut self,
        window: Arc<Window>,
        _event_loop: &ActiveEventLoop,
    ) -> Result<()> {
        window.set_title(&format!("{} - Asset Preview", C::specs().name));

        // Load ROM for preview
        let preview_data =
            L::load_for_preview(&self.config.rom_path).context("Failed to load ROM for preview")?;

        self.metadata = preview_data.metadata.clone();
        window.set_title(&format!(
            "{} - {} - Asset Preview",
            C::specs().name,
            self.metadata.title
        ));

        // Create graphics
        let console = C::default();
        let graphics = console
            .create_graphics(window.clone())
            .context("Failed to create graphics")?;

        // Create viewer
        let viewer = L::Viewer::new(&preview_data);

        // Initialize egui
        let egui_state = egui_winit::State::new(
            self.egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );
        let egui_renderer = egui_wgpu::Renderer::new(
            graphics.device(),
            graphics.surface_format(),
            egui_wgpu::RendererOptions::default(),
        );

        self.graphics = Some(graphics);
        self.data_pack = Some(preview_data.data_pack);
        self.viewer = Some(viewer);
        self.egui_state = Some(egui_state);
        self.egui_renderer = Some(egui_renderer);
        self.window = Some(window);
        self.last_update = Instant::now();

        // Select initial asset if specified
        if let Some(asset) = &self.config.initial_asset
            && let Some(viewer) = &mut self.viewer
        {
            for cat in AssetCategory::all() {
                if viewer.asset_ids(*cat).contains(asset) {
                    self.selected_category = *cat;
                    self.selected_asset = Some(asset.clone());
                    viewer.select_asset(*cat, asset);
                    break;
                }
            }
        }

        tracing::info!("Preview mode loaded: {}", self.config.rom_path.display());
        Ok(())
    }

    fn on_window_event(&mut self, event: &WindowEvent) -> bool {
        if let (Some(egui_state), Some(window)) = (&mut self.egui_state, &self.window) {
            let response = egui_state.on_window_event(window, event);
            if response.consumed {
                self.needs_redraw = true;
                return true;
            }
            if response.repaint {
                self.needs_redraw = true;
            }
        }

        match event {
            WindowEvent::Resized(size) => {
                if let Some(graphics) = &mut self.graphics {
                    graphics.resize(size.width, size.height);
                }
                self.needs_redraw = true;
                false
            }
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                self.handle_key_input(key_event.clone());
                false
            }
            _ => false,
        }
    }

    fn next_tick(&self) -> Instant {
        // Preview mode runs at ~30 FPS to save power
        self.last_update + Duration::from_millis(33)
    }

    fn advance_simulation(&mut self) {
        let now = Instant::now();
        let dt = (now - self.last_update).as_secs_f32();
        self.last_update = now;

        // Update viewer (for animations, audio playback, etc.)
        if let Some(viewer) = &mut self.viewer {
            viewer.update(dt);
        }

        self.needs_redraw = true;
    }

    fn update_next_tick(&mut self) {
        // next_tick is computed dynamically in next_tick()
    }

    fn render(&mut self) {
        self.render_ui();
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn mark_needs_redraw(&mut self) {
        self.needs_redraw = true;
    }

    fn clear_needs_redraw(&mut self) {
        self.needs_redraw = false;
    }

    fn on_runtime_error(&mut self, error: RuntimeError) {
        tracing::error!("Preview error: {}", error.0);
    }

    fn should_exit(&self) -> bool {
        self.should_exit
    }

    fn request_exit(&mut self) {
        self.should_exit = true;
    }

    fn request_redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

/// Run a preview application for the given console.
pub fn run_preview<C, L>(config: PreviewConfig) -> Result<()>
where
    C: Console + Clone + Default,
    C::Graphics: StandaloneGraphicsSupport,
    L: PreviewRomLoader<Console = C>,
{
    let app = PreviewApp::<C, L>::new(config);
    super::run(app)
}
