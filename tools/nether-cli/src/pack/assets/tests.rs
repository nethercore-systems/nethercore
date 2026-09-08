//! Tests for asset loading functionality.

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::super::load_assets;
    use crate::manifest::{AssetsSection, NetherManifest};
    use crate::pack::assets::{
        animation::load_keyframes,
        audio::load_sound,
        data::load_data,
        environment::load_epu_environment,
        mesh::load_mesh,
        texture::load_texture,
        utils::{hash_sample_data, sanitize_name},
    };
    use tempfile::tempdir;
    use zx_common::{
        vertex_stride_packed, NetherZXAnimationHeader, NetherZXMeshHeader, TextureFormat,
        FORMAT_COLOR, FORMAT_UV,
    };

    #[test]
    fn xm_extracted_ids_preserve_blank_names_collisions_and_pcm_aliases() {
        let dir = tempdir().unwrap();
        // Small ordinary XM: four single-sample instruments, authored PCM deltas.
        let mut xm = vec![0u8; 336];
        xm[..17].copy_from_slice(b"Extended Module: ");
        xm[37] = 0x1a;
        xm[58..60].copy_from_slice(&0x0104u16.to_le_bytes());
        xm[60..64].copy_from_slice(&276u32.to_le_bytes());
        for (offset, value) in [(64, 1u16), (68, 1), (72, 4), (74, 1), (76, 6), (78, 125)] {
            xm[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
        }
        for (name, pcm) in [("", 32u8), ("Perc !", 64), ("Perc !", 192), ("Alias", 32)] {
            let mut instrument = vec![0u8; 243];
            instrument[..4].copy_from_slice(&243u32.to_le_bytes());
            instrument[4..4 + name.len()].copy_from_slice(name.as_bytes());
            instrument[27..29].copy_from_slice(&1u16.to_le_bytes());
            instrument[29..33].copy_from_slice(&40u32.to_le_bytes());
            xm.extend(instrument);
            let mut sample = vec![0u8; 40];
            sample[..4].copy_from_slice(&16u32.to_le_bytes());
            sample[12] = 64;
            sample[15] = 128;
            xm.extend(sample);
            xm.push(pcm);
            xm.extend([0u8; 15]);
        }
        std::fs::write(dir.path().join("mapping.xm"), xm).unwrap();
        let manifest = NetherManifest::parse(
            r#"
[game]
id="xm-mapping"
title="Mapping"
author="Test"
version="0.1.0"
[[assets.trackers]]
id="music"
path="mapping.xm"
"#,
        )
        .unwrap();
        let pack = load_assets(dir.path(), &manifest.assets, TextureFormat::Rgba8).unwrap();
        let ids = &pack.trackers[0].sample_ids;
        assert_eq!(ids.len(), 4);
        assert!(ids.iter().all(|id| !id.is_empty()));
        assert_ne!(ids[1], ids[2]);
        assert_eq!(ids[0], ids[3]);
        for (index, positive) in [(0, true), (1, true), (2, false), (3, true)] {
            let data = &pack
                .sounds
                .iter()
                .find(|s| s.id == ids[index])
                .unwrap()
                .data;
            assert!(!data.is_empty());
            assert!(data.iter().all(|&s| if positive { s > 0 } else { s < 0 }));
        }
    }

    #[test]
    fn empty_xm_sample_payloads_keep_silent_slots_and_explicit_overrides() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("empty.xm");
        for name in ["", "Empty"] {
            for count in [0u16, 1, 2] {
                let mut xm = vec![0u8; 336];
                xm[..17].copy_from_slice(b"Extended Module: ");
                xm[37] = 0x1a;
                xm[58..60].copy_from_slice(&0x0104u16.to_le_bytes());
                xm[60..64].copy_from_slice(&276u32.to_le_bytes());
                for (offset, value) in [(64, 1u16), (68, 1), (72, 1), (76, 6), (78, 125)] {
                    xm[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
                }
                let mut instrument = vec![0u8; 243];
                instrument[..4].copy_from_slice(&243u32.to_le_bytes());
                instrument[4..4 + name.len()].copy_from_slice(name.as_bytes());
                instrument[27..29].copy_from_slice(&count.to_le_bytes());
                instrument[29..33].copy_from_slice(&40u32.to_le_bytes());
                xm.extend(instrument);
                xm.extend(vec![0u8; 40 * count as usize]);
                std::fs::write(&path, &xm).unwrap();
                let manifest = NetherManifest::parse(
                    r#"
[game]
id="empty-xm"
title="Empty"
author="Test"
version="0.1.0"
[[assets.trackers]]
id="music"
path="empty.xm"
"#,
                )
                .unwrap();
                let pack = load_assets(dir.path(), &manifest.assets, TextureFormat::Rgba8).unwrap();
                assert_eq!(
                    pack.trackers[0].sample_ids,
                    vec![String::new(); usize::from(count.max(1))],
                    "name={name}, count={count}"
                );
                assert!(pack.sounds.is_empty());
                if !name.is_empty() {
                    let supplied = std::collections::HashSet::from([name.to_string()]);
                    let tracker =
                        super::super::audio::load_tracker("music", &path, &supplied, None).unwrap();
                    assert_eq!(tracker.sample_ids, vec![name; usize::from(count.max(1))]);
                }
                if count == 2 {
                    let mut embedded = xm.clone();
                    embedded[579..583].copy_from_slice(&1u32.to_le_bytes());
                    embedded.push(1);
                    std::fs::write(&path, embedded).unwrap();
                    let error = super::super::audio::load_tracker(
                        "music",
                        &path,
                        &Default::default(),
                        None,
                    )
                    .unwrap_err();
                    assert!(error.to_string().contains("Missing XM sample"));
                }
                if count > 0 {
                    xm.pop();
                    std::fs::write(&path, &xm).unwrap();
                    assert!(super::super::audio::load_tracker(
                        "music",
                        &path,
                        &Default::default(),
                        None
                    )
                    .is_err());
                }
            }
        }
    }

    #[test]
    fn named_empty_it_sample_is_silent_unless_explicitly_provided() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("empty.it");
        let mut writer = nether_it::ItWriter::new("empty");
        writer.add_sample(
            nether_it::ItSample {
                name: "Untitled".into(),
                ..Default::default()
            },
            &[],
        );
        std::fs::write(&path, writer.write()).unwrap();
        let empty = std::collections::HashSet::new();
        let tracker = super::super::audio::load_tracker("music", &path, &empty, None).unwrap();
        assert_eq!(tracker.sample_ids, vec![String::new()]);
        let supplied = std::collections::HashSet::from(["Untitled".to_string()]);
        let tracker = super::super::audio::load_tracker("music", &path, &supplied, None).unwrap();
        assert_eq!(tracker.sample_ids, vec!["Untitled"]);
    }

    #[test]
    fn it_duplicate_sample_names_and_content_keep_index_mapping() {
        use nether_it::{ItSample, ItWriter};
        let dir = tempdir().unwrap();
        let mut writer = ItWriter::new("Duplicate names");
        for (name, pcm) in [
            ("Square", vec![1000i16; 16]),
            ("Square", vec![-1000i16; 16]),
            ("Alias", vec![1000i16; 16]),
            ("...", vec![500i16; 16]),
        ] {
            writer.add_sample(
                ItSample {
                    name: name.into(),
                    c5_speed: 22050,
                    ..Default::default()
                },
                &pcm,
            );
        }
        writer.add_pattern(1);
        writer.set_orders(&[0, 255]);
        std::fs::write(dir.path().join("duplicates.it"), writer.write()).unwrap();
        let mut wav = b"RIFF".to_vec();
        wav.extend_from_slice(&(36u32 + 32).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&22050u32.to_le_bytes());
        wav.extend_from_slice(&44100u32.to_le_bytes());
        wav.extend_from_slice(&2u16.to_le_bytes());
        wav.extend_from_slice(&16u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&32u32.to_le_bytes());
        for _ in 0..16 {
            wav.extend_from_slice(&2000i16.to_le_bytes());
        }
        std::fs::write(dir.path().join("explicit.wav"), wav).unwrap();
        let manifest = NetherManifest::parse(
            r#"
[game]
id="mapping-test"
title="Mapping"
author="Test"
version="0.1.0"
[[assets.trackers]]
id="music"
path="duplicates.it"
[[assets.sounds]]
id="square"
path="explicit.wav"
[[assets.sounds]]
id="music_sample0"
path="explicit.wav"
"#,
        )
        .unwrap();
        let pack = load_assets(dir.path(), &manifest.assets, TextureFormat::Rgba8).unwrap();
        let ids = &pack.trackers[0].sample_ids;
        assert_eq!(ids.len(), 4);
        assert!(ids.iter().all(|id| !id.is_empty()));
        assert_ne!(ids[0], ids[1]);
        assert_eq!(ids[0], "music_sample0_");
        assert!(pack
            .sounds
            .iter()
            .find(|s| s.id == "square")
            .unwrap()
            .data
            .iter()
            .all(|s| *s == 2000));
        let ids_in_order: Vec<_> = pack.sounds.iter().map(|s| s.id.clone()).collect();
        let mut sorted = ids_in_order.clone();
        sorted.sort();
        assert_eq!(ids_in_order, sorted);
        assert_eq!(ids[0], ids[2]);
        for (index, value) in [(0, 1000i16), (1, -1000), (2, 1000), (3, 500)] {
            let sound = pack.sounds.iter().find(|s| s.id == ids[index]).unwrap();
            assert!(sound.data.iter().all(|sample| *sample == value));
        }
        // A matching explicit name must not hide a corrupt embedded payload.
        let mut bad = ItWriter::new("Truncated PCM");
        bad.add_sample(
            ItSample {
                name: "square".into(),
                ..Default::default()
            },
            &[1234i16; 16],
        );
        bad.add_pattern(1);
        bad.set_orders(&[0, 255]);
        let mut bytes = bad.write();
        bytes.pop();
        std::fs::write(dir.path().join("duplicates.it"), bytes).unwrap();
        let error = load_assets(dir.path(), &manifest.assets, TextureFormat::Rgba8).unwrap_err();
        assert!(error.to_string().contains("IT sample extraction"));
    }

    #[test]
    fn it_packed_handles_follow_samples_not_instruments() {
        use crate::pack::assets::audio::load_tracker;
        use nether_it::{ItFlags, ItInstrument, ItSample, ItWriter};
        for instrument_mode in [false, true] {
            let dir = tempdir().unwrap();
            let path = dir.path().join("mapping.it");
            let mut writer = ItWriter::new("Sample mapping");
            if instrument_mode {
                writer.set_flags(ItFlags::INSTRUMENTS);
                writer.add_instrument(ItInstrument {
                    name: "Lead".into(),
                    ..Default::default()
                });
            } else {
                writer.set_flags(ItFlags::from_bits(0));
            }
            writer.add_sample(
                ItSample {
                    name: "Saw".into(),
                    ..Default::default()
                },
                &[1, 2, 3, 4],
            );
            writer.add_pattern(1);
            writer.set_orders(&[0, 255]);
            std::fs::write(&path, writer.write()).unwrap();
            let available = [sanitize_name("Saw", "music", 0)].into_iter().collect();
            let packed = load_tracker("music", &path, &available, None).unwrap();
            assert_eq!(packed.sample_ids, vec![sanitize_name("Saw", "music", 0)]);
        }
    }

    #[test]
    fn test_manifest_parsing() {
        let manifest_toml = r#"
[game]
id = "test-game"
title = "Test Game"
author = "Test Author"
version = "1.0.0"
description = "A test game"
tags = ["action", "puzzle"]

[assets]
"#;
        let manifest = NetherManifest::parse(manifest_toml).unwrap();
        assert_eq!(manifest.game.id, "test-game");
        assert_eq!(manifest.game.title, "Test Game");
        assert_eq!(manifest.game.author, "Test Author");
        assert_eq!(manifest.game.version, "1.0.0");
        assert_eq!(manifest.game.description, "A test game");
        assert_eq!(manifest.game.tags, vec!["action", "puzzle"]);
    }

    #[test]
    fn test_manifest_with_assets() {
        let manifest_toml = r#"
[game]
id = "asset-game"
title = "Asset Game"
author = "Author"
version = "0.1.0"

[[assets.textures]]
id = "player"
path = "assets/player.png"

[[assets.textures]]
id = "enemy"
path = "assets/enemy.png"

[[assets.epu_environments]]
id = "studio"
px = "env/px.png"
nx = "env/nx.png"
py = "env/py.png"
ny = "env/ny.png"
pz = "env/pz.png"
nz = "env/nz.png"

[[assets.sounds]]
id = "jump"
path = "assets/jump.wav"

[[assets.data]]
id = "level1"
path = "assets/level1.bin"
"#;
        let manifest = NetherManifest::parse(manifest_toml).unwrap();
        assert_eq!(manifest.assets.textures.len(), 2);
        assert_eq!(manifest.assets.textures[0].id, Some("player".to_string()));
        assert_eq!(manifest.assets.textures[1].id, Some("enemy".to_string()));
        assert_eq!(manifest.assets.epu_environments.len(), 1);
        assert_eq!(manifest.assets.epu_environments[0].id, "studio");
        assert_eq!(manifest.assets.sounds.len(), 1);
        assert_eq!(manifest.assets.sounds[0].id, Some("jump".to_string()));
        assert_eq!(manifest.assets.data.len(), 1);
        assert_eq!(manifest.assets.data[0].id, Some("level1".to_string()));
    }

    #[test]
    fn test_manifest_minimal() {
        let manifest_toml = r#"
[game]
id = "minimal"
title = "Minimal Game"
author = "Author"
version = "1.0.0"
"#;
        let manifest = NetherManifest::parse(manifest_toml).unwrap();
        assert_eq!(manifest.game.id, "minimal");
        assert!(manifest.game.description.is_empty());
        assert!(manifest.game.tags.is_empty());
        assert!(manifest.assets.textures.is_empty());
    }

    #[test]
    fn test_load_data_file() {
        let dir = tempdir().unwrap();
        let data_path = dir.path().join("test.bin");

        // Write test data
        let test_data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        std::fs::write(&data_path, &test_data).unwrap();

        // Load it
        let packed = load_data("test_asset", &data_path).unwrap();
        assert_eq!(packed.id, "test_asset");
        assert_eq!(packed.data, test_data);
    }

    #[test]
    fn test_load_texture_png_rgba8() {
        let dir = tempdir().unwrap();
        let img_path = dir.path().join("test.png");

        // Create a minimal 2x2 PNG
        let img = image::RgbaImage::from_fn(2, 2, |x, y| {
            if (x + y) % 2 == 0 {
                image::Rgba([255, 0, 0, 255]) // Red
            } else {
                image::Rgba([0, 255, 0, 255]) // Green
            }
        });
        img.save(&img_path).unwrap();

        // Load it as RGBA8
        let packed = load_texture("test_tex", &img_path, TextureFormat::Rgba8).unwrap();
        assert_eq!(packed.id, "test_tex");
        assert_eq!(packed.width, 2);
        assert_eq!(packed.height, 2);
        assert_eq!(packed.format, TextureFormat::Rgba8);
        assert_eq!(packed.data.len(), 2 * 2 * 4); // RGBA8
    }

    #[test]
    fn test_load_texture_png_bc7() {
        let dir = tempdir().unwrap();
        let img_path = dir.path().join("test.png");

        // Create a 16x16 PNG (must be at least 4x4 for BC7)
        let img = image::RgbaImage::from_fn(16, 16, |x, y| {
            if (x + y) % 2 == 0 {
                image::Rgba([255, 0, 0, 255]) // Red
            } else {
                image::Rgba([0, 255, 0, 255]) // Green
            }
        });
        img.save(&img_path).unwrap();

        // Load it as BC7
        let packed = load_texture("test_tex", &img_path, TextureFormat::Bc7).unwrap();
        assert_eq!(packed.id, "test_tex");
        assert_eq!(packed.width, 16);
        assert_eq!(packed.height, 16);
        assert_eq!(packed.format, TextureFormat::Bc7);
        // BC7: 4x4 blocks = 16 blocks x 16 bytes = 256 bytes
        assert_eq!(packed.data.len(), 4 * 4 * 16);
    }

    #[test]
    fn test_load_epu_environment_rgba8() {
        let dir = tempdir().unwrap();
        let env_dir = dir.path().join("env");
        std::fs::create_dir_all(&env_dir).unwrap();

        for (name, color) in [
            ("px.png", [255, 0, 0, 255]),
            ("nx.png", [0, 255, 0, 255]),
            ("py.png", [0, 0, 255, 255]),
            ("ny.png", [255, 255, 0, 255]),
            ("pz.png", [255, 0, 255, 255]),
            ("nz.png", [0, 255, 255, 255]),
        ] {
            let img = image::RgbaImage::from_pixel(4, 4, image::Rgba(color));
            img.save(env_dir.join(name)).unwrap();
        }

        let entry = crate::manifest::EpuEnvironmentEntry {
            id: "axis_room".to_string(),
            px: "env/px.png".to_string(),
            nx: "env/nx.png".to_string(),
            py: "env/py.png".to_string(),
            ny: "env/ny.png".to_string(),
            pz: "env/pz.png".to_string(),
            nz: "env/nz.png".to_string(),
        };

        let packed = load_epu_environment("axis_room", dir.path(), &entry).unwrap();
        assert_eq!(packed.id, "axis_room");
        assert_eq!(packed.width, 4);
        assert_eq!(packed.height, 4);
        assert!(packed.validate());
    }

    #[test]
    fn test_load_wav_basic() {
        let dir = tempdir().unwrap();
        let wav_path = dir.path().join("test.wav");

        // Create a minimal WAV file (44 byte header + 8 samples)
        let mut wav_data = vec![];

        // RIFF header
        wav_data.extend_from_slice(b"RIFF");
        wav_data.extend_from_slice(&52u32.to_le_bytes()); // File size - 8
        wav_data.extend_from_slice(b"WAVE");

        // fmt chunk
        wav_data.extend_from_slice(b"fmt ");
        wav_data.extend_from_slice(&16u32.to_le_bytes()); // Chunk size
        wav_data.extend_from_slice(&1u16.to_le_bytes()); // Audio format (PCM)
        wav_data.extend_from_slice(&1u16.to_le_bytes()); // Num channels (mono)
        wav_data.extend_from_slice(&22050u32.to_le_bytes()); // Sample rate
        wav_data.extend_from_slice(&44100u32.to_le_bytes()); // Byte rate
        wav_data.extend_from_slice(&2u16.to_le_bytes()); // Block align
        wav_data.extend_from_slice(&16u16.to_le_bytes()); // Bits per sample

        // data chunk
        wav_data.extend_from_slice(b"data");
        wav_data.extend_from_slice(&16u32.to_le_bytes()); // Chunk size (8 samples * 2 bytes)

        // Audio samples
        let samples: [i16; 8] = [0, 1000, 2000, 3000, 2000, 1000, 0, -1000];
        for sample in &samples {
            wav_data.extend_from_slice(&sample.to_le_bytes());
        }

        std::fs::write(&wav_path, &wav_data).unwrap();

        // Load it
        let packed = load_sound("test_sfx", &wav_path).unwrap();
        assert_eq!(packed.id, "test_sfx");
        assert_eq!(packed.data.len(), 8);
        assert_eq!(packed.data[0], 0);
        assert_eq!(packed.data[1], 1000);
    }

    #[test]
    fn test_load_wav_invalid() {
        let dir = tempdir().unwrap();
        let bad_path = dir.path().join("bad.wav");

        // Not a WAV file
        std::fs::write(&bad_path, b"not a wav file").unwrap();

        let result = load_sound("bad", &bad_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_assets_section() {
        let dir = tempdir().unwrap();
        let assets = AssetsSection::default();

        let pack = load_assets(dir.path(), &assets, TextureFormat::Rgba8).unwrap();
        assert!(pack.is_empty());
        assert_eq!(pack.asset_count(), 0);
    }

    #[test]
    fn test_load_mesh_nczxmesh() {
        let dir = tempdir().unwrap();
        let mesh_path = dir.path().join("test.nczxmesh");

        // Create a minimal NetherZXMesh file
        // Format 0 = position only (8 bytes per vertex)
        // 3 vertices, 3 indices (a triangle)
        let header = NetherZXMeshHeader::new(3, 3, 0);
        let mut mesh_data = header.to_bytes().to_vec();

        // Add vertex data (3 vertices * 8 bytes = 24 bytes)
        // Position is f16x4, but we just use placeholder bytes
        mesh_data.extend_from_slice(&[0u8; 24]);

        // Add index data (3 indices * 2 bytes = 6 bytes)
        mesh_data.extend_from_slice(&0u16.to_le_bytes()); // index 0
        mesh_data.extend_from_slice(&1u16.to_le_bytes()); // index 1
        mesh_data.extend_from_slice(&2u16.to_le_bytes()); // index 2

        std::fs::write(&mesh_path, &mesh_data).unwrap();

        // Load it
        let packed = load_mesh("test_mesh", &mesh_path).unwrap();
        assert_eq!(packed.id, "test_mesh");
        assert_eq!(packed.format, 0);
        assert_eq!(packed.vertex_count, 3);
        assert_eq!(packed.index_count, 3);
        assert_eq!(packed.vertex_data.len(), 24);
        assert_eq!(packed.index_data.len(), 3);
        assert_eq!(packed.index_data, vec![0, 1, 2]);
    }

    #[test]
    fn test_load_mesh_with_uv_and_color() {
        let dir = tempdir().unwrap();
        let mesh_path = dir.path().join("test_uv_color.nczxmesh");

        // Format 3 = position (8) + UV (4) + color (4) = 16 bytes per vertex
        let format = FORMAT_UV | FORMAT_COLOR;

        let header = NetherZXMeshHeader::new(4, 6, format);
        let mut mesh_data = header.to_bytes().to_vec();

        // Add vertex data (4 vertices * 16 bytes = 64 bytes)
        mesh_data.extend_from_slice(&[0u8; 64]);

        // Add index data (6 indices * 2 bytes = 12 bytes)
        for i in 0u16..6 {
            mesh_data.extend_from_slice(&i.to_le_bytes());
        }

        std::fs::write(&mesh_path, &mesh_data).unwrap();

        // Load it
        let packed = load_mesh("uv_color_mesh", &mesh_path).unwrap();
        assert_eq!(packed.format, format);
        assert_eq!(packed.vertex_count, 4);
        assert_eq!(packed.index_count, 6);
        assert_eq!(packed.vertex_data.len(), 64);
        assert_eq!(packed.index_data.len(), 6);
    }

    #[test]
    fn test_load_mesh_invalid_too_small() {
        let dir = tempdir().unwrap();
        let mesh_path = dir.path().join("bad.nczxmesh");

        // File too small to contain header
        std::fs::write(&mesh_path, [0u8; 5]).unwrap();

        let result = load_mesh("bad", &mesh_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_mesh_invalid_truncated_data() {
        let dir = tempdir().unwrap();
        let mesh_path = dir.path().join("truncated.nczxmesh");

        // Valid header but truncated vertex data
        let header = NetherZXMeshHeader::new(10, 0, 0); // Claims 10 vertices (80 bytes needed)
        let mut mesh_data = header.to_bytes().to_vec();
        mesh_data.extend_from_slice(&[0u8; 20]); // Only 20 bytes provided

        std::fs::write(&mesh_path, &mesh_data).unwrap();

        let result = load_mesh("truncated", &mesh_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_vertex_stride_packed() {
        // Verify z_common::vertex_stride_packed works as expected
        // Position only
        assert_eq!(vertex_stride_packed(0), 8);

        // Position + UV
        assert_eq!(vertex_stride_packed(1), 12);

        // Position + Color
        assert_eq!(vertex_stride_packed(2), 12);

        // Position + UV + Color
        assert_eq!(vertex_stride_packed(3), 16);

        // Position + Normal
        assert_eq!(vertex_stride_packed(4), 12);

        // Position + UV + Color + Normal
        assert_eq!(vertex_stride_packed(7), 20);

        // Position + Skinned
        assert_eq!(vertex_stride_packed(8), 16);

        // All features
        assert_eq!(vertex_stride_packed(15), 28);
    }

    #[test]
    fn test_load_keyframes_nczxanim() {
        let dir = tempdir().unwrap();
        let anim_path = dir.path().join("test.nczxanim");

        // Create a minimal .nczxanim file
        // 2 bones, 3 frames (2 * 3 * 16 = 96 bytes of data)
        let header = NetherZXAnimationHeader::new(2, 3);
        let mut anim_data = header.to_bytes().to_vec();

        // Add frame data (96 bytes)
        anim_data.extend_from_slice(&[0u8; 96]);

        std::fs::write(&anim_path, &anim_data).unwrap();

        // Load it
        let packed = load_keyframes("test_anim", &anim_path, None, None).unwrap();
        assert_eq!(packed.id, "test_anim");
        assert_eq!(packed.bone_count, 2);
        assert_eq!(packed.frame_count, 3);
        assert_eq!(packed.data.len(), 96);
        assert!(packed.validate());
    }

    #[test]
    fn test_load_keyframes_invalid_header() {
        let dir = tempdir().unwrap();
        let bad_path = dir.path().join("bad.nczxanim");

        // Invalid header (bone_count = 0)
        let mut data = vec![0u8; 4];
        data[0] = 0; // bone_count = 0 (invalid)
        data[1] = 0; // flags
        data[2] = 10; // frame_count LSB
        data[3] = 0; // frame_count MSB

        std::fs::write(&bad_path, &data).unwrap();

        let result = load_keyframes("bad", &bad_path, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_keyframes_truncated() {
        let dir = tempdir().unwrap();
        let trunc_path = dir.path().join("truncated.nczxanim");

        // Valid header but truncated data
        let header = NetherZXAnimationHeader::new(5, 10); // 5 bones, 10 frames = 800 bytes
        let mut data = header.to_bytes().to_vec();
        data.extend_from_slice(&[0u8; 100]); // Only 100 bytes instead of 800

        std::fs::write(&trunc_path, &data).unwrap();

        let result = load_keyframes("truncated", &trunc_path, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_manifest_with_keyframes() {
        let manifest_toml = r#"
[game]
id = "anim-game"
title = "Animation Game"
author = "Author"
version = "0.1.0"

[[assets.keyframes]]
id = "walk"
path = "assets/walk.nczxanim"

[[assets.keyframes]]
id = "run"
path = "assets/run.nczxanim"
"#;
        let manifest = NetherManifest::parse(manifest_toml).unwrap();
        assert_eq!(manifest.assets.keyframes.len(), 2);
        assert_eq!(manifest.assets.keyframes[0].id, Some("walk".to_string()));
        assert_eq!(manifest.assets.keyframes[1].id, Some("run".to_string()));
    }

    #[test]
    fn test_manifest_wildcard_animations() {
        // Test wildcard animation import syntax (no id, no animation_name)
        let manifest_toml = r#"
[game]
id = "wildcard-test"
title = "Wildcard Test"
author = "Author"
version = "0.1.0"

[[assets.animations]]
path = "models/player.glb"
skin_name = "Armature"
id_prefix = "player_"
"#;
        let manifest = NetherManifest::parse(manifest_toml).unwrap();
        assert_eq!(manifest.assets.animations.len(), 1);

        let entry = &manifest.assets.animations[0];
        assert!(entry.id.is_none()); // Wildcard: no explicit id
        assert!(entry.animation_name.is_none()); // Wildcard: no animation_name
        assert_eq!(entry.skin_name, Some("Armature".to_string()));
        assert_eq!(entry.id_prefix, Some("player_".to_string()));
    }

    #[test]
    fn test_manifest_mixed_animations() {
        // Test mixing explicit and wildcard animation entries
        let manifest_toml = r#"
[game]
id = "mixed-test"
title = "Mixed Test"
author = "Author"
version = "0.1.0"

[[assets.animations]]
id = "custom_idle"
path = "models/player.glb"
animation_name = "Idle"
skin_name = "Armature"

[[assets.animations]]
path = "models/enemy.glb"
skin_name = "Armature"
id_prefix = "enemy_"
"#;
        let manifest = NetherManifest::parse(manifest_toml).unwrap();
        assert_eq!(manifest.assets.animations.len(), 2);

        // First entry: explicit
        let entry1 = &manifest.assets.animations[0];
        assert_eq!(entry1.id, Some("custom_idle".to_string()));
        assert_eq!(entry1.animation_name, Some("Idle".to_string()));

        // Second entry: wildcard
        let entry2 = &manifest.assets.animations[1];
        assert!(entry2.id.is_none());
        assert!(entry2.animation_name.is_none());
        assert_eq!(entry2.id_prefix, Some("enemy_".to_string()));
    }

    // =========================================================================
    // Tests for XM Sample Extraction Helper Functions
    // =========================================================================

    #[test]
    fn test_sanitize_name_basic() {
        // Basic sanitization
        assert_eq!(sanitize_name("kick", "track", 0), "kick");
        assert_eq!(sanitize_name("Kick", "track", 0), "kick");
        assert_eq!(sanitize_name("KICK", "track", 0), "kick");
    }

    #[test]
    fn test_sanitize_name_whitespace() {
        // Whitespace trimming
        assert_eq!(sanitize_name("  kick  ", "track", 0), "kick");
        assert_eq!(sanitize_name("\tkick\t", "track", 0), "kick");
        assert_eq!(sanitize_name("  My Kick  ", "track", 0), "my_kick");
    }

    #[test]
    fn test_sanitize_name_special_chars() {
        // Special character replacement
        assert_eq!(sanitize_name("My Kick!", "track", 0), "my_kick");
        assert_eq!(sanitize_name("kick@drum", "track", 0), "kick_drum");
        assert_eq!(sanitize_name("kick#1", "track", 0), "kick_1");
        assert_eq!(sanitize_name("kick$drum", "track", 0), "kick_drum");
        assert_eq!(sanitize_name("kick%drum", "track", 0), "kick_drum");
    }

    #[test]
    fn test_sanitize_name_multiple_underscores() {
        // Multiple special chars should not create consecutive underscores
        assert_eq!(sanitize_name("kick!!!drum", "track", 0), "kick_drum");
        assert_eq!(sanitize_name("kick   drum", "track", 0), "kick_drum");
    }

    #[test]
    fn test_sanitize_name_valid_chars() {
        // Valid characters should be preserved
        assert_eq!(sanitize_name("kick_drum", "track", 0), "kick_drum");
        assert_eq!(sanitize_name("kick-drum", "track", 0), "kick-drum");
        assert_eq!(sanitize_name("kick123", "track", 0), "kick123");
    }

    #[test]
    fn test_sanitize_name_empty() {
        // Empty names should generate from tracker ID and index
        assert_eq!(sanitize_name("", "boss_theme", 0), "boss_theme_inst0");
        assert_eq!(sanitize_name("  ", "boss_theme", 1), "boss_theme_inst1");
        assert_eq!(sanitize_name("\t\n", "boss_theme", 5), "boss_theme_inst5");
    }

    #[test]
    fn test_sanitize_name_leading_trailing_underscores() {
        // Leading/trailing underscores should be removed
        assert_eq!(sanitize_name("_kick_", "track", 0), "kick");
        assert_eq!(sanitize_name("___kick___", "track", 0), "kick");
    }

    #[test]
    fn test_sanitize_name_unicode() {
        // Unicode characters should be replaced
        assert_eq!(sanitize_name("kick♪drum", "track", 0), "kick_drum");
        assert_eq!(sanitize_name("kick🥁drum", "track", 0), "kick_drum");
    }

    #[test]
    fn test_sanitize_name_real_world_examples() {
        // Real-world instrument names from tracker files
        assert_eq!(sanitize_name("BD_KICK_01", "track", 0), "bd_kick_01");
        assert_eq!(
            sanitize_name("Snare (layered)", "track", 0),
            "snare_layered"
        );
        assert_eq!(
            sanitize_name("Hi-Hat [Closed]", "track", 0),
            "hi-hat_closed"
        );
        assert_eq!(sanitize_name("Bass: Deep Sub", "track", 0), "bass_deep_sub");
    }

    #[test]
    fn test_hash_sample_data_consistency() {
        // Same data should produce same hash
        let data1 = vec![100i16, 200, 300, 400, 500];
        let data2 = vec![100i16, 200, 300, 400, 500];

        let hash1 = hash_sample_data(&data1);
        let hash2 = hash_sample_data(&data2);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_sample_data_different() {
        // Different data should produce different hashes
        let data1 = vec![100i16, 200, 300, 400, 500];
        let data2 = vec![100i16, 200, 300, 400, 501]; // Last value different

        let hash1 = hash_sample_data(&data1);
        let hash2 = hash_sample_data(&data2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_sample_data_empty() {
        // Empty data should hash successfully
        let empty: Vec<i16> = Vec::new();
        let hash = hash_sample_data(&empty);

        // Should produce a valid 32-byte hash
        assert_eq!(hash.len(), 32);

        // Empty data should produce consistent hash
        let hash2 = hash_sample_data(&empty);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_hash_sample_data_single_sample() {
        // Single sample should hash
        let data = vec![42i16];
        let hash = hash_sample_data(&data);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_hash_sample_data_large() {
        // Large sample (simulating real audio)
        let data: Vec<i16> = (0..22050).map(|i| (i % 1000) as i16).collect();
        let hash = hash_sample_data(&data);
        assert_eq!(hash.len(), 32);

        // Should be deterministic
        let hash2 = hash_sample_data(&data);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_hash_sample_data_order_matters() {
        // Order should matter for hashing
        let data1 = vec![1i16, 2, 3, 4, 5];
        let data2 = vec![5i16, 4, 3, 2, 1]; // Reversed

        let hash1 = hash_sample_data(&data1);
        let hash2 = hash_sample_data(&data2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_sample_data_similar_but_different() {
        // Very similar data should still produce different hashes
        let data1 = vec![1000i16; 100]; // 100 samples of value 1000
        let data2 = vec![1001i16; 100]; // 100 samples of value 1001

        let hash1 = hash_sample_data(&data1);
        let hash2 = hash_sample_data(&data2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_collision_resistance() {
        // Generate multiple different samples and verify no collisions
        let mut hashes = std::collections::HashSet::new();

        for i in 0..100 {
            let data: Vec<i16> = vec![i as i16; 10];
            let hash = hash_sample_data(&data);

            // Should be no collisions
            assert!(hashes.insert(hash), "Hash collision detected!");
        }

        assert_eq!(hashes.len(), 100);
    }
}
