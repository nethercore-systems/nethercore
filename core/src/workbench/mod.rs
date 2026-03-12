//! Machine-drivable EPU workbench types.
//!
//! These types describe the stable JSON contract shared between the local
//! workbench server, the standalone player, and thin external clients.

use serde::{Deserialize, Serialize};

/// Stable workbench protocol version.
pub const EPU_WORKBENCH_PROTOCOL_VERSION: u32 = 1;

/// Serializable blend mode.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EpuWorkbenchBlend {
    Add,
    Multiply,
    Max,
    Lerp,
    Screen,
    HsvMod,
    Min,
    Overlay,
}

/// Serializable layer state.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EpuWorkbenchLayer {
    pub opcode: u8,
    pub variant_id: u8,
    pub domain_id: u8,
    pub region_mask: u8,
    pub blend: EpuWorkbenchBlend,
    pub color_a: [u8; 3],
    pub color_b: [u8; 3],
    pub alpha_a: u8,
    pub alpha_b: u8,
    pub intensity: u8,
    pub param_a: u8,
    pub param_b: u8,
    pub param_c: u8,
    pub param_d: u8,
    pub direction: u16,
}

/// Serializable full 8-layer config.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EpuWorkbenchConfig {
    pub layers: [EpuWorkbenchLayer; 8],
}

/// Partial update for a single layer.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EpuWorkbenchLayerPatch {
    pub opcode: Option<u8>,
    pub variant_id: Option<u8>,
    pub domain_id: Option<u8>,
    pub region_mask: Option<u8>,
    pub blend: Option<EpuWorkbenchBlend>,
    pub color_a: Option<[u8; 3]>,
    pub color_b: Option<[u8; 3]>,
    pub alpha_a: Option<u8>,
    pub alpha_b: Option<u8>,
    pub intensity: Option<u8>,
    pub param_a: Option<u8>,
    pub param_b: Option<u8>,
    pub param_c: Option<u8>,
    pub param_d: Option<u8>,
    pub direction: Option<u16>,
}

/// Editor/workbench view state that affects live rendering.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EpuWorkbenchViewState {
    pub selected_layer: Option<usize>,
    pub isolated_layer: Option<usize>,
    pub clear_layer_isolation: Option<bool>,
    pub locked: Option<bool>,
    pub show_benchmarks: Option<bool>,
    pub scene_index: Option<i32>,
    pub show_ui: Option<bool>,
    pub show_probe: Option<bool>,
    pub show_background: Option<bool>,
}

/// Scene selection in the showcase ROM.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EpuWorkbenchSceneMode {
    Showcase,
    Benchmark,
}

/// Scene selection request.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EpuWorkbenchSceneSelection {
    pub mode: EpuWorkbenchSceneMode,
    pub scene_index: i32,
    #[serde(default = "default_true")]
    pub load_into_editor: bool,
    #[serde(default = "default_true")]
    pub lock_editor: bool,
}

/// Crop rectangle in pixels.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EpuWorkbenchCrop {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Named capture outputs for frame review.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EpuWorkbenchCaptureResult {
    pub frame_path: String,
    pub background_crop_path: String,
    pub probe_crop_path: String,
    pub width: u32,
    pub height: u32,
    pub background_crop: EpuWorkbenchCrop,
    pub probe_crop: EpuWorkbenchCrop,
}

/// Export output payload.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EpuWorkbenchExportResult {
    pub json_path: Option<String>,
    pub rust_path: Option<String>,
    pub json_text: Option<String>,
    pub rust_text: Option<String>,
}

/// Console-specific metadata surfaced to the client.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EpuWorkbenchMetadata {
    pub opcode_names: Vec<String>,
}

/// Snapshot returned by the server for the current session.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EpuWorkbenchSnapshot {
    pub protocol_version: u32,
    pub game_id: String,
    pub scene_mode: Option<EpuWorkbenchSceneMode>,
    pub scene_index: Option<i32>,
    pub show_ui: Option<bool>,
    pub show_probe: Option<bool>,
    pub show_background: Option<bool>,
    pub config: EpuWorkbenchConfig,
    pub view: EpuWorkbenchViewState,
    pub metadata: EpuWorkbenchMetadata,
}

/// Default crop pair used for capture separation.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EpuWorkbenchCaptureCrops {
    pub background: EpuWorkbenchCrop,
    pub probe: EpuWorkbenchCrop,
}

/// Export request options.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EpuWorkbenchExportOptions {
    pub label: Option<String>,
    pub rust_const_name: Option<String>,
    pub include_json_text: bool,
    pub include_rust_text: bool,
}

fn default_true() -> bool {
    true
}
