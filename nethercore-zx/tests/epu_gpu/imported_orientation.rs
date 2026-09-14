use super::*;
use nethercore_zx::graphics::epu::{EpuRuntime, EpuRuntimeSettings, runtime::ImportedCubeFaces};

// Match the declared import footprint and clamped bilinear cache lookup.
// The raw background point is not the reference for a filtered cache texel.
fn cache_reference(direction: [f32; 4]) -> [f32; 3] {
    use glam::DVec3 as V;
    let n = V::new(
        direction[0] as f64,
        direction[1] as f64,
        direction[2] as f64,
    );
    let p = n / (n.x.abs() + n.y.abs() + n.z.abs());
    let (x, y) = if p.z < 0.0 {
        (
            (1.0 - p.y.abs()) * if p.x >= 0.0 { 1.0 } else { -1.0 },
            (1.0 - p.x.abs()) * if p.y >= 0.0 { 1.0 } else { -1.0 },
        )
    } else {
        (p.x, p.y)
    };
    let px = (x * 0.5 + 0.5) * 128.0 - 0.5;
    let py = (y * 0.5 + 0.5) * 128.0 - 0.5;
    let mut result = V::ZERO;
    let normals = [V::X, -V::X, V::Y, -V::Y, V::Z, -V::Z];
    let horizontal = [V::Z, -V::Z, V::X, V::X, -V::X, V::X];
    let vertical = [-V::Y, -V::Y, -V::Z, V::Z, -V::Y, -V::Y];
    for iy in 0..2 {
        for ix in 0..2 {
            let tx = (px.floor() + ix as f64).clamp(0.0, 127.0);
            let ty = (py.floor() + iy as f64).clamp(0.0, 127.0);
            let w = if ix == 0 {
                1.0 - (px - px.floor())
            } else {
                px - px.floor()
            };
            let h = if iy == 0 {
                1.0 - (py - py.floor())
            } else {
                py - py.floor()
            };
            let mut texel = V::ZERO;
            for (oy, wy) in [(-0.5, 1.0), (0.0, 2.0), (0.5, 1.0)] {
                for (ox, wx) in [(-0.5, 1.0), (0.0, 2.0), (0.5, 1.0)] {
                    let l = super::irradiance::oct_direction(
                        (tx + 0.5 + ox) / 64.0 - 1.0,
                        (ty + 0.5 + oy) / 64.0 - 1.0,
                    );
                    let face = (0..6)
                        .max_by(|a, b| l.dot(normals[*a]).total_cmp(&l.dot(normals[*b])))
                        .unwrap();
                    let depth = l.dot(normals[face]);
                    let u = (0.5 + 0.5 * l.dot(horizontal[face]) / depth)
                        .clamp(0.5 / 32.0, 1.0 - 0.5 / 32.0);
                    let v = (0.5 + 0.5 * l.dot(vertical[face]) / depth)
                        .clamp(0.5 / 32.0, 1.0 - 0.5 / 32.0);
                    texel += V::new(u, v, (face + 1) as f64 / 8.0) * (wx * wy / 16.0);
                }
            }
            result += texel * (w * h);
        }
    }
    result.as_vec3().to_array()
}

#[test]
fn imported_face_uv_matches_background_and_cache() {
    let (d, q) = gpu();
    let size = 32u32;
    // R/G encode image U/V independently; B identifies the face. Texel-center
    // values make interior bilinear interpolation an exact affine oracle.
    let faces: Vec<_> = (0..6)
        .map(|face| {
            let t = texture(
                &d,
                size,
                size,
                wgpu::TextureFormat::Rgba16Float,
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            );
            let mut data = Vec::new();
            for y in 0..size {
                for x in 0..size {
                    data.extend(
                        [
                            (x as f32 + 0.5) / size as f32,
                            (y as f32 + 0.5) / size as f32,
                            (face + 1) as f32 / 8.0,
                            1.0,
                        ]
                        .map(|v| half::f16::from_f32(v).to_bits()),
                    );
                }
            }
            q.write_texture(
                t.as_image_copy(),
                bytemuck::cast_slice(&data),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(size * 8),
                    rows_per_image: Some(size),
                },
                t.size(),
            );
            t
        })
        .collect();
    let views: Vec<_> = faces
        .iter()
        .map(|t| t.create_view(&Default::default()))
        .collect();
    let mut runtime = EpuRuntime::new_with_settings(
        &d,
        EpuRuntimeSettings {
            map_size: 128,
            min_mip_size: 4,
        },
    );
    let mut encoder = d.create_command_encoder(&Default::default());
    runtime.build_imported_envs(
        &d,
        &q,
        &mut encoder,
        &[ImportedCubeFaces {
            env_id: 3,
            faces: [
                &views[0], &views[1], &views[2], &views[3], &views[4], &views[5],
            ],
            face_size: size,
        }],
    );
    q.submit([encoder.finish()]);
    let mut directions = Vec::<[f32; 4]>::new();
    let mut expected = Vec::new();
    for face in 0..6 {
        for v in [0.25f32, 0.5, 0.75] {
            for u in [0.25f32, 0.5, 0.75] {
                let x = 2.0 * u - 1.0;
                let y = 2.0 * v - 1.0;
                // Declared top-left-image cube convention, expressed as face-to-world
                // vectors rather than copying the shader's world-to-UV selection.
                directions.push(match face {
                    0 => [1.0, -y, x, 0.0],
                    1 => [-1.0, -y, -x, 0.0],
                    2 => [x, 1.0, -y, 0.0],
                    3 => [x, -1.0, y, 0.0],
                    4 => [-x, -y, 1.0, 0.0],
                    _ => [x, -y, -1.0, 0.0],
                });
                expected.push([u, v, (face + 1) as f32 / 8.0]);
            }
        }
    }
    let inputs = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&directions),
        usage: wgpu::BufferUsages::STORAGE,
    });
    // Execute the unchanged production imported-background helper, plus mip0
    // from the actual runtime import. No private copy of either UV algorithm.
    let sampling = include_str!("../../shaders/common/20_environment/90_sampling.wgsl")
        .split("// Sample background from")
        .next()
        .unwrap();
    let source = format!(
        r#"
const EPU_IMPORTED_FACE_BASE_INVALID: u32 = 0xffffffffu;
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@group(0) @binding(1) var epu_env_radiance: texture_2d_array<f32>;
@group(0) @binding(2) var epu_sampler: sampler;
@group(0) @binding(3) var epu_imported_faces: texture_2d_array<f32>;
@group(0) @binding(4) var<storage, read> epu_imported_face_base_layers: array<u32>;
@group(0) @binding(5) var<storage, read> directions: array<vec4f>;
@group(0) @binding(6) var<storage, read> epu_source_kinds: array<u32>;
{sampling}
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p: vec3u) {{
    let dir = normalize(directions[p.x].xyz);
    let background = sample_epu_imported_cube(3u, dir);
    let uv = epu_octahedral_encode(dir) * 0.5 + 0.5;
    let cached = textureSampleLevel(epu_env_radiance, epu_sampler, uv, 3, 0.0).rgb;
    textureStore(result, vec2u(p.x, 0u), vec4f(background, 1.0));
    textureStore(result, vec2u(p.x, 1u), vec4f(cached, 1.0));
}}
"#
    );
    let p = pipeline(&d, source, "probe");
    let output = texture(
        &d,
        directions.len() as u32,
        2,
        wgpu::TextureFormat::Rgba16Float,
        wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
    );
    let output_view = output.create_view(&Default::default());
    let sampler = d.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let group = d.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &p.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&output_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(runtime.env_radiance_view()),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(runtime.imported_faces_view()),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: runtime
                    .imported_face_base_layers_buffer()
                    .as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: inputs.as_entire_binding(),
            },
        ],
    });
    let mut e = d.create_command_encoder(&Default::default());
    {
        let mut pass = e.begin_compute_pass(&Default::default());
        pass.set_pipeline(&p);
        pass.set_bind_group(0, &group, &[]);
        pass.dispatch_workgroups(directions.len() as u32, 1, 1);
    }
    q.submit([e.finish()]);
    let pixels = read(&d, &q, &output);
    assert_eq!(pixels.len(), expected.len() * 2);
    let cached_expected: Vec<_> = directions.iter().copied().map(cache_reference).collect();
    let mut errors = [0.0f32; 2];
    for (row, values) in pixels.chunks_exact(expected.len()).enumerate() {
        for (actual, reference) in values.iter().zip(if row == 0 {
            &expected
        } else {
            &cached_expected
        }) {
            assert!(actual.iter().all(|v| v.is_finite()) && actual[3] == 1.0);
            for c in 0..3 {
                errors[row] = errors[row].max((actual[c] - reference[c]).abs());
            }
        }
    }
    println!(
        "imported face UV: {} directions, background/cache errors={errors:?}; bounds=[0.001, 0.004]",
        expected.len()
    );
    assert!(
        errors[0] <= 0.001 && errors[1] <= 0.004,
        "within-face orientation/ingestion mismatch: {errors:?}"
    );
}
