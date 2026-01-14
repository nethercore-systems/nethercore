//! Tests for asset viewer

use super::*;
use zx_common::{PackedSound, PackedTexture};

fn create_test_data() -> PreviewData<ZXDataPack> {
    let mut data_pack = ZXDataPack::default();

    // Add a test texture
    data_pack
        .textures
        .push(PackedTexture::new("test_tex", 64, 64, vec![0; 64 * 64 * 4]));

    // Add a test sound (0.5 seconds)
    data_pack
        .sounds
        .push(PackedSound::new("test_sfx", vec![0i16; 11025]));

    PreviewData {
        data_pack,
        metadata: super::super::PreviewMetadata {
            id: "test".to_string(),
            title: "Test Game".to_string(),
            author: "Test".to_string(),
            version: "1.0.0".to_string(),
        },
    }
}

#[test]
fn test_viewer_creation() {
    let data = create_test_data();
    let viewer = ZXAssetViewer::new(&data);

    assert_eq!(viewer.asset_count(AssetCategory::Textures), 1);
    assert_eq!(viewer.asset_count(AssetCategory::Sounds), 1);
    assert_eq!(viewer.asset_count(AssetCategory::Meshes), 0);
}

#[test]
fn test_asset_selection() {
    let data = create_test_data();
    let mut viewer = ZXAssetViewer::new(&data);

    viewer.select_asset(AssetCategory::Textures, "test_tex");

    assert_eq!(viewer.selected_category(), AssetCategory::Textures);
    assert_eq!(viewer.selected_id(), Some("test_tex"));
    assert!(viewer.selected_texture().is_some());
}

#[test]
fn test_texture_controls() {
    let data = create_test_data();
    let mut viewer = ZXAssetViewer::new(&data);

    viewer.texture_zoom_in();
    assert!(viewer.texture_zoom() > 1.0);

    viewer.texture_reset_zoom();
    assert!((viewer.texture_zoom() - 1.0).abs() < f32::EPSILON);

    viewer.texture_pan(10.0, 5.0);
    assert_eq!(viewer.texture_pan_offset(), (10.0, 5.0));
}

#[test]
fn test_sound_controls() {
    let data = create_test_data();
    let mut viewer = ZXAssetViewer::new(&data);

    viewer.select_asset(AssetCategory::Sounds, "test_sfx");

    assert!(!viewer.sound_is_playing());
    viewer.sound_toggle_play();
    assert!(viewer.sound_is_playing());

    viewer.sound_seek(0.5);
    assert!((viewer.sound_progress() - 0.5).abs() < 0.01);

    viewer.sound_stop();
    assert!(!viewer.sound_is_playing());
    assert!((viewer.sound_progress()).abs() < f32::EPSILON);
}
