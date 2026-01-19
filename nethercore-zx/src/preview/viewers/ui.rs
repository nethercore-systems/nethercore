//! UI rendering implementation for asset viewer

use half::f16;

use super::{AssetCategory, ZXAssetViewer};

impl ZXAssetViewer {
    /// Render the main UI for the asset viewer
    pub(super) fn render_ui_impl(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Asset Preview");

            if let Some(id) = &self.selected_id {
                let id_owned = id.clone(); // Clone to avoid borrow issues
                ui.label(format!("Selected: {}", id_owned));

                if let Some(info) = self.selected_info() {
                    ui.label(info);
                }

                ui.separator();

                match self.selected_category {
                    AssetCategory::Textures => {
                        self.render_texture_ui(ctx, ui, &id_owned);
                    }
                    AssetCategory::Sounds => {
                        self.render_sound_ui(ui);
                    }
                    AssetCategory::Meshes => {
                        self.render_mesh_ui(ui);
                    }
                    AssetCategory::Animations => {
                        self.render_animation_ui(ui);
                    }
                    AssetCategory::Trackers => {
                        self.render_tracker_ui(ui);
                    }
                    AssetCategory::Skeletons => {
                        self.render_skeleton_ui(ui);
                    }
                    AssetCategory::Fonts => {
                        self.render_font_ui(ctx, ui, &id_owned);
                    }
                    AssetCategory::Data => {
                        self.render_data_ui(ui);
                    }
                }
            } else {
                ui.label("No asset selected");
                ui.label("Select an asset from the category tabs above");
            }
        });
    }

    fn render_texture_ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, id_owned: &str) {
        ui.heading("Texture Preview");

        // Extract values to avoid borrow issues
        let zoom = self.texture_zoom;

        ui.horizontal(|ui| {
            if ui.button("Zoom In").clicked() {
                self.texture_zoom_in();
            }
            if ui.button("Zoom Out").clicked() {
                self.texture_zoom_out();
            }
            if ui.button("Reset").clicked() {
                self.texture_reset_zoom();
            }
            ui.label(format!("Zoom: {:.1}x", zoom));

            ui.separator();

            if ui
                .button(if self.texture_linear_filter {
                    "Linear"
                } else {
                    "Nearest"
                })
                .clicked()
            {
                self.texture_linear_filter = !self.texture_linear_filter;
                // Invalidate cache to recreate texture with new filter
                self.cached_texture = None;
                self.cached_texture_id = None;
            }
        });

        ui.separator();

        // Render the texture - use cached texture to avoid recreating every frame
        if let Some(texture) = self.selected_texture() {
            let width = texture.width;
            let height = texture.height;
            let texture_data = texture.data.clone();

            // Update cache if needed
            self.update_texture_cache(
                ctx,
                id_owned,
                &format!("preview_{}", id_owned),
                width as u32,
                height as u32,
                &texture_data,
            );

            // Use the cached texture
            if let Some(ref texture_handle) = self.cached_texture {
                let display_size = egui::vec2(width as f32 * zoom, height as f32 * zoom);
                ui.add(egui::Image::new(texture_handle).fit_to_exact_size(display_size));
            }
        } else {
            ui.label("Failed to load texture");
        }
    }

    fn render_sound_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Sound Preview");

        // Extract sound data to avoid borrow issues
        let sound_data = self.selected_sound().map(|s| s.data.clone());
        let is_playing = self.sound_playing;
        let progress = self.sound_progress();

        if let Some(sound_samples) = sound_data {
            ui.horizontal(|ui| {
                if is_playing {
                    if ui.button("⏸ Stop").clicked() {
                        self.sound_stop();
                    }
                } else if ui.button("▶ Play").clicked() {
                    self.sound_toggle_play();
                }
                ui.label(format!("Position: {:.1}%", progress * 100.0));
            });

            ui.separator();

            // Draw waveform
            let available_width = ui.available_width();
            let waveform_height = 200.0;

            let (rect, _response) = ui.allocate_exact_size(
                egui::vec2(available_width, waveform_height),
                egui::Sense::hover(),
            );

            if ui.is_rect_visible(rect) {
                let painter = ui.painter();

                // Background
                painter.rect_filled(rect, 0.0, egui::Color32::from_gray(20));

                // Center line
                let center_y = rect.center().y;
                painter.line_segment(
                    [
                        rect.left_top() + egui::vec2(0.0, waveform_height / 2.0),
                        rect.right_top() + egui::vec2(0.0, waveform_height / 2.0),
                    ],
                    egui::Stroke::new(1.0, egui::Color32::from_gray(60)),
                );

                // Draw waveform
                let samples_per_pixel =
                    (sound_samples.len() as f32 / available_width).max(1.0) as usize;

                for x in 0..(available_width as usize) {
                    let sample_idx = x * samples_per_pixel;
                    if sample_idx >= sound_samples.len() {
                        break;
                    }

                    // Get min/max samples in this pixel range
                    let mut min_sample = i16::MAX;
                    let mut max_sample = i16::MIN;

                    for i in 0..samples_per_pixel {
                        let idx = sample_idx + i;
                        if idx >= sound_samples.len() {
                            break;
                        }
                        let sample = sound_samples[idx];
                        min_sample = min_sample.min(sample);
                        max_sample = max_sample.max(sample);
                    }

                    // Normalize to -1.0 to 1.0
                    let min_norm = min_sample as f32 / i16::MAX as f32;
                    let max_norm = max_sample as f32 / i16::MAX as f32;

                    // Convert to screen space
                    let min_y = center_y - min_norm * (waveform_height / 2.0);
                    let max_y = center_y - max_norm * (waveform_height / 2.0);

                    let x_pos = rect.left() + x as f32;

                    // Draw line for this pixel column
                    painter.line_segment(
                        [egui::pos2(x_pos, min_y), egui::pos2(x_pos, max_y)],
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 200, 255)),
                    );
                }

                // Draw playback position
                if is_playing {
                    let pos_x = rect.left() + progress * available_width;
                    painter.line_segment(
                        [
                            egui::pos2(pos_x, rect.top()),
                            egui::pos2(pos_x, rect.bottom()),
                        ],
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 100, 100)),
                    );
                }
            }
        } else {
            ui.label("Failed to load sound");
        }
    }

    fn render_mesh_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Mesh Preview");

        if let Some(mesh) = self.selected_mesh() {
            ui.label(format!("Vertices: {}", mesh.vertex_count));
            ui.label(format!("Indices: {}", mesh.index_count));
            ui.label(format!("Stride: {} bytes", mesh.stride()));
            ui.label(format!(
                "Total size: {} bytes",
                mesh.vertex_data.len() + mesh.index_data.len() * 2
            ));

            let mut flags = Vec::new();
            if mesh.has_uv() {
                flags.push("UV");
            }
            if mesh.has_color() {
                flags.push("Color");
            }
            if mesh.has_normal() {
                flags.push("Normal");
            }
            if mesh.is_skinned() {
                flags.push("Skinned");
            }
            ui.label(format!("Features: {}", flags.join(", ")));

            ui.separator();

            // Show vertex data format
            ui.label("Vertex Format:");
            ui.indent("vertex_format", |ui| {
                ui.label("• Position (xyz): 12 bytes");
                if mesh.has_uv() {
                    ui.label("• UV (uv): 8 bytes");
                }
                if mesh.has_color() {
                    ui.label("• Color (rgba): 4 bytes");
                }
                if mesh.has_normal() {
                    ui.label("• Normal (xyz): 12 bytes");
                }
                if mesh.is_skinned() {
                    ui.label("• Bone Indices (4×u8): 4 bytes");
                    ui.label("• Bone Weights (4×u8): 4 bytes");
                }
            });

            ui.separator();

            // Show sample vertices
            ui.label("Sample Vertex Data (first 3 vertices):");
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    let stride = mesh.stride();
                    for i in 0..3.min(mesh.vertex_count as usize) {
                        let offset = i * stride;
                        if offset + 8 <= mesh.vertex_data.len() {
                            ui.collapsing(format!("Vertex {}", i), |ui| {
                                // Read position (first 8 bytes: f16x4)
                                let pos_bytes = &mesh.vertex_data[offset..offset + 8];
                                let x = f16::from_le_bytes([pos_bytes[0], pos_bytes[1]]).to_f32();
                                let y = f16::from_le_bytes([pos_bytes[2], pos_bytes[3]]).to_f32();
                                let z = f16::from_le_bytes([pos_bytes[4], pos_bytes[5]]).to_f32();
                                ui.code(format!("pos: ({:.3}, {:.3}, {:.3})", x, y, z));
                            });
                        }
                    }
                });

            ui.separator();

            // Show sample indices
            ui.label("Index Data (first 12 indices):");
            let sample_indices: Vec<String> = mesh
                .index_data
                .iter()
                .take(12)
                .map(|idx| format!("{}", idx))
                .collect();
            ui.code(sample_indices.join(", "));
            if mesh.index_data.len() > 12 {
                ui.label("...");
            }
        } else {
            ui.label("Failed to load mesh");
        }
    }

    fn render_animation_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Animation Preview");

        // Extract animation data to avoid borrow issues
        let anim_data = self
            .selected_animation()
            .map(|a| (a.frame_count, a.bone_count, a.data.clone()));

        if let Some((frame_count, bone_count, keyframe_data)) = anim_data {
            let keyframe_data_size = keyframe_data.len();
            let is_playing = self.animation_playing;
            let current_frame = self.animation_frame;

            ui.horizontal(|ui| {
                if ui
                    .button(if is_playing { "⏸ Pause" } else { "▶ Play" })
                    .clicked()
                {
                    self.animation_playing = !self.animation_playing;
                }
                ui.label(format!("Frame: {}/{}", current_frame, frame_count));
            });

            ui.separator();

            ui.label(format!("Bones: {}", bone_count));
            ui.label(format!("Frames: {}", frame_count));
            ui.label(format!(
                "Duration: {:.2}s @ 30fps",
                frame_count as f32 / 30.0
            ));
            ui.label(format!("Keyframe data: {} bytes", keyframe_data_size));

            ui.separator();

            // Show animation format info
            ui.label("Format:");
            ui.indent("anim_format", |ui| {
                ui.label(format!("• {} bone matrices per frame", bone_count));
                ui.label("• 12 floats per matrix (3×4)");
                ui.label(format!("• {} bytes per frame", bone_count * 12 * 4));
            });

            ui.separator();

            // Progress bar
            ui.label("Playback Progress:");
            let progress = if frame_count > 0 {
                current_frame as f32 / frame_count as f32
            } else {
                0.0
            };
            ui.add(egui::ProgressBar::new(progress).show_percentage());

            ui.separator();

            // Show sample bone transform from current frame
            ui.label(format!("Sample: Bone 0 at frame {}", current_frame));
            if bone_count > 0 && frame_count > 0 {
                let frame_idx = (current_frame as usize) % (frame_count as usize);
                let bone_idx = 0;
                let matrix_offset = (frame_idx * bone_count as usize + bone_idx) * 12 * 4;

                if matrix_offset + 48 <= keyframe_data_size {
                    ui.collapsing("Transform Matrix", |ui| {
                        let data = &keyframe_data[matrix_offset..matrix_offset + 48];
                        let mut floats = Vec::new();
                        for chunk in data.chunks(4) {
                            floats
                                .push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
                        }
                        ui.code(format!(
                            "[ {:.3}, {:.3}, {:.3}, {:.3} ]\n\
                             [ {:.3}, {:.3}, {:.3}, {:.3} ]\n\
                             [ {:.3}, {:.3}, {:.3}, {:.3} ]",
                            floats[0],
                            floats[1],
                            floats[2],
                            floats[3],
                            floats[4],
                            floats[5],
                            floats[6],
                            floats[7],
                            floats[8],
                            floats[9],
                            floats[10],
                            floats[11],
                        ));
                    });
                }
            }
        } else {
            ui.label("Failed to load animation");
        }
    }

    fn render_tracker_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Tracker Preview");

        if let Some(tracker) = self.selected_tracker() {
            ui.label(format!("Instruments: {}", tracker.instrument_count()));
            ui.label(format!(
                "Pattern data: {} bytes",
                tracker.pattern_data_size()
            ));

            ui.separator();

            let is_playing = self.tracker_playing;

            ui.horizontal(|ui| {
                if is_playing {
                    if ui.button("⏸ Stop").clicked() {
                        self.tracker_playing = false;
                    }
                } else if ui.button("▶ Play").clicked() {
                    self.start_tracker_playback();
                }

                // Show position if playing
                if let Some(ref state) = self.tracker_state {
                    ui.label(format!("Order: {}", state.order_position));
                    ui.label(format!("Row: {}", state.row));
                }
            });
        }
    }

    fn render_skeleton_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Skeleton Preview");
        if let Some(skeleton) = self.selected_skeleton() {
            ui.label(format!("Bones: {}", skeleton.bone_count));
            ui.label(format!(
                "Inverse bind matrices: {}",
                skeleton.inverse_bind_matrices.len()
            ));

            ui.separator();

            // Show bone matrices in a scrollable area
            ui.label("Bone Transforms:");
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    for (i, matrix) in skeleton.inverse_bind_matrices.iter().enumerate() {
                        ui.collapsing(format!("Bone {}", i), |ui| {
                            ui.code(format!(
                                "[ {:.3}, {:.3}, {:.3}, {:.3} ]\n\
                             [ {:.3}, {:.3}, {:.3}, {:.3} ]\n\
                             [ {:.3}, {:.3}, {:.3}, {:.3} ]",
                                matrix.row0[0],
                                matrix.row0[1],
                                matrix.row0[2],
                                matrix.row0[3],
                                matrix.row1[0],
                                matrix.row1[1],
                                matrix.row1[2],
                                matrix.row1[3],
                                matrix.row2[0],
                                matrix.row2[1],
                                matrix.row2[2],
                                matrix.row2[3],
                            ));
                        });
                    }
                });
        } else {
            ui.label("Failed to load skeleton");
        }
    }

    fn render_font_ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, id_owned: &str) {
        ui.heading("Font Preview");

        // Extract font data to avoid borrow issues
        let font_data = self.selected_font().map(|f| {
            (
                f.atlas_width,
                f.atlas_height,
                f.line_height,
                f.baseline,
                f.glyphs.len(),
                f.atlas_data.clone(),
                f.glyphs
                    .iter()
                    .take(20)
                    .map(|g| g.codepoint)
                    .collect::<Vec<_>>(),
            )
        });

        if let Some((
            width,
            height,
            line_height,
            baseline,
            glyph_count,
            atlas_data,
            sample_glyphs,
        )) = font_data
        {
            ui.label(format!("Atlas: {}x{}", width, height));
            ui.label(format!("Glyphs: {}", glyph_count));
            ui.label(format!("Line height: {:.1}", line_height));
            ui.label(format!("Baseline: {:.1}", baseline));

            ui.separator();

            // Extract zoom control value
            let zoom = self.texture_zoom;

            ui.horizontal(|ui| {
                if ui.button("Zoom In").clicked() {
                    self.texture_zoom_in();
                }
                if ui.button("Zoom Out").clicked() {
                    self.texture_zoom_out();
                }
                if ui.button("Reset").clicked() {
                    self.texture_reset_zoom();
                }
                ui.label(format!("Zoom: {:.1}x", zoom));

                ui.separator();

                if ui
                    .button(if self.texture_linear_filter {
                        "Linear"
                    } else {
                        "Nearest"
                    })
                    .clicked()
                {
                    self.texture_linear_filter = !self.texture_linear_filter;
                    // Invalidate cache to recreate texture with new filter
                    self.cached_texture = None;
                    self.cached_texture_id = None;
                }
            });

            ui.separator();

            // Update cache if needed
            let font_id = format!("font_{}", id_owned);
            self.update_texture_cache(
                ctx,
                &font_id,
                &format!("font_atlas_{}", id_owned),
                width,
                height,
                &atlas_data,
            );

            // Use the cached texture
            if let Some(ref texture_handle) = self.cached_texture {
                let display_size = egui::vec2(width as f32 * zoom, height as f32 * zoom);
                ui.add(egui::Image::new(texture_handle).fit_to_exact_size(display_size));
            }

            ui.separator();

            // Show some sample glyphs
            ui.label("Sample Characters:");
            ui.horizontal_wrapped(|ui| {
                for codepoint in &sample_glyphs {
                    if let Some(ch) = char::from_u32(*codepoint) {
                        ui.label(format!("'{}'", ch));
                    }
                }
                if glyph_count > 20 {
                    ui.label("...");
                }
            });
        } else {
            ui.label("Failed to load font");
        }
    }

    fn render_data_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Data Preview");
        if let Some(data) = self.selected_data() {
            ui.label(format!("Size: {} bytes", data.data.len()));

            // Show hex dump of first bytes
            let preview_bytes = data.data.len().min(256);
            ui.label(format!("First {} bytes (hex):", preview_bytes));

            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    ui.code(
                        data.data[..preview_bytes]
                            .chunks(16)
                            .enumerate()
                            .map(|(i, chunk)| {
                                format!(
                                    "{:04x}: {}",
                                    i * 16,
                                    chunk
                                        .iter()
                                        .map(|b| format!("{:02x}", b))
                                        .collect::<Vec<_>>()
                                        .join(" ")
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("\n"),
                    );
                });
        }
    }
}
