//! Tests for ZXDataPack and related types

use super::*;
use nethercore_shared::math::BoneMatrix3x4;

#[test]
fn test_empty_data_pack() {
    let pack = ZXDataPack::new();
    assert!(pack.is_empty());
    assert_eq!(pack.asset_count(), 0);
}

#[test]
fn test_data_pack_with_assets() {
    let mut pack = ZXDataPack::new();
    pack.textures
        .push(PackedTexture::new("test", 2, 2, vec![0; 16]));
    pack.meshes.push(PackedMesh {
        id: "mesh".to_string(),
        format: 0,
        vertex_count: 3,
        index_count: 3,
        vertex_data: vec![0; 36],
        index_data: vec![0, 1, 2],
    });

    assert!(!pack.is_empty());
    assert_eq!(pack.asset_count(), 2);
}

#[test]
fn test_find_texture() {
    let mut pack = ZXDataPack::new();
    pack.textures
        .push(PackedTexture::new("player", 32, 32, vec![0; 32 * 32 * 4]));
    pack.textures
        .push(PackedTexture::new("enemy", 16, 16, vec![0; 16 * 16 * 4]));

    assert!(pack.find_texture("player").is_some());
    assert!(pack.find_texture("enemy").is_some());
    assert!(pack.find_texture("missing").is_none());
}

#[test]
fn test_packed_texture_validation() {
    let valid = PackedTexture::new("test", 2, 2, vec![0; 16]);
    assert!(valid.validate());

    let invalid = PackedTexture::new("test", 2, 2, vec![0; 10]); // Wrong size
    assert!(!invalid.validate());
}

#[test]
fn test_mesh_format_flags() {
    let mesh = PackedMesh {
        id: "test".to_string(),
        format: 0b1111, // All flags
        vertex_count: 1,
        index_count: 0,
        vertex_data: vec![],
        index_data: vec![],
    };

    assert!(mesh.has_uv());
    assert!(mesh.has_color());
    assert!(mesh.has_normal());
    assert!(mesh.is_skinned());
}

#[test]
fn test_mesh_stride() {
    // Position only (f16x4 = 8 bytes)
    let pos_only = PackedMesh {
        id: "test".to_string(),
        format: 0,
        vertex_count: 1,
        index_count: 0,
        vertex_data: vec![],
        index_data: vec![],
    };
    assert_eq!(pos_only.stride(), 8);

    // Position + UV + Normal (f16x4 + unorm16x2 + octahedral = 8 + 4 + 4 = 16)
    let pos_uv_norm = PackedMesh {
        id: "test".to_string(),
        format: 0b0101, // UV + Normal
        vertex_count: 1,
        index_count: 0,
        vertex_data: vec![],
        index_data: vec![],
    };
    assert_eq!(pos_uv_norm.stride(), 16);

    // Full skinned (f16x4 + unorm16x2 + unorm8x4 + octahedral + u8x4 + unorm8x4 = 8+4+4+4+4+4 = 28)
    let skinned = PackedMesh {
        id: "test".to_string(),
        format: 0b1111, // All flags
        vertex_count: 1,
        index_count: 0,
        vertex_data: vec![],
        index_data: vec![],
    };
    assert_eq!(skinned.stride(), 28);
}

#[test]
fn test_packed_sound_duration() {
    let sound = PackedSound::new("test", vec![0; 22050]); // 1 second
    assert!((sound.duration_seconds() - 1.0).abs() < 0.001);
}

#[test]
fn test_packed_skeleton() {
    let skeleton = PackedSkeleton::new(
        "test",
        vec![BoneMatrix3x4::IDENTITY, BoneMatrix3x4::IDENTITY],
    );
    assert_eq!(skeleton.bone_count, 2);
    assert!(skeleton.validate());
}

#[test]
fn test_find_mesh() {
    let mut pack = ZXDataPack::new();
    pack.meshes.push(PackedMesh {
        id: "cube".to_string(),
        format: 0b0001, // UV only
        vertex_count: 24,
        index_count: 36,
        vertex_data: vec![0; 24 * 20], // pos + uv
        index_data: vec![0; 36],
    });
    pack.meshes.push(PackedMesh {
        id: "sphere".to_string(),
        format: 0,
        vertex_count: 100,
        index_count: 200,
        vertex_data: vec![0; 100 * 12],
        index_data: vec![0; 200],
    });

    let cube = pack.find_mesh("cube");
    assert!(cube.is_some());
    assert_eq!(cube.unwrap().vertex_count, 24);

    let sphere = pack.find_mesh("sphere");
    assert!(sphere.is_some());
    assert_eq!(sphere.unwrap().vertex_count, 100);

    assert!(pack.find_mesh("missing").is_none());
}

#[test]
fn test_find_skeleton() {
    let mut pack = ZXDataPack::new();
    pack.skeletons.push(PackedSkeleton::new(
        "player_rig",
        vec![BoneMatrix3x4::IDENTITY; 20],
    ));
    pack.skeletons.push(PackedSkeleton::new(
        "enemy_rig",
        vec![BoneMatrix3x4::IDENTITY; 10],
    ));

    let player = pack.find_skeleton("player_rig");
    assert!(player.is_some());
    assert_eq!(player.unwrap().bone_count, 20);

    let enemy = pack.find_skeleton("enemy_rig");
    assert!(enemy.is_some());
    assert_eq!(enemy.unwrap().bone_count, 10);

    assert!(pack.find_skeleton("missing").is_none());
}

#[test]
fn test_find_font() {
    let mut pack = ZXDataPack::new();
    pack.fonts.push(PackedFont {
        id: "pixel".to_string(),
        atlas_width: 256,
        atlas_height: 256,
        atlas_data: vec![0; 256 * 256 * 4],
        glyphs: vec![
            PackedGlyph {
                codepoint: 'A' as u32,
                x: 0,
                y: 0,
                w: 8,
                h: 8,
                x_offset: 0.0,
                y_offset: 0.0,
                advance: 8.0,
            },
            PackedGlyph {
                codepoint: 'B' as u32,
                x: 8,
                y: 0,
                w: 8,
                h: 8,
                x_offset: 0.0,
                y_offset: 0.0,
                advance: 8.0,
            },
        ],
        line_height: 10.0,
        baseline: 8.0,
    });

    let font = pack.find_font("pixel");
    assert!(font.is_some());
    assert_eq!(font.unwrap().glyphs.len(), 2);
    assert!((font.unwrap().line_height - 10.0).abs() < 0.001);

    assert!(pack.find_font("missing").is_none());
}

#[test]
fn test_find_sound() {
    let mut pack = ZXDataPack::new();
    pack.sounds.push(PackedSound::new("jump", vec![0i16; 2205])); // 0.1 sec
    pack.sounds
        .push(PackedSound::new("explosion", vec![0i16; 22050])); // 1 sec

    let jump = pack.find_sound("jump");
    assert!(jump.is_some());
    assert_eq!(jump.unwrap().data.len(), 2205);

    let explosion = pack.find_sound("explosion");
    assert!(explosion.is_some());
    assert!((explosion.unwrap().duration_seconds() - 1.0).abs() < 0.001);

    assert!(pack.find_sound("missing").is_none());
}

#[test]
fn test_find_data() {
    let mut pack = ZXDataPack::new();
    pack.data
        .push(PackedData::new("level1", vec![1, 2, 3, 4, 5]));
    pack.data.push(PackedData::new("config", vec![0xFF; 100]));

    let level = pack.find_data("level1");
    assert!(level.is_some());
    assert_eq!(level.unwrap().data, vec![1, 2, 3, 4, 5]);

    let config = pack.find_data("config");
    assert!(config.is_some());
    assert_eq!(config.unwrap().data.len(), 100);

    assert!(pack.find_data("missing").is_none());
}

#[test]
fn test_find_keyframes() {
    let mut pack = ZXDataPack::new();

    // Create 2 bones, 3 frames animation (2 * 3 * 16 = 96 bytes)
    let walk_data = vec![0u8; 2 * 3 * 16];
    pack.keyframes
        .push(PackedKeyframes::new("walk", 2, 3, walk_data));

    // Create 4 bones, 10 frames animation (4 * 10 * 16 = 640 bytes)
    let run_data = vec![0u8; 4 * 10 * 16];
    pack.keyframes
        .push(PackedKeyframes::new("run", 4, 10, run_data));

    let walk = pack.find_keyframes("walk");
    assert!(walk.is_some());
    assert_eq!(walk.unwrap().bone_count, 2);
    assert_eq!(walk.unwrap().frame_count, 3);
    assert!(walk.unwrap().validate());

    let run = pack.find_keyframes("run");
    assert!(run.is_some());
    assert_eq!(run.unwrap().bone_count, 4);
    assert_eq!(run.unwrap().frame_count, 10);
    assert!(run.unwrap().validate());

    assert!(pack.find_keyframes("missing").is_none());
}

#[test]
fn test_packed_keyframes_frame_data() {
    // 2 bones, 2 frames (2 * 2 * 16 = 64 bytes)
    let mut data = vec![0u8; 64];
    // Mark first frame's first bone
    data[0] = 0xFF;
    // Mark second frame's first bone
    data[32] = 0xAA;

    let kf = PackedKeyframes::new("test", 2, 2, data);
    assert!(kf.validate());

    let frame0 = kf.frame_data(0).unwrap();
    assert_eq!(frame0.len(), 32); // 2 bones * 16 bytes
    assert_eq!(frame0[0], 0xFF);

    let frame1 = kf.frame_data(1).unwrap();
    assert_eq!(frame1.len(), 32);
    assert_eq!(frame1[0], 0xAA);

    // Out of bounds
    assert!(kf.frame_data(2).is_none());
}

#[test]
fn test_serialization_roundtrip() {
    let mut pack = ZXDataPack::new();

    // Add one of each asset type
    pack.textures
        .push(PackedTexture::new("tex", 4, 4, vec![0xAB; 64]));
    pack.meshes.push(PackedMesh {
        id: "mesh".to_string(),
        format: 0b0101,
        vertex_count: 3,
        index_count: 3,
        vertex_data: vec![1, 2, 3],
        index_data: vec![0, 1, 2],
    });
    pack.skeletons
        .push(PackedSkeleton::new("skel", vec![BoneMatrix3x4::IDENTITY]));
    pack.keyframes
        .push(PackedKeyframes::new("anim", 2, 5, vec![0; 2 * 5 * 16]));
    pack.fonts.push(PackedFont {
        id: "font".to_string(),
        atlas_width: 64,
        atlas_height: 64,
        atlas_data: vec![0; 64 * 64 * 4],
        glyphs: vec![PackedGlyph {
            codepoint: 'X' as u32,
            x: 0,
            y: 0,
            w: 8,
            h: 8,
            x_offset: 0.0,
            y_offset: 0.0,
            advance: 8.0,
        }],
        line_height: 12.0,
        baseline: 10.0,
    });
    pack.sounds.push(PackedSound::new("sfx", vec![100i16; 100]));
    pack.data.push(PackedData::new("raw", vec![9, 8, 7]));

    // Serialize with bitcode
    let encoded = bitcode::encode(&pack);

    // Deserialize
    let decoded: ZXDataPack = bitcode::decode(&encoded).expect("decode failed");

    // Verify all assets survived
    assert_eq!(decoded.asset_count(), 7);
    assert_eq!(decoded.textures.len(), 1);
    assert_eq!(decoded.meshes.len(), 1);
    assert_eq!(decoded.skeletons.len(), 1);
    assert_eq!(decoded.keyframes.len(), 1);
    assert_eq!(decoded.fonts.len(), 1);
    assert_eq!(decoded.sounds.len(), 1);
    assert_eq!(decoded.data.len(), 1);

    // Verify content
    assert_eq!(decoded.find_texture("tex").unwrap().width, 4);
    assert_eq!(decoded.find_mesh("mesh").unwrap().format, 0b0101);
    assert_eq!(decoded.find_skeleton("skel").unwrap().bone_count, 1);
    assert_eq!(decoded.find_keyframes("anim").unwrap().bone_count, 2);
    assert_eq!(decoded.find_keyframes("anim").unwrap().frame_count, 5);
    assert!((decoded.find_font("font").unwrap().line_height - 12.0).abs() < 0.001);
    assert_eq!(decoded.find_sound("sfx").unwrap().data.len(), 100);
    assert_eq!(decoded.find_data("raw").unwrap().data, vec![9, 8, 7]);
}

// ========================================================================
// TextureFormat tests
// ========================================================================

#[test]
fn test_texture_format_default() {
    let tex = PackedTexture::new("test", 64, 64, vec![0; 64 * 64 * 4]);
    assert_eq!(tex.format, TextureFormat::Rgba8);
}

#[test]
fn test_texture_format_equality() {
    assert_eq!(TextureFormat::Rgba8, TextureFormat::Rgba8);
    assert_eq!(TextureFormat::Bc7, TextureFormat::Bc7);

    assert_ne!(TextureFormat::Rgba8, TextureFormat::Bc7);
}

#[test]
fn test_texture_format_is_bc7() {
    assert!(!TextureFormat::Rgba8.is_bc7());
    assert!(TextureFormat::Bc7.is_bc7());
}

#[test]
fn test_texture_format_data_size_rgba8() {
    assert_eq!(TextureFormat::Rgba8.data_size(64, 64), 64 * 64 * 4);
    assert_eq!(TextureFormat::Rgba8.data_size(32, 32), 32 * 32 * 4);
    assert_eq!(TextureFormat::Rgba8.data_size(128, 64), 128 * 64 * 4);
}

#[test]
fn test_texture_format_data_size_bc7() {
    // 64×64 = 16×16 blocks × 16 bytes = 4096 bytes
    assert_eq!(TextureFormat::Bc7.data_size(64, 64), 4096);

    // 32×32 = 8×8 blocks × 16 bytes = 1024 bytes
    assert_eq!(TextureFormat::Bc7.data_size(32, 32), 1024);

    // 128×128 = 32×32 blocks × 16 bytes = 16384 bytes
    assert_eq!(TextureFormat::Bc7.data_size(128, 128), 16384);
}

#[test]
fn test_texture_format_data_size_bc7_non_aligned() {
    // 30×30 → 8×8 blocks (rounds up) × 16 bytes = 1024 bytes
    assert_eq!(TextureFormat::Bc7.data_size(30, 30), 8 * 8 * 16);

    // 1×1 → 1×1 blocks × 16 bytes = 16 bytes
    assert_eq!(TextureFormat::Bc7.data_size(1, 1), 16);

    // 5×7 → 2×2 blocks × 16 bytes = 64 bytes
    assert_eq!(TextureFormat::Bc7.data_size(5, 7), 2 * 2 * 16);
}

#[test]
fn test_bc7_compression_ratio() {
    let w: u16 = 64;
    let h: u16 = 64;
    let rgba8 = TextureFormat::Rgba8.data_size(w, h);
    let bc7 = TextureFormat::Bc7.data_size(w, h);
    assert_eq!(rgba8 / bc7, 4); // 4× compression ratio
}

#[test]
fn test_packed_texture_with_format() {
    let tex = PackedTexture::with_format(
        "material",
        64,
        64,
        TextureFormat::Bc7,
        vec![0; 4096], // BC7 size for 64×64
    );

    assert_eq!(tex.width, 64);
    assert_eq!(tex.height, 64);
    assert_eq!(tex.format, TextureFormat::Bc7);
    assert!(tex.is_bc7());
    assert!(tex.validate());
}

#[test]
fn test_packed_texture_dimensions_u32() {
    let tex = PackedTexture::new("test", 256, 128, vec![0; 256 * 128 * 4]);
    let (w, h) = tex.dimensions_u32();
    assert_eq!(w, 256);
    assert_eq!(h, 128);
}

#[test]
fn test_packed_data() {
    let data = PackedData::new("level", vec![0x01, 0x02, 0x03, 0x04]);
    assert_eq!(data.id, "level");
    assert_eq!(data.data.len(), 4);
}

#[test]
fn test_packed_glyph() {
    let glyph = PackedGlyph {
        codepoint: 'A' as u32,
        x: 10,
        y: 20,
        w: 8,
        h: 16,
        x_offset: 1.0,
        y_offset: 2.0,
        advance: 10.0,
    };
    assert_eq!(glyph.codepoint, 65);
    assert!((glyph.advance - 10.0).abs() < 0.001);
    assert_eq!(glyph.w, 8);
    assert_eq!(glyph.h, 16);
}

#[test]
fn test_find_tracker() {
    let mut pack = ZXDataPack::new();
    pack.trackers.push(PackedTracker::new(
        "level1_music",
        TrackerFormat::Xm,
        vec![0x45, 0x78, 0x74], // Dummy XM data
        vec!["kick".to_string(), "snare".to_string()],
    ));
    pack.trackers.push(PackedTracker::new(
        "boss_theme",
        TrackerFormat::Xm,
        vec![0x01, 0x02, 0x03],
        vec!["bass".to_string()],
    ));

    let level = pack.find_tracker("level1_music");
    assert!(level.is_some());
    assert_eq!(level.unwrap().instrument_count(), 2);
    assert_eq!(level.unwrap().sample_ids[0], "kick");

    let boss = pack.find_tracker("boss_theme");
    assert!(boss.is_some());
    assert_eq!(boss.unwrap().instrument_count(), 1);

    assert!(pack.find_tracker("missing").is_none());
}

#[test]
fn test_packed_tracker() {
    let tracker = PackedTracker::new(
        "test_song",
        TrackerFormat::Xm,
        vec![0; 1024],
        vec!["drum".to_string(), "bass".to_string(), "lead".to_string()],
    );

    assert_eq!(tracker.id, "test_song");
    assert_eq!(tracker.instrument_count(), 3);
    assert_eq!(tracker.pattern_data_size(), 1024);
}
