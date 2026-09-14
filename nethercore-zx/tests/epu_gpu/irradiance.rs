//! Analytic Lambert oracle plus real runtime source-selection regression.
use super::*;
use nethercore_zx::graphics::epu::runtime::ImportedCubeFaces;
use nethercore_zx::graphics::epu::{EpuBuilder, EpuRuntime, EpuRuntimeSettings, RampParams};

// Original pre-RED tolerance, retained after full-source quadrature repair.
const COEFFICIENT_TOLERANCE: f32 = 0.04;
const SOURCE_TOLERANCE: f32 = 0.0001;
const N: u32 = 128;
type Coefficients = [[f32; 4]; 9];

fn read_sh(d: &wgpu::Device, q: &wgpu::Queue, source: &wgpu::Buffer, id: u32) -> Coefficients {
    let b = d.create_buffer(&wgpu::BufferDescriptor {
        label: Some("irradiance readback"),
        size: 144,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut e = d.create_command_encoder(&Default::default());
    e.copy_buffer_to_buffer(source, u64::from(id) * 144, &b, 0, 144);
    q.submit([e.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    b.slice(..)
        .map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
    d.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    rx.recv().unwrap().unwrap();
    *bytemuck::from_bytes::<Coefficients>(&b.slice(..).get_mapped_range())
}

fn extract(d: &wgpu::Device, q: &wgpu::Queue, view: &wgpu::TextureView, id: u32) -> Coefficients {
    let p = pipeline(d, EPU_COMPUTE_IRRAD.to_owned(), "epu_extract_sh9");
    let active = buffer(d, &[id], wgpu::BufferUsages::STORAGE);
    let uniforms = buffer(d, &[1, 0, 0, 0], wgpu::BufferUsages::UNIFORM);
    let output = buffer(
        d,
        &vec![0; (id as usize + 1) * 36],
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
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
                binding: 2,
                resource: active.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(view),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: output.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 7,
                resource: uniforms.as_entire_binding(),
            },
        ],
    });
    let mut e = d.create_command_encoder(&Default::default());
    {
        let mut pass = e.begin_compute_pass(&Default::default());
        pass.set_pipeline(&p);
        pass.set_bind_group(0, &group, &[]);
        pass.dispatch_workgroups(1, 1, 1);
    }
    q.submit([e.finish()]);
    read_sh(d, q, &output, id)
}

fn field_texture(
    d: &wgpu::Device,
    q: &wgpu::Queue,
    size: u32,
    field: impl Fn(f32, f32) -> f32,
) -> wgpu::Texture {
    let t = texture(
        d,
        size,
        size,
        wgpu::TextureFormat::Rgba16Float,
        wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    );
    let mut pixels = Vec::new();
    for y in 0..size {
        for x in 0..size {
            let value = field(
                2.0 * (x as f32 + 0.5) / size as f32 - 1.0,
                2.0 * (y as f32 + 0.5) / size as f32 - 1.0,
            );
            pixels.extend([value, value, value, 1.0].map(|v| half::f16::from_f32(v).to_bits()));
        }
    }
    q.write_texture(
        t.as_image_copy(),
        bytemuck::cast_slice(&pixels),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(size * 8),
            rows_per_image: None,
        },
        t.size(),
    );
    t
}

fn record(name: &str, coefficients: &Coefficients) {
    println!("{name}: {coefficients:?}");
    // Opt-in captures keep routine tests free of filesystem writes.
    if let Ok(dir) = std::env::var("EPU_IRRADIANCE_EVIDENCE") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            std::path::Path::new(&dir).join(format!("{name}.txt")),
            format!("GPU SH9 RGB plus padding (padding is not evaluated):\n{coefficients:#?}\n"),
        )
        .unwrap();
    }
}

fn analytic_error(c: &Coefficients, a: f32) -> f32 {
    // Closed-form integral, independent of the production sampler and SH projection:
    // L=1+a*P2(z) -> E(n)=pi*(1+a/4*P2(n.z)).
    let pi = std::f32::consts::PI;
    let mut expected = [0.0; 9];
    expected[0] = pi * (4.0 * pi).sqrt();
    expected[6] = a * pi / 4.0 * (4.0 * pi / 5.0).sqrt();
    let mut error = 0.0f32;
    for i in 0..9 {
        for channel in 0..3 {
            assert!(c[i][channel].is_finite());
            error = error.max((c[i][channel] - expected[i]).abs());
        }
    }
    error
}

#[test]
fn irradiance_analytic_lambert_coefficients() {
    let (d, q) = gpu();
    let mut errors = Vec::new();
    for (name, a) in [("constant", 0.0), ("p2", 0.8)] {
        let t = field_texture(&d, &q, N, |x, y| {
            let mut v = glam::Vec3::new(x, y, 1.0 - x.abs() - y.abs());
            if v.z < 0.0 {
                v.x = (1.0 - y.abs()) * x.signum();
                v.y = (1.0 - x.abs()) * y.signum();
            }
            let z = v.normalize().z;
            1.0 + a * (3.0 * z * z - 1.0) / 2.0
        });
        let view = t.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        let c = extract(&d, &q, &view, 0);
        record(name, &c);
        let error = analytic_error(&c, a);
        println!("{name}: maximum coefficient error={error}, tolerance={COEFFICIENT_TOLERANCE}");
        if a == 0.0 {
            // A conservative all-normal reconstruction bound from |Y_lm| maxima.
            let bounds = [
                0.282095, 0.488603, 0.488603, 0.488603, 0.546274, 0.546274, 0.630784, 0.546274,
                0.546274,
            ];
            let deviation = (c[0][0] * bounds[0] - std::f32::consts::PI).abs()
                + (1..9).map(|i| c[i][0].abs() * bounds[i]).sum::<f32>();
            assert!(
                deviation < 0.04,
                "constant reconstruction bound={deviation}"
            );
        }
        errors.push(error);
    }
    assert!(
        errors.iter().all(|e| *e < COEFFICIENT_TOLERANCE),
        "Lambert coefficient errors={errors:?}"
    );
}

#[test]
fn irradiance_runtime_uses_source_mip_zero() {
    let (d, q) = gpu();
    let mut runtime = EpuRuntime::new_with_settings(
        &d,
        EpuRuntimeSettings {
            map_size: N,
            min_mip_size: 4,
        },
    );
    let mut builder = EpuBuilder::new();
    builder.ramp_bounds(RampParams {
        sky_color: [255; 3],
        wall_color: [128; 3],
        floor_color: [32; 3],
        ..Default::default()
    });
    let config = builder.finish();
    let mut errors = Vec::new();
    let mut imported_errors = Vec::new();
    for (name, id, imported, face_size) in [
        ("single", 0, false, N),
        ("batch", 3, false, N),
        ("import", 5, true, N),
        ("import-grown", 17, true, N * 2),
        ("batch-grown", 18, false, N),
    ] {
        let mut e = d.create_command_encoder(&Default::default());
        if imported {
            let faces: [_; 6] = std::array::from_fn(|face| {
                field_texture(&d, &q, face_size, |u, v| {
                    let z2 = match face {
                        0 | 1 => u * u / (1.0 + u * u + v * v),
                        2 | 3 => v * v / (1.0 + u * u + v * v),
                        _ => 1.0 / (1.0 + u * u + v * v),
                    };
                    1.0 + 0.8 * (3.0 * z2 - 1.0) / 2.0
                })
            });
            let views = faces.each_ref().map(|t| t.create_view(&Default::default()));
            runtime.build_imported_envs(
                &d,
                &q,
                &mut e,
                &[ImportedCubeFaces {
                    env_id: id,
                    face_size,
                    faces: views.each_ref(),
                }],
            );
        } else if name == "single" {
            runtime.build_env(&d, &q, &mut e, &config);
        } else {
            runtime.build_envs(&d, &q, &mut e, &[(id, &config)]);
        }
        q.submit([e.finish()]);
        let actual = read_sh(&d, &q, runtime.sh9_buffer(), id);
        let source = extract(&d, &q, runtime.env_radiance_mip_view(0).unwrap(), id);
        record(name, &actual);
        record(&format!("{name}-mip0"), &source);
        let mut error = 0.0f32;
        for i in 0..9 {
            for channel in 0..3 {
                assert!(actual[i][channel].is_finite());
                error = error.max((actual[i][channel] - source[i][channel]).abs());
            }
        }
        println!(
            "{name}: runtime/mip0 coefficient difference={error}, tolerance={SOURCE_TOLERANCE}"
        );
        errors.push(error);
        if imported {
            imported_errors.push(analytic_error(&actual, 0.8));
        }
    }
    assert!(
        errors.iter().all(|e| *e < SOURCE_TOLERANCE),
        "runtime must extract mip0: {errors:?}"
    );
    assert!(
        imported_errors.iter().all(|e| *e < COEFFICIENT_TOLERANCE),
        "imported Lambert errors={imported_errors:?}"
    );
}

// Independent spherical-polygon solid angle; no production quadrature reused.
pub(super) fn oct_direction(x: f64, y: f64) -> glam::DVec3 {
    let mut v = glam::DVec3::new(x, y, 1.0 - x.abs() - y.abs());
    if v.z < 0.0 {
        v.x = (1.0 - y.abs()) * if x >= 0.0 { 1.0 } else { -1.0 };
        v.y = (1.0 - x.abs()) * if y >= 0.0 { 1.0 } else { -1.0 };
    }
    v.normalize()
}
fn triangle_area(a: glam::DVec3, b: glam::DVec3, c: glam::DVec3) -> f64 {
    2.0 * a
        .dot(b.cross(c))
        .abs()
        .atan2(1.0 + a.dot(b) + b.dot(c) + c.dot(a))
}
#[test]
fn irradiance_narrow_cap_energy_is_covered_at_every_orientation() {
    let (d, q) = gpu();
    let phi = std::f64::consts::TAU * (0.5 * 0.6180339887498949);
    let z = 1.0 - 1.0 / 64.0f64;
    let first = glam::DVec3::new(
        phi.cos() * (1.0 - z * z).sqrt(),
        phi.sin() * (1.0 - z * z).sqrt(),
        z,
    );
    let cutoff = 5.0f64.to_radians().cos();
    let mut errors = Vec::new();
    for axis in [glam::DVec3::Z, first, glam::DVec3::X, -glam::DVec3::Z] {
        let field = |x: f32, y: f32| {
            if oct_direction(x as f64, y as f64).dot(axis) > cutoff {
                1.0
            } else {
                0.0
            }
        };
        let t = field_texture(&d, &q, N, field);
        let view = t.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        let c = extract(&d, &q, &view, 0);
        let mut energy = 0.0;
        for y in 0..N {
            for x in 0..N {
                let lo_x = 2.0 * x as f64 / N as f64 - 1.0;
                let lo_y = 2.0 * y as f64 / N as f64 - 1.0;
                let step = 2.0 / N as f64;
                let value = field((lo_x + step / 2.0) as f32, (lo_y + step / 2.0) as f32) as f64;
                let a = oct_direction(lo_x, lo_y);
                let b = oct_direction(lo_x + step, lo_y);
                let cc = oct_direction(lo_x + step, lo_y + step);
                let dd = oct_direction(lo_x, lo_y + step);
                energy += value * (triangle_area(a, b, cc) + triangle_area(a, cc, dd));
            }
        }
        // Piecewise-constant source texels, exact cell solid angles, Lambert l=0.
        let expected = energy * std::f64::consts::PI / (4.0 * std::f64::consts::PI).sqrt();
        let error = (c[0][0] as f64 - expected).abs();
        assert!(expected > 0.01 && c[0][0].is_finite());
        println!(
            "cap axis={axis:?} c0={} expected={expected} error={error}; tolerance=0.001",
            c[0][0]
        );
        errors.push(error);
    }
    assert!(
        errors.iter().all(|e| *e < 0.001),
        "narrow-cap source energy lost or overestimated: {errors:?}"
    );
}

// Exercise the actual mixed-source dispatcher, not a copy of its build order.
// ZXGraphics requires a surface; this Windows regression owns a hidden window.
#[cfg(target_os = "windows")]
#[test]
#[allow(deprecated)] // winit's synchronous window creation keeps this probe bounded.
fn source_growth_preserves_cached_procedural_radiance() {
    use nethercore_core::console::ConsoleResourceManager;
    use nethercore_zx::{
        graphics::ZXGraphics, resource_manager::ZResourceManager, state::ZXFFIState,
    };
    use winit::platform::windows::EventLoopBuilderExtWindows;

    let event_loop = winit::event_loop::EventLoop::builder()
        .with_any_thread(true)
        .build()
        .unwrap();
    let window = event_loop
        .create_window(winit::window::Window::default_attributes().with_visible(false))
        .unwrap();
    let mut graphics = ZXGraphics::new_blocking(std::sync::Arc::new(window)).unwrap();
    let d = graphics.device().clone();
    let q = graphics.queue().clone();
    *graphics.epu_runtime_mut() = EpuRuntime::new_with_settings(
        &d,
        EpuRuntimeSettings {
            map_size: N,
            min_mip_size: 4,
        },
    );
    let resources = ZResourceManager::new();
    let mut state = ZXFFIState::default();
    let mut builder = EpuBuilder::new();
    builder.ramp_bounds(RampParams {
        sky_color: [255; 3],
        wall_color: [128; 3],
        floor_color: [32; 3],
        ..Default::default()
    });
    let procedural = state.bind_epu_config(builder.finish());
    state.add_shading_state();
    let mut baseline = None;
    for frame in 0..4 {
        if frame == 2 {
            // The real allocator starts imports at 255; missing handles resolve
            // to the existing fallback texture. Its contents are not the oracle.
            let imported = state.bind_epu_textures([0; 6]);
            assert_eq!(imported, 255);
            assert!(imported >= graphics.epu_runtime().layer_capacity());
            state.add_shading_state();
        } else if frame == 3 {
            state.shading_pool.clear();
            state.update_environment_index(procedural);
            state.add_shading_state();
        }
        let mut encoder = d.create_command_encoder(&Default::default());
        resources.render_game_to_target(&mut graphics, &mut encoder, &state, [0.; 4]);
        q.submit([encoder.finish()]);
        let runtime = graphics.epu_runtime();
        let actual = extract(
            &d,
            &q,
            runtime.env_radiance_mip_view(0).unwrap(),
            procedural,
        );
        let capacity = runtime.layer_capacity();
        println!(
            "source-growth frame={frame} capacity={capacity} radiance c0={}",
            actual[0][0]
        );
        assert_eq!(capacity, if frame < 2 { 8 } else { 256 });
        if let Some(expected) = baseline {
            // Frozen before RED: identical source must yield exactly identical
            // GPU coefficients. Reading stale sh9_buffer alone misses this bug.
            assert_eq!(
                actual, expected,
                "frame {frame}: procedural source radiance lost"
            );
        } else {
            assert!(actual.iter().flatten().all(|v| v.is_finite()));
            assert!(actual[0][0] > 0.1, "baseline must be illuminated");
            baseline = Some(actual);
        }
    }
}

#[test]
fn irradiance_mixed_builds_in_one_submission_keep_counts() {
    let (d, q) = gpu();
    let mut epu = EpuRuntime::new_with_settings(
        &d,
        EpuRuntimeSettings {
            map_size: N,
            min_mip_size: 4,
        },
    );
    epu.ensure_layer_capacity(&d, 4);
    epu.ensure_imported_face_capacity(&d, N);
    let mut builder = EpuBuilder::new();
    builder.ramp_bounds(RampParams {
        sky_color: [255; 3],
        wall_color: [128; 3],
        floor_color: [32; 3],
        ..Default::default()
    });
    let config = builder.finish();
    let faces: [_; 6] = std::array::from_fn(|_| field_texture(&d, &q, N, |_, _| 1.0));
    let views: Vec<_> = faces
        .iter()
        .map(|t| t.create_view(&Default::default()))
        .collect();
    let import = ImportedCubeFaces {
        env_id: 2,
        faces: [
            &views[0], &views[1], &views[2], &views[3], &views[4], &views[5],
        ],
        face_size: N,
    };
    let mut encoder = d.create_command_encoder(&Default::default());
    epu.build_envs(&d, &q, &mut encoder, &[(0, &config), (1, &config)]);
    epu.build_imported_envs(&d, &q, &mut encoder, &[import]);
    q.submit([encoder.finish()]);
    for id in [0, 1, 2] {
        let actual = read_sh(&d, &q, epu.sh9_buffer(), id);
        let expected = extract(&d, &q, epu.env_radiance_mip_view(0).unwrap(), id);
        println!(
            "mixed submission env{id} actual c0={} expected c0={}",
            actual[0][0], expected[0][0]
        );
        assert!(expected[0][0] > 0.1);
        assert!(
            actual
                .iter()
                .zip(expected.iter())
                .all(|(a, b)| (0..3).all(|c| (a[c] - b[c]).abs() < SOURCE_TOLERANCE)),
            "shared dispatch count corrupted env {id}"
        );
    }
}
