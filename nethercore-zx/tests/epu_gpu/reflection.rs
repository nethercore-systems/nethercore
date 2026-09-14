//! Finite-case true BP gate: independent CPU tensor quadrature vs generated GPU materials.
//! Frozen before RED: |GPU-reference| + 0.001 reference budget <= 0.01 per RGB channel.
//! No CPU golden-rule model is used to decide GPU acceptance.
use super::*;
use glam::DVec3;

const ACCEPTANCE: f64 = 0.01;
const REFERENCE_BUDGET: f64 = 0.001;
const SIZE: u32 = 128; // Existing production default, not an increased QA resolution.
const FIELDS: usize = 6;
const CONTROLS: usize = 5;
const SHININESS: [f64; 4] = [1., 16., 64., 256.];
const NOV: [f64; 3] = [1., 0.5, 0.1];
const IMPORTED_DIRECTIONAL_COLORS: [[f32; 3]; 6] = [
    [0.83, 0.11, 0.27], // +X
    [0.19, 0.71, 0.37], // -X
    [0.31, 0.23, 0.91], // +Y
    [0.67, 0.43, 0.13], // -Y
    [0.41, 0.89, 0.53], // +Z
    [0.73, 0.17, 0.61], // -Z
];

fn basis(n: DVec3) -> (DVec3, DVec3) {
    let up = if n.y.abs() < 0.999 {
        DVec3::Y
    } else {
        DVec3::X
    };
    let t = up.cross(n).normalize();
    (t, n.cross(t))
}

fn cap_axes(s: f64, nov: f64) -> [DVec3; 4] {
    let v = DVec3::new((1. - nov * nov).sqrt(), 0., nov);
    let adversarial = [0.0f64, 31.].map(|i| {
        // The parent's adversarial caps target the rejected 64-sample rule.
        let z = ((i + 0.5) / 64.).powf(1. / (s + 1.));
        let phi = std::f64::consts::TAU * ((i + 0.5) * 0.6180339887498949).fract();
        let r = (1. - z * z).sqrt();
        let h = DVec3::new(r * phi.cos(), r * phi.sin(), z);
        2. * v.dot(h) * h - v
    });
    [
        DVec3::new(-v.x, 0., nov),
        DVec3::Z,
        adversarial[0],
        adversarial[1],
    ]
}

fn oct_direction(x: f64, y: f64) -> DVec3 {
    let mut v = DVec3::new(x, y, 1. - x.abs() - y.abs());
    if v.z < 0. {
        v.x = (1. - y.abs()) * x.signum();
        v.y = (1. - x.abs()) * y.signum();
    }
    v.normalize()
}

fn source(d: &wgpu::Device, q: &wgpu::Queue, n: DVec3) -> wgpu::Texture {
    let (t, b) = basis(n);
    let tex = d.create_texture(&wgpu::TextureDescriptor {
        label: Some("analytic fields sampled into production-size mip0"),
        size: wgpu::Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 72,
        },
        mip_level_count: 6,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    });
    // Only the radiance evaluator is injected. The actual production texel
    // filter runs on GPU, isolated before saturation because P2 peaks at 1.8.
    let mut states = Vec::<u32>::new();
    for s in SHININESS {
        for nov in NOV {
            let axes = cap_axes(s, nov);
            for field in 0..FIELDS {
                let axis = if field < 2 {
                    t
                } else {
                    let a = axes[field - 2];
                    t * a.x + b * a.y + n * a.z
                };
                let mut words = [0u32; 32];
                words[..3].copy_from_slice(&axis.to_array().map(|v| (v as f32).to_bits()));
                words[3] = field as u32;
                states.extend(words);
            }
        }
    }
    let analytic = EPU_COMPUTE_ENV.replace(
        "return evaluate_epu_layers(dir, st.layers);",
        r#"
        let axis = bitcast<vec3f>(st.layers[0].xyz);
        let field = st.layers[0].w;
        var value = 1.0;
        if field == 1u { let x = dot(dir, axis); value = 1.0 + 0.8 * (3.0*x*x-1.0)/2.0; }
        if field >= 2u { value = select(0.0, 1.0, dot(dir, axis) > cos(5.0*PI/180.0)); }
        return vec3f(value, value*0.5, value*0.25);
    "#,
    );
    assert_ne!(
        analytic, EPU_COMPUTE_ENV,
        "evaluator injection boundary changed"
    );
    let p = pipeline(
        d,
        format!(
            "{EPU_COMMON}\n{analytic}\n{}",
            r#"
@compute @workgroup_size(8,8,1)
fn source_probe(@builtin(global_invocation_id) p: vec3u) {
    if p.x >= epu_frame.map_size || p.y >= epu_frame.map_size || p.z >= epu_frame.active_count { return; }
    let id = epu_active_env_ids[p.z];
    textureStore(epu_out_sharp, p.xy, i32(id),
        vec4f(evaluate_env_texel(p.xy, epu_frame.map_size, epu_states[id]), 1.0));
}
"#
        ),
        "source_probe",
    );
    let storage = wgpu::BufferUsages::STORAGE;
    let states = buffer(d, &states, storage);
    let ids = buffer(d, &(0..72).collect::<Vec<_>>(), storage);
    let frame = buffer(d, &[72, SIZE, 0, 0], wgpu::BufferUsages::UNIFORM);
    let view = tex.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        base_mip_level: 0,
        mip_level_count: Some(1),
        ..Default::default()
    });
    let group = d.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &p.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: states.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: ids.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: frame.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&view),
            },
        ],
    });
    let mut e = d.create_command_encoder(&Default::default());
    {
        let mut pass = e.begin_compute_pass(&Default::default());
        pass.set_pipeline(&p);
        pass.set_bind_group(0, &group, &[]);
        pass.dispatch_workgroups(SIZE.div_ceil(8), SIZE.div_ceil(8), 72);
    }
    q.submit([e.finish()]);
    // Deliberately distinct higher mips: proves the reflection reads source mip0.
    // Full runtime ingestion is checked separately from this linear isolation.
    for mip in 1..6 {
        let size = SIZE >> mip;
        let pixels = vec![half::f16::from_f32(0.75).to_bits(); (size * size * 72 * 4) as usize];
        q.write_texture(
            wgpu::TexelCopyTextureInfo {
                mip_level: mip,
                ..tex.as_image_copy()
            },
            bytemuck::cast_slice(&pixels),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(size * 8),
                rows_per_image: Some(size),
            },
            wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 72,
            },
        );
    }
    tex
}

fn material(mode: u8, control: usize, s: f64) -> ([u32; 20], [f64; 3]) {
    let rgb = match control {
        0 => [0, 0, 0],
        2 => [51, 153, 255],
        _ => [255, 255, 255],
    };
    let mut words = [0; 20];
    words[0] = rgb[0] << 24 | rgb[1] << 16 | rgb[2] << 8 | 255;
    let value1 = if mode == 2 { 256. - s } else { s - 1. } as u32;
    let value0 = if mode == 2 {
        if control == 3 { 0 } else { 255 }
    } else if control == 4 {
        128
    } else {
        0
    };
    words[1] = value0 | value1 << 8; // Emissive/rim remain zero.
    // Positive rim exponent avoids undefined pow(0,0) even at zero intensity.
    words[2] = if control == 3 {
        0x0a0a0a20
    } else {
        (words[0] & 0xffffff00) | 32
    };
    words[3] = 2 | 4 | 8 | 16 | 32 | 64 | (15 << 8) | 0x10000;
    let color = if control == 3 {
        [if mode == 2 { 0.04 } else { 10. / 255. }; 3]
    } else {
        rgb.map(|v| {
            v as f64 / 255.
                * if mode == 3 && control == 4 {
                    127. / 255.
                } else {
                    1.
                }
        })
    };
    (words, color)
}

fn render(
    d: &wgpu::Device,
    q: &wgpu::Queue,
    mode: u8,
    n: DVec3,
    env: &wgpu::TextureView,
    ambient_only: bool,
    single_env: Option<u32>,
) -> Vec<[f32; 4]> {
    let frame_layout = production_bind_groups::create_frame_bind_group_layout(d, mode);
    let texture_layout = production_bind_groups::create_texture_bind_group_layout(d);
    let layout = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&frame_layout, &texture_layout],
        push_constant_ranges: &[],
    });
    // Only replace vertex input generation: fs, its material unpacking, and all
    // production lighting functions are executed unmodified in both RED/GREEN.
    let src = format!(
        "{}\n{}",
        generated_material_source(mode),
        r#"
@vertex fn reflection_vertex(@builtin(vertex_index) vertex: u32,
    @builtin(instance_index) instance: u32) -> VertexOut {
    let positions = array(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
    let data = unified_transforms[instance];
    var out: VertexOut;
    out.clip_position = vec4f(positions[vertex], 0.5, 1.);
    out.world_position = vec3f(0.);
    out.world_normal = data[0].xyz;
    out.camera_position = data[1].xyz;
    out.world_tangent = data[2].xyz;
    out.bitangent_sign = 1.;
    out.uv = vec2f(0.5);
    out.shading_state_index = instance;
    return out;
}"#
    );
    let module = d.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("reflection generated material"),
        source: wgpu::ShaderSource::Wgsl(src.into()),
    });
    let pipeline = d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("reflection_vertex"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("fs"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba16Float,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview: None,
        cache: None,
    });
    let (t, _) = basis(n);
    let mut transforms = Vec::<f32>::new();
    let mut shading = Vec::new();
    for (si, s) in SHININESS.into_iter().enumerate() {
        for (vi, nov) in NOV.into_iter().enumerate() {
            let v = t * (1. - nov * nov).sqrt() + n * nov;
            for field in 0..FIELDS {
                for control in 0..CONTROLS {
                    for axis in [n, v, t, DVec3::ZERO] {
                        transforms.extend([axis.x as f32, axis.y as f32, axis.z as f32, 0.]);
                    }
                    let (mut words, _) = material(mode, control, s);
                    if ambient_only {
                        words[0] = 0x808080ff; // Fixed albedo, no metallic/damping.
                        words[1] &= 0xff00;
                        words[2] = 32; // Zero Mode3 specular, positive rim exponent.
                    }
                    words[19] = single_env.unwrap_or(((si * 3 + vi) * FIELDS + field) as u32);
                    shading.extend(words);
                }
            }
        }
    }
    let transforms = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&transforms),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let storage = wgpu::BufferUsages::STORAGE;
    let mut sh9 = [0u32; 72 * 36];
    if ambient_only {
        // c0*Y00/PI = 1: uniform unit radiance, independently of roughness.
        let c0 = (std::f32::consts::PI / 0.282095).to_bits();
        for layer in sh9.as_chunks_mut::<36>().0.iter_mut() {
            layer[..3].fill(c0);
        }
    }
    let bindings = [
        (0, transforms),
        (1, buffer(d, &[0; 4], storage)),
        (2, buffer(d, &shading, storage)),
        (3, buffer(d, &[0; 12], storage)),
        (5, buffer(d, &[0; 4], storage)),
        (8, buffer(d, &[0; 72 * 32], storage)),
        (9, buffer(d, &[0; 4], wgpu::BufferUsages::UNIFORM)),
        (
            10,
            buffer(d, &(0..72).map(|i| i % 2).collect::<Vec<_>>(), storage),
        ),
        (11, buffer(d, &sh9, storage)),
        (13, buffer(d, &[u32::MAX; 72], storage)),
    ];
    let sampler = d.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let mut entries: Vec<_> = bindings
        .iter()
        .map(|(binding, b)| wgpu::BindGroupEntry {
            binding: *binding,
            resource: b.as_entire_binding(),
        })
        .collect();
    entries.extend([
        wgpu::BindGroupEntry {
            binding: 6,
            resource: wgpu::BindingResource::TextureView(env),
        },
        wgpu::BindGroupEntry {
            binding: 7,
            resource: wgpu::BindingResource::Sampler(&sampler),
        },
        wgpu::BindGroupEntry {
            binding: 12,
            resource: wgpu::BindingResource::TextureView(env),
        },
    ]);
    let frame = d.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &frame_layout,
        entries: &entries,
    });
    let dummy = texture(
        d,
        1,
        1,
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureUsages::TEXTURE_BINDING,
    );
    let dummy_view = dummy.create_view(&Default::default());
    let mut entries: Vec<_> = (0..4)
        .map(|binding| wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::TextureView(&dummy_view),
        })
        .collect();
    entries.extend((4..6).map(|binding| wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::Sampler(&sampler),
    }));
    let textures = d.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &texture_layout,
        entries: &entries,
    });
    let count = (72 * CONTROLS) as u32;
    let target = texture(
        d,
        count,
        1,
        wgpu::TextureFormat::Rgba16Float,
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    );
    let view = target.create_view(&Default::default());
    let mut e = d.create_command_encoder(&Default::default());
    {
        let mut pass = e.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &frame, &[]);
        pass.set_bind_group(1, &textures, &[]);
        for i in 0..count {
            pass.set_viewport(i as f32, 0., 1., 1., 0., 1.);
            pass.draw(0..3, i..i + 1);
        }
    }
    q.submit([e.finish()]);
    read(d, q, &target)
}

fn imported_runtime(
    d: &wgpu::Device,
    q: &wgpu::Queue,
    colors: [[f32; 3]; 6],
    env_id: u32,
) -> nethercore_zx::graphics::epu::EpuRuntime {
    use nethercore_zx::graphics::epu::runtime::ImportedCubeFaces;
    use nethercore_zx::graphics::epu::{EpuRuntime, EpuRuntimeSettings};
    let faces = colors.map(|rgb| {
        let face = texture(
            d,
            1,
            1,
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        let pixel = [rgb[0], rgb[1], rgb[2], 1.0].map(|v| half::f16::from_f32(v).to_bits());
        q.write_texture(
            face.as_image_copy(),
            bytemuck::cast_slice(&pixel),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(8),
                rows_per_image: Some(1),
            },
            face.size(),
        );
        face
    });
    let views = faces
        .each_ref()
        .map(|face| face.create_view(&Default::default()));
    let mut runtime = EpuRuntime::new_with_settings(
        d,
        EpuRuntimeSettings {
            map_size: SIZE,
            min_mip_size: 4,
        },
    );
    let mut encoder = d.create_command_encoder(&Default::default());
    runtime.build_imported_envs(
        d,
        q,
        &mut encoder,
        &[ImportedCubeFaces {
            env_id,
            faces: views.each_ref(),
            face_size: 1,
        }],
    );
    q.submit([encoder.finish()]);
    runtime
}

#[test]
fn reflection_imported_runtime_constant_radiance() {
    let (d, q) = gpu();
    let runtime = imported_runtime(&d, &q, [[1.0, 0.5, 0.25]; 6], 1);
    let mut worst = 0.0f64;
    for mode in [2, 3] {
        let pixels = render(
            &d,
            &q,
            mode,
            DVec3::Z,
            runtime.env_radiance_view(),
            false,
            Some(1),
        );
        for (index, actual) in pixels.iter().enumerate() {
            assert!(actual.iter().all(|v| v.is_finite()) && actual[3] == 1.0);
            let row = index / (FIELDS * CONTROLS);
            let (_, color) = material(mode, index % CONTROLS, SHININESS[row / 3]);
            for c in 0..3 {
                let expected = REFERENCE[row][0] * color[c] * [1.0, 0.5, 0.25][c];
                worst = worst.max((actual[c] as f64 - expected).abs());
            }
        }
    }
    println!(
        "imported runtime reflection constant RGB max error={worst}; total={}; bound={ACCEPTANCE}",
        worst + REFERENCE_BUDGET
    );
    assert!(worst + REFERENCE_BUDGET <= ACCEPTANCE);
}

// Incoming-direction integration, not importance sampling of half vectors.
// Partition the sphere by its dominant world axis: +X,-X,+Y,-Y,+Z,-Z.
// Constant faces need no image-UV convention. On each cube surface patch,
// P[axis]=+/-1, the other coordinates are u,v; L=P/|P| and
// dOmega=du*dv/|P|^3. This is independent of the production oct cache/filter.
fn imported_directional_reference(n: DVec3, colors: [[f64; 3]; 6], steps: usize) -> [[f64; 3]; 12] {
    let (t, _) = basis(n);
    let views = NOV.map(|nov| t * (1.0 - nov * nov).sqrt() + n * nov);
    let mut result = [[0.0; 3]; 12];
    let step = 2.0 / steps as f64;
    // ponytail: fixed two-resolution midpoint rule, no adaptive framework;
    // increase quadrature only if the explicit convergence gate fails.
    for axis in 0..3 {
        for sign in [1.0, -1.0] {
            for y in 0..steps {
                for x in 0..steps {
                    let mut p = DVec3::ZERO;
                    p[axis] = sign;
                    p[(axis + 1) % 3] = -1.0 + (x as f64 + 0.5) * step;
                    p[(axis + 2) % 3] = -1.0 + (y as f64 + 0.5) * step;
                    let length = p.length();
                    let l = p / length;
                    let nol = n.dot(l).max(0.0);
                    if nol == 0.0 {
                        continue;
                    }
                    // Source definition uses world direction, not sampled GPU texels.
                    let mut dominant = 0;
                    for candidate in 1..3 {
                        if l[candidate].abs() > l[dominant].abs() {
                            dominant = candidate;
                        }
                    }
                    let face = 2 * dominant + usize::from(l[dominant] < 0.0);
                    let weight = nol * step * step / length.powi(3);
                    for (vi, v) in views.iter().enumerate() {
                        let h = (*v + l).normalize();
                        let noh = n.dot(h).max(0.0);
                        for (si, s) in SHININESS.iter().copied().enumerate() {
                            let bp = (0.0397436 * s + 0.0856832) * noh.powi(s as i32);
                            for c in 0..3 {
                                result[si * NOV.len() + vi][c] += colors[face][c] * bp * weight;
                            }
                        }
                    }
                }
            }
        }
    }
    result
}

#[test]
fn reflection_imported_runtime_directional_radiance() {
    // RGB16F source values are frozen before either ingestion or integration.
    // Alpha is only storage padding. Each RGB is distinct, bounded below unity.
    let colors =
        IMPORTED_DIRECTIONAL_COLORS.map(|rgb| rgb.map(|v| half::f16::from_f32(v).to_f32()));
    let oracle_colors = colors.map(|rgb| rgb.map(f64::from));
    let (d, q) = gpu();
    const ENV_ID: u32 = 5; // Ordinary nonzero slot, within initial capacity.
    let runtime = imported_runtime(&d, &q, colors, ENV_ID);
    let mut convergence = 0.0f64;
    let mut worst = 0.0f64;
    let mut zero_worst = 0.0f64;
    let mut worst_case = None;
    for n in [DVec3::Z, DVec3::X, DVec3::Y] {
        // Reuse each unit-F0 RGB reference across modes, controls and duplicate
        // field pixels emitted by the existing renderer; never integrate per pixel.
        let coarse = imported_directional_reference(n, oracle_colors, 128);
        let fine = imported_directional_reference(n, oracle_colors, 256);
        for row in 0..fine.len() {
            for c in 0..3 {
                assert!(coarse[row][c].is_finite() && fine[row][c].is_finite());
                convergence = convergence.max((fine[row][c] - coarse[row][c]).abs());
            }
        }
        println!("imported directional normal={n:?} cumulative convergence={convergence}");
        assert!(
            convergence <= REFERENCE_BUDGET,
            "incoming-L quadrature did not converge within {REFERENCE_BUDGET}: {convergence}"
        );
        for mode in [2, 3] {
            let pixels = render(
                &d,
                &q,
                mode,
                n,
                runtime.env_radiance_view(),
                false,
                Some(ENV_ID),
            );
            assert_eq!(
                pixels.len(),
                SHININESS.len() * NOV.len() * FIELDS * CONTROLS
            );
            for (index, actual) in pixels.iter().enumerate() {
                assert!(
                    actual.iter().all(|v| v.is_finite()) && actual[3] == 1.0,
                    "uncovered/nonfinite imported pixel: normal={n:?} mode={mode} index={index}: {actual:?}"
                );
                let row = index / (FIELDS * CONTROLS);
                let control = index % CONTROLS;
                let s = SHININESS[row / NOV.len()];
                let nov = NOV[row % NOV.len()];
                let (_, f0) = material(mode, control, s);
                let expected: [f64; 3] = std::array::from_fn(|c| fine[row][c] * f0[c]);
                for c in 0..3 {
                    let error = (actual[c] as f64 - expected[c]).abs();
                    if error > worst {
                        worst = error;
                        worst_case = Some((n, mode, s, nov, control, *actual, expected));
                    }
                    if control == 0 {
                        zero_worst = zero_worst.max((actual[c] as f64).abs());
                    }
                }
            }
        }
    }
    println!(
        "imported directional max RGB error={worst}; reference convergence={convergence}; reserved={REFERENCE_BUDGET}; total={}; bound={ACCEPTANCE}; zero F0 max={zero_worst}; worst (N,mode,s,NoV,control,actual,expected)={worst_case:?}",
        worst + REFERENCE_BUDGET
    );
    assert!(
        worst + REFERENCE_BUDGET <= ACCEPTANCE && zero_worst == 0.0,
        "imported directional reflection violates frozen absolute RGB bound"
    );
}

#[test]
fn reflection_imported_runtime_basis_switch_continuity() {
    // Frozen continuity gate, separate from the per-image 0.01 accuracy gate.
    const DISPLACEMENT_BOUND: f64 = 2e-5;
    const REFERENCE_BOUND: f64 = 0.001;
    const RESIDUAL_JUMP_BOUND: f64 = 0.002;
    const AZIMUTHS: [f64; 4] = [
        0.0,
        std::f64::consts::FRAC_PI_4,
        std::f64::consts::FRAC_PI_2,
        3.0 * std::f64::consts::FRAC_PI_4,
    ];
    const ENV_ID: u32 = 5;
    const WHITE_F0: usize = 1;
    assert_eq!(NOV[0], 1.0);
    let colors =
        IMPORTED_DIRECTIONAL_COLORS.map(|rgb| rgb.map(|v| half::f16::from_f32(v).to_f32()));
    let oracle_colors = colors.map(|rgb| rgb.map(f64::from));
    let (d, q) = gpu();
    // One WORLD-fixed source import, never rebuilt or rotated with N or T.
    let runtime = imported_runtime(&d, &q, colors, ENV_ID);
    println!("basis continuity fixed world faces (+X,-X,+Y,-Y,+Z,-Z), exact f16 RGB={colors:?}");
    let mut worst = 0.0f64;
    let mut convergence = 0.0f64;
    let mut reference_change = 0.0f64;
    let mut checked_pairs = 0;
    let mut cases = Vec::new();
    for hemisphere in [1.0, -1.0] {
        for azimuth in AZIMUTHS {
            let normals = [0.999 - 1e-7, 0.999 + 1e-7].map(|y: f64| {
                let r = (1.0 - y * y).sqrt();
                DVec3::new(r * azimuth.cos(), hemisphere * y, r * azimuth.sin())
            });
            cases.push((format!("threshold/{hemisphere}/{azimuth}"), normals, true));
        }
    }
    // Both signs of each frame's singular axis; V=N remains continuous.
    for pole in [DVec3::Y, -DVec3::Y, DVec3::Z, -DVec3::Z] {
        let normals = [-1e-6, 1e-6].map(|x| (pole + DVec3::X * x).normalize());
        cases.push((format!("pole/{pole:?}"), normals, false));
    }
    let expected_pairs = cases.len() * 2 * SHININESS.len();
    for (case, normals, threshold) in cases {
        // Reproduce render's camera construction, not a basis-rotated camera.
        // At NoV=1 its T contribution is zero; V=N on both sides.
        let views = normals.map(|n| basis(n).0 * (1.0 - NOV[0] * NOV[0]).sqrt() + n * NOV[0]);
        assert_eq!(views, normals);
        let physical_n = normals.map(|n| n.as_vec3().normalize().as_dvec3());
        let physical_v = views.map(|v| v.as_vec3().normalize().as_dvec3());
        let dn = physical_n[1].distance(physical_n[0]);
        let dv = physical_v[1].distance(physical_v[0]);
        assert!(dn < DISPLACEMENT_BOUND && dv < DISPLACEMENT_BOUND);
        assert!(normals[1].distance(normals[0]) < DISPLACEMENT_BOUND);
        // Check the fixture straddles the switch after f32 upload/normalization;
        // this is a geometry precondition, not a source-string failure oracle.
        if threshold {
            assert!(physical_n[0].y.abs() < f64::from(0.999f32));
            assert!(physical_n[1].y.abs() >= f64::from(0.999f32));
        }
        println!(
            "basis continuity geometry case={case}; N={normals:?}; physical_N={physical_n:?}; physical_V={physical_v:?}; dN={dn}; dV={dv}; bound={DISPLACEMENT_BOUND}"
        );
        // Independent incoming-L integration once per N/resolution, reused for
        // both modes and all shininess values. At NoV=1 oracle T also drops out.
        let coarse = normals.map(|n| imported_directional_reference(n, oracle_colors, 128));
        let fine = normals.map(|n| imported_directional_reference(n, oracle_colors, 256));
        for si in 0..SHININESS.len() {
            let row = si * NOV.len();
            for c in 0..3 {
                for side in 0..2 {
                    assert!(coarse[side][row][c].is_finite() && fine[side][row][c].is_finite());
                    convergence =
                        convergence.max((fine[side][row][c] - coarse[side][row][c]).abs());
                }
                reference_change = reference_change.max((fine[1][row][c] - fine[0][row][c]).abs());
            }
        }
        println!(
            "basis continuity reference case={case}; convergence={convergence}; reference_change={reference_change}; bound={REFERENCE_BOUND}"
        );
        assert!(convergence <= REFERENCE_BOUND && reference_change <= REFERENCE_BOUND);
        for mode in [2, 3] {
            let pixels = normals.map(|n| {
                render(
                    &d,
                    &q,
                    mode,
                    n,
                    runtime.env_radiance_view(),
                    false,
                    Some(ENV_ID),
                )
            });
            for side in &pixels {
                assert_eq!(side.len(), SHININESS.len() * NOV.len() * FIELDS * CONTROLS);
            }
            for (si, s) in SHININESS.into_iter().enumerate() {
                assert_eq!(material(mode, WHITE_F0, s).1, [1.0; 3]);
                // NoV=1 only.
                let row = si * NOV.len();
                // ponytail: one representative field slot of the single env;
                // duplicate fields, other controls and NoV rows are ignored,
                // not accepted coverage. Extend only for a separate named gate.
                let index = row * FIELDS * CONTROLS + WHITE_F0;
                let actual = [pixels[0][index], pixels[1][index]];
                for pixel in actual {
                    assert!(
                        pixel.iter().all(|v| v.is_finite()) && pixel[3] == 1.0,
                        "nonfinite/nonopaque continuity pixel: case={case} mode={mode} s={s} pixel={pixel:?}"
                    );
                }
                let expected = [fine[0][row], fine[1][row]];
                let gpu_jump: [f64; 3] =
                    std::array::from_fn(|c| f64::from(actual[1][c]) - f64::from(actual[0][c]));
                let reference_jump: [f64; 3] =
                    std::array::from_fn(|c| expected[1][c] - expected[0][c]);
                let residual: [f64; 3] =
                    std::array::from_fn(|c| (gpu_jump[c] - reference_jump[c]).abs());
                for error in residual {
                    worst = worst.max(error);
                }
                checked_pairs += 1;
                println!(
                    "basis continuity case={case}; mode={mode}; s={s}; NoV=1; F0=white; actual={actual:?}; reference={expected:?}; gpu_jump={gpu_jump:?}; reference_jump={reference_jump:?}; abs_RGB_residual_jump={residual:?}; bound={RESIDUAL_JUMP_BOUND}"
                );
            }
        }
    }
    assert_eq!(checked_pairs, expected_pairs);
    println!(
        "basis continuity checked_pairs={checked_pairs}; max_abs_RGB_residual_jump={worst}; convergence={convergence}; reference_change={reference_change}; residual_bound={RESIDUAL_JUMP_BOUND}; reference_bound={REFERENCE_BOUND}; displacement_bound={DISPLACEMENT_BOUND}"
    );
    assert!(
        worst <= RESIDUAL_JUMP_BOUND,
        "fixed-world basis-switch continuity exceeds frozen residual-jump bound: {worst}"
    );
}

#[test]
fn ambient_diffuse_has_no_extra_specular_gain() {
    let (d, q) = gpu();
    // Black specular source isolates SH9 diffuse through actual generated fs.
    let env = d.create_texture(&wgpu::TextureDescriptor {
        label: Some("black specular source for diffuse isolation"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 72,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let mut worst = 0.0f64;
    for mode in [2, 3] {
        let pixels = render(
            &d,
            &q,
            mode,
            DVec3::Z,
            &env.create_view(&Default::default()),
            true,
            None,
        );
        for (index, actual) in pixels.iter().enumerate() {
            let row = index / (FIELDS * CONTROLS);
            let nov = NOV[row % 3];
            // Preserve the declared workflow's diffuse Fresnel term; only the
            // undocumented shininess-dependent ambient gain is under test.
            let diffuse = if mode == 2 {
                let roughness = (256.0 - SHININESS[row / 3]) / 255.0;
                let fresnel = 0.04 + 0.96 * 2.0f64.powf((-5.55473 * nov - 6.98316) * nov);
                1.0 - fresnel * (1.0 - roughness)
            } else {
                1.0
            };
            let expected = (128.0 / 255.0) * diffuse;
            for value in &actual[..3] {
                assert!(value.is_finite());
                worst = worst.max((*value as f64 - expected).abs());
            }
        }
    }
    println!("ambient diffuse max error={worst}; tolerance=0.001");
    assert!(
        worst < 0.001,
        "hidden roughness/shininess gain dims diffuse ambient"
    );
}

#[test]
fn reflection_true_bp_generated_modes_2_and_3() {
    let (d, q) = gpu();
    let mut worst = 0.0f64;
    let mut zero_worst = 0.0f64;
    let mut failures = 0;
    let mut rows = Vec::new();
    for n in [
        DVec3::Z,
        DVec3::new(0.3, 0.8, -0.52).normalize(),
        DVec3::Y,
        -DVec3::Z,
    ] {
        let env = source(&d, &q, n);
        for mode in [2, 3] {
            let pixels = render(
                &d,
                &q,
                mode,
                n,
                &env.create_view(&Default::default()),
                false,
                None,
            );
            for (index, actual) in pixels.iter().enumerate() {
                assert!(
                    actual.iter().all(|v| v.is_finite()) && actual[3] == 1.,
                    "uncovered/nonfinite pixel {index}: {actual:?}"
                );
                let row = index / (FIELDS * CONTROLS);
                let field = (index / CONTROLS) % FIELDS;
                let control = index % CONTROLS;
                let (_, color) = material(mode, control, SHININESS[row / 3]);
                let expected: [f64; 3] =
                    std::array::from_fn(|c| REFERENCE[row][field] * color[c] * [1., 0.5, 0.25][c]);
                let error = (0..3)
                    .map(|c| (actual[c] as f64 - expected[c]).abs())
                    .fold(0., f64::max);
                worst = worst.max(error);
                if control == 0 {
                    zero_worst = zero_worst.max(
                        actual[..3]
                            .iter()
                            .map(|v| v.abs() as f64)
                            .fold(0., f64::max),
                    );
                }
                if error + REFERENCE_BUDGET > ACCEPTANCE {
                    failures += 1;
                }
                rows.push(serde_json::json!({"normal": n.to_array(), "mode": mode,
                    "shininess": SHININESS[row/3], "NoV": NOV[row%3], "field": field,
                    "control": control, "actual": actual, "expected": expected, "error": error}));
            }
        }
    }
    println!(
        "reflection: pixels={} max RGB error={worst:.10}, plus reference budget={:.10}, total={:.10}, acceptance={ACCEPTANCE}; failing pixels={failures}; zero F0 max={zero_worst}",
        rows.len(),
        REFERENCE_BUDGET,
        worst + REFERENCE_BUDGET
    );
    let worst_row = rows
        .iter()
        .max_by(|a, b| {
            a["error"]
                .as_f64()
                .unwrap()
                .total_cmp(&b["error"].as_f64().unwrap())
        })
        .unwrap();
    println!("reflection worst: {worst_row}");
    if let Ok(path) = std::env::var("EPU_REFLECTION_EVIDENCE") {
        std::fs::write(path, serde_json::to_string_pretty(&serde_json::json!({
            "acceptance": ACCEPTANCE, "reference_budget": REFERENCE_BUDGET,
            "max_error": worst, "zero_F0_max": zero_worst, "failing_pixels": failures, "rows": rows,
        })).unwrap()).unwrap();
    }
    assert!(
        failures == 0 && zero_worst == 0.,
        "production GPU reflection violates frozen bound"
    );
}

// Independent CPU oracle: 1024 polar Gauss-Legendre x 4096 azimuth samples.
// Parent 512 vs 1024 convergence max: 0.0008643796512958282.
// Rows s=1,16,64,256 crossed with NoV=1,.5,.1; columns documented above.
const REFERENCE: [[f64; 6]; 12] = [
    [
        0.3587565960919376,
        0.318484368893869,
        0.002948029216853652,
        0.002948029216853652,
        0.0,
        2.4957827573655006e-05,
    ],
    [
        0.29380289451724306,
        0.2616345786750606,
        0.0014944734969277135,
        0.0025920343895900113,
        0.0,
        0.0,
    ],
    [
        0.22931248316564434,
        0.20385782956555687,
        0.00029288126032248936,
        0.002218636771855019,
        0.0,
        0.0,
    ],
    [
        0.8062103043324733,
        0.6157320210578959,
        0.01724472383963693,
        0.01724472383963693,
        2.6656896608125347e-05,
        0.007427919613370908,
    ],
    [
        0.2797172458976412,
        0.3046987701063782,
        0.00845011749701886,
        0.0017360942710815982,
        0.0,
        4.1710331040254086e-05,
    ],
    [
        0.07782943885264128,
        0.09786942537693977,
        0.0012860854439195625,
        0.00014873995828359993,
        0.0,
        0.0,
    ],
    [
        0.9423294745153471,
        0.6246298320377066,
        0.06074196863560026,
        0.06074196863560026,
        0.00043972918504657456,
        0.02968285296564782,
    ],
    [
        0.2579706411125513,
        0.3436489816224821,
        0.02912067488222974,
        8.193720903929101e-06,
        0.00017013311244139175,
        0.00691186626430657,
    ],
    [
        0.03390142309006003,
        0.05360563121043272,
        0.002901589845629219,
        0.0,
        0.0,
        0.0,
    ],
    [
        0.9840927501800945,
        0.6080750727946813,
        0.21748691168840176,
        0.21748691168840176,
        0.002764384035022028,
        0.11539103664698215,
    ],
    [
        0.2517894357377865,
        0.3653760529034345,
        0.09260377302838066,
        0.0,
        0.0027490836204383706,
        0.037209341856589424,
    ],
    [
        0.01730992875035927,
        0.029863285072190167,
        0.005669159026079832,
        0.0,
        0.00015976212353860248,
        0.0001500844383289062,
    ],
];
