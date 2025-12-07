//! Debug overlay rendering utilities
//!
//! Provides debug overlay UI that can be used by any console to display
//! performance metrics, memory usage, and network statistics.

use super::types::DebugStats;
use std::collections::VecDeque;

/// Frame time sample for graph
pub const FRAME_TIME_HISTORY_SIZE: usize = 120;

/// Target frame time for reference line (60 FPS = 16.67ms)
pub const TARGET_FRAME_TIME_MS: f32 = 16.67;

/// Maximum frame time shown in graph (30 FPS = 33.33ms, 2x target)
pub const GRAPH_MAX_FRAME_TIME_MS: f32 = 33.33;

/// Render the debug overlay window with performance and network stats
///
/// # Arguments
///
/// * `ctx` - egui context for rendering
/// * `stats` - Debug statistics to display
/// * `mode` - Current application mode (to show different graphs)
/// * `frame_time_ms` - Current frame time in milliseconds
/// * `render_fps` - Render loop FPS
/// * `game_tick_fps` - Game update loop FPS (when playing)
pub fn render_debug_overlay(
    ctx: &egui::Context,
    stats: &DebugStats,
    is_playing: bool,
    frame_time_ms: f32,
    render_fps: f32,
    game_tick_fps: f32,
) {
    egui::Window::new("Debug")
        .default_pos([10.0, 10.0])
        .resizable(true)
        .default_width(300.0)
        .show(ctx, |ui| {
            // Performance section
            ui.heading("Performance");
            if is_playing {
                ui.label(format!("Game FPS: {:.1}", game_tick_fps));
                ui.label(format!("Render FPS: {:.1}", render_fps));
            } else {
                ui.label(format!("FPS: {:.1}", render_fps));
            }
            ui.label(format!("Frame time: {:.2}ms", frame_time_ms));

            // Frame time graph
            ui.add_space(4.0);
            let graph_height = 60.0;
            let (rect, _response) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), graph_height),
                egui::Sense::hover(),
            );

            if ui.is_rect_visible(rect) {
                let painter = ui.painter_at(rect);

                // Background
                painter.rect_filled(rect, 2.0, egui::Color32::from_gray(30));

                // Choose which times to display based on mode
                let (times_to_display, graph_label, graph_max) = if is_playing {
                    // Game tick budget visualization - full height = 16.67ms budget
                    let label = format!("Game tick budget ({:.1}ms target)", TARGET_FRAME_TIME_MS);
                    (&stats.game_tick_times, label, TARGET_FRAME_TIME_MS)
                } else {
                    (
                        &stats.frame_times,
                        "Frame time (0-33ms)".to_string(),
                        GRAPH_MAX_FRAME_TIME_MS,
                    )
                };

                // Target line (16.67ms for 60 FPS)
                let target_y = rect.bottom() - (TARGET_FRAME_TIME_MS / graph_max * graph_height);
                painter.hline(
                    rect.left()..=rect.right(),
                    target_y,
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)),
                );

                // Budget bars (for game tick times in Playing mode)
                if !times_to_display.is_empty() {
                    let bar_width = rect.width() / FRAME_TIME_HISTORY_SIZE as f32;
                    for (i, &time_ms) in times_to_display.iter().enumerate() {
                        let x = rect.left() + i as f32 * bar_width;

                        if is_playing {
                            // Stacked budget bar: update (blue) + render (orange) + available (green)

                            // Get update and render times for this tick
                            let update_time = time_ms; // game_tick_times[i]
                            let render_time =
                                stats.game_render_times.get(i).copied().unwrap_or(0.0);
                            let total_time = update_time + render_time;

                            // Calculate heights (scaled to budget, capped at 150%)
                            let update_height =
                                ((update_time / TARGET_FRAME_TIME_MS).min(1.5) * graph_height);
                            let render_height =
                                ((render_time / TARGET_FRAME_TIME_MS).min(1.5) * graph_height);
                            let total_height = (update_height + render_height).min(graph_height);

                            let bottom_y = rect.bottom();

                            // Draw background (unused budget) - green if under budget, red if over
                            let bg_color = if total_time <= TARGET_FRAME_TIME_MS {
                                egui::Color32::from_rgb(40, 80, 40) // Dark green - headroom available
                            } else {
                                egui::Color32::from_rgb(80, 40, 40) // Dark red - over budget
                            };
                            painter.rect_filled(
                                egui::Rect::from_min_max(
                                    egui::pos2(x, rect.top()),
                                    egui::pos2(x + bar_width - 1.0, bottom_y),
                                ),
                                0.0,
                                bg_color,
                            );

                            // Draw update time (bottom, blue)
                            if update_height > 0.0 {
                                painter.rect_filled(
                                    egui::Rect::from_min_max(
                                        egui::pos2(x, bottom_y - update_height),
                                        egui::pos2(x + bar_width - 1.0, bottom_y),
                                    ),
                                    0.0,
                                    egui::Color32::from_rgb(80, 120, 200), // Blue - update time
                                );
                            }

                            // Draw render time (stacked on top of update, orange)
                            if render_height > 0.0 {
                                painter.rect_filled(
                                    egui::Rect::from_min_max(
                                        egui::pos2(x, bottom_y - total_height),
                                        egui::pos2(x + bar_width - 1.0, bottom_y - update_height),
                                    ),
                                    0.0,
                                    egui::Color32::from_rgb(220, 140, 60), // Orange - render time
                                );
                            }
                        } else {
                            // Standard bars for render times in Library mode
                            let height = (time_ms / graph_max * graph_height).min(graph_height);
                            let bar_rect = egui::Rect::from_min_max(
                                egui::pos2(x, rect.bottom() - height),
                                egui::pos2(x + bar_width - 1.0, rect.bottom()),
                            );

                            let color = if time_ms <= TARGET_FRAME_TIME_MS {
                                egui::Color32::from_rgb(100, 200, 100)
                            } else {
                                egui::Color32::from_rgb(200, 200, 100)
                            };

                            painter.rect_filled(bar_rect, 0.0, color);
                        }
                    }
                }

                // Label
                painter.text(
                    egui::pos2(rect.left() + 4.0, rect.top() + 2.0),
                    egui::Align2::LEFT_TOP,
                    graph_label,
                    egui::FontId::proportional(10.0),
                    egui::Color32::from_gray(150),
                );
            }

            ui.separator();

            // Memory section
            ui.heading("Memory");
            let vram_mb = stats.vram_used as f32 / (1024.0 * 1024.0);
            let vram_limit_mb = stats.vram_limit as f32 / (1024.0 * 1024.0);
            let vram_pct = stats.vram_used as f32 / stats.vram_limit as f32;
            ui.label(format!(
                "VRAM: {:.2} / {:.2} MB ({:.1}%)",
                vram_mb,
                vram_limit_mb,
                vram_pct * 100.0
            ));
            ui.add(egui::ProgressBar::new(vram_pct).show_percentage());

            ui.separator();

            // Network section
            ui.heading("Network");
            if let Some(ping) = stats.ping_ms {
                ui.label(format!("Ping: {}ms", ping));
                ui.label(format!("Rollback frames: {}", stats.rollback_frames));
                ui.label(format!("Frame advantage: {}", stats.frame_advantage));

                // Network interrupted warning
                if let Some(timeout_ms) = stats.network_interrupted {
                    ui.add_space(4.0);
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 200, 50),
                        format!("⚠ Connection interrupted ({}ms)", timeout_ms),
                    );
                }
            } else {
                ui.label("No network session");
            }
        });
}

/// Calculate FPS from frame time samples
pub fn calculate_fps(frame_times: &[std::time::Instant]) -> f32 {
    if frame_times.len() < 2 {
        return 0.0;
    }
    let elapsed = frame_times
        .last()
        .unwrap()
        .duration_since(*frame_times.first().unwrap())
        .as_secs_f32();
    if elapsed > 0.0 {
        frame_times.len() as f32 / elapsed
    } else {
        0.0
    }
}

/// Update frame time ring buffer
pub fn update_frame_times(frame_times: &mut VecDeque<f32>, new_time_ms: f32) {
    frame_times.push_back(new_time_ms);
    while frame_times.len() > FRAME_TIME_HISTORY_SIZE {
        frame_times.pop_front();
    }
}
