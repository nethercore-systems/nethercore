//! Bounded GPU regressions. Uses the production-assembled shaders, not CPU formula copies.
//! Run explicitly: cargo test -p nethercore-zx --test epu_gpu -- --nocapture
#![allow(dead_code)]
// Indexed channel/case loops and explicit packed fields mirror the shader oracles.
// Keep this test-only representation stable; production modules are linted separately.
#![allow(clippy::needless_range_loop, clippy::too_many_arguments)]
#[path = "epu_gpu/aperture_bars_multi.rs"]
mod aperture_bars_multi;
#[path = "epu_gpu/aperture_plane.rs"]
mod aperture_plane;
#[path = "epu_gpu/aperture_simple_shapes.rs"]
mod aperture_simple_shapes;
#[path = "../../examples/3-inspectors/epu-showcase/src/presets/set_17_18.rs"]
mod arcade_preset;
#[path = "epu_gpu/atmosphere_ramp.rs"]
mod atmosphere_ramp;
#[path = "epu_gpu/band.rs"]
mod band;
#[path = "epu_gpu/celestial.rs"]
mod celestial;
#[path = "epu_gpu/celestial_angles.rs"]
mod celestial_angles;
#[path = "epu_gpu/celestial_phase.rs"]
mod celestial_phase;
#[path = "epu_gpu/celestial_ring.rs"]
mod celestial_ring;
#[path = "epu_gpu/celestial_uv.rs"]
mod celestial_uv;
#[path = "epu_gpu/cell_candidate.rs"]
mod cell_candidate;
#[path = "epu_gpu/cell_continuity.rs"]
mod cell_continuity;
#[path = "epu_gpu/cell_grid_zero.rs"]
mod cell_grid_zero;
#[path = "epu_gpu/cell_hex_2d_probe.rs"]
mod cell_hex_2d_probe;
#[path = "epu_gpu/cell_offset_fractional.rs"]
mod cell_offset_fractional;
#[path = "epu_gpu/cell_radial.rs"]
mod cell_radial;
#[path = "epu_gpu/cell_sites.rs"]
mod cell_sites;
#[path = "epu_gpu/cell_uniform.rs"]
mod cell_uniform;
#[path = "epu_gpu/cell_warped_radial.rs"]
mod cell_warped_radial;
// Import the editable example verbatim, including its local tests.
#[allow(clippy::items_after_test_module)]
#[path = "../../examples/3-inspectors/epu-showcase/src/constants.rs"]
mod constants;
#[path = "epu_gpu/cost.rs"]
mod cost;
#[path = "epu_gpu/decal_facing.rs"]
mod decal_facing;
#[path = "epu_gpu/decal_paint.rs"]
mod decal_paint;
#[path = "epu_gpu/direction_decode.rs"]
mod direction_decode;
#[path = "epu_gpu/domain_boundary.rs"]
mod domain_boundary;
#[path = "epu_gpu/flow_streaks.rs"]
mod flow_streaks;
#[path = "../build_support/formats.rs"]
mod formats;
#[path = "epu_gpu/grid.rs"]
mod grid_regular;
#[path = "../../examples/3-inspectors/epu-showcase/src/presets/set_05_06.rs"]
mod grove_preset;
#[path = "epu_gpu/imported_orientation.rs"]
mod imported_orientation;
#[path = "epu_gpu/irradiance.rs"]
mod irradiance;
#[path = "epu_gpu/lighting.rs"]
mod lighting;
#[path = "epu_gpu/lobe_centre.rs"]
mod lobe_centre;
#[path = "epu_gpu/patches_poles.rs"]
mod patches_poles;
#[path = "epu_gpu/portal.rs"]
mod portal;
#[path = "../src/graphics/vertex/attributes.rs"]
mod production_attributes;
#[path = "../src/graphics/pipeline/bind_groups.rs"]
mod production_bind_groups;
#[path = "epu_gpu/reflection.rs"]
mod reflection;
#[path = "epu_gpu/scatter.rs"]
mod scatter;
#[path = "epu_gpu/scatter_phased.rs"]
mod scatter_phased;
#[path = "../build_support/generator.rs"]
mod shader_generator;
#[path = "../src/graphics/epu/shaders.rs"]
mod shaders;
#[path = "epu_gpu/shared_noise.rs"]
mod shared_noise;
#[path = "epu_gpu/showcase_seam.rs"]
mod showcase_seam;
#[path = "../build_support/snippets.rs"]
mod snippets;
#[path = "../build_support/sources.rs"]
mod sources;
#[path = "epu_gpu/structural_bounds.rs"]
mod structural_bounds;
#[path = "epu_gpu/trace_direct3d.rs"]
mod trace_direct3d;
#[path = "epu_gpu/trace_wrap.rs"]
mod trace_wrap;
#[path = "epu_gpu/veil_polar.rs"]
mod veil_polar;
use shaders::*;
use wgpu::util::DeviceExt;

// Run a small production-WGSL probe with only a readback image binding.
fn probe_image(body: &str, width: u32, height: u32) -> Vec<[f32; 4]> {
    probe_image_bounds(body, EPU_BOUNDS, width, height)
}

fn probe_image_bounds(body: &str, bounds: &str, width: u32, height: u32) -> Vec<[f32; 4]> {
    probe_image_input_bounds(body, bounds, width, height, &[])
}

// Keep instructions runtime-fed when qualifying input-flow-sensitive shaders.
fn probe_image_input_bounds(
    body: &str,
    bounds: &str,
    width: u32,
    height: u32,
    inputs: &[u32],
) -> Vec<[f32; 4]> {
    let (d, q) = gpu();
    let p = pipeline(
        &d,
        format!("{EPU_COMMON}\n{bounds}\n{EPU_FEATURES}\n{body}"),
        "probe",
    );
    let t = texture(
        &d,
        width,
        height,
        wgpu::TextureFormat::Rgba16Float,
        wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
    );
    let view = t.create_view(&Default::default());
    let input = (!inputs.is_empty()).then(|| {
        d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(inputs),
            usage: wgpu::BufferUsages::STORAGE,
        })
    });
    let mut entries = vec![wgpu::BindGroupEntry {
        binding: 0,
        resource: wgpu::BindingResource::TextureView(&view),
    }];
    if let Some(input) = &input {
        entries.push(wgpu::BindGroupEntry {
            binding: 1,
            resource: input.as_entire_binding(),
        });
    }
    let group = d.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &p.get_bind_group_layout(0),
        entries: &entries,
    });
    let mut e = d.create_command_encoder(&Default::default());
    {
        let mut pass = e.begin_compute_pass(&Default::default());
        pass.set_pipeline(&p);
        pass.set_bind_group(0, &group, &[]);
        pass.dispatch_workgroups(width, height, 1);
    }
    q.submit([e.finish()]);
    read(&d, &q, &t)
}

#[test]
fn region_mask_zero_weight_selection_is_neutral() {
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let w = f32(p.x) / 4.0;
    let regions = RegionWeights(w, 0.0, 1.0-w);
    textureStore(result, vec2i(p.xy), vec4f(
        region_weight(regions, REGION_SKY),
        region_weight(regions, REGION_SKY | REGION_WALLS), w, 1.0));
}
"#,
        5,
        1,
    );
    for pixel in pixels {
        assert!(pixel.iter().all(|v| v.is_finite()));
        assert!(
            (pixel[0] - pixel[1]).abs() < 0.001,
            "adding a zero-weight region changes strength: {pixel:?}"
        );
    }
}

// Retained regression for the accepted literal wall-color / independent paint-opacity repair.
#[test]
fn ramp_authored_wall_color_and_paint_opacity() {
    // Absolute budget fixed before execution: f32 evaluation + rgba16float readback.
    const TOL: f32 = 0.001;
    let colors = [
        [0u8, 0, 0],
        [255, 0, 0],
        [0, 255, 0],
        [0, 0, 255],
        [51, 153, 255],
        [255, 255, 255],
    ];
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let colors = array<u32,6>(0u, 0xff0000u, 0x00ff00u, 0x0000ffu, 0x3399ffu, 0xffffffu);
    let alphas = array<u32,3>(0u, 7u, 15u);
    let alpha = alphas[p.x / 6u];
    // Fixed nonblack sky/floor, thresholds +/-0.6, minimum softness, LERP.
    let instr = vec4u((0xc3u << 24u) | (0x80ffu << 8u) | (alpha << 4u) | 15u,
        colors[p.x % 6u], 0x80402010u, (1u << 27u) | (7u << 24u) | (3u << 21u) | 0x2040u);
    let up = decode_dir16(instr_dir16(instr));
    let dir = normalize(cross(up, vec3f(0.,0.,1.)));
    let value = evaluate_bounds_layer(dir, instr, OP_RAMP, up, RegionWeights(1.,0.,0.));
    var layers: array<vec4u,8>;
    layers[0] = instr;
    var output = vec4f(value.sample.rgb, value.sample.w);
    if p.y == 1u { output = vec4f(value.regions.sky, value.regions.wall, value.regions.floor, value.region_mix); }
    if p.y == 2u { output = vec4f(evaluate_epu_layers(dir, layers), 1.); }
    if p.y == 3u { output = vec4f(f32(instr_a(instr))/255., f32(instr_b(instr))/255., f32(instr_c(instr))/255., instr_alpha_a_f32(instr)); }
    textureStore(result, p.xy, output);
}
"#,
        18,
        4,
    );
    assert_eq!(pixels.len(), 72);
    assert!(
        pixels.iter().flatten().all(|v| v.is_finite()),
        "fixture: nonfinite GPU readback"
    );
    let mut failures = Vec::new();
    for case in 0..18 {
        let alpha = [0., 7. / 15., 1.][case / 6];
        let expected = colors[case % 6].map(|c| f32::from(c) / 255.);
        let raw = pixels[case];
        let regions = pixels[18 + case];
        let composed = pixels[36 + case];
        let decoded = pixels[54 + case];
        assert_eq!(
            regions,
            [0., 1., 0., 1.],
            "full wall ownership independent of paint"
        );
        assert!((raw[3] - alpha).abs() <= TOL && (decoded[3] - alpha).abs() <= TOL);
        assert_eq!(composed[3], 1., "fixture: output coverage");
        for c in 0..3 {
            assert!(
                (decoded[c] - expected[c]).abs() <= TOL,
                "fixture: packed RGB mismatch"
            );
            assert!(
                (composed[c] - raw[c] * alpha).abs() <= TOL,
                "shared evaluator must honor paint opacity"
            );
            if (raw[c] - expected[c]).abs() > TOL || (composed[c] - expected[c] * alpha).abs() > TOL
            {
                failures.push((case, c, raw[c], composed[c], expected[c], alpha));
            }
        }
        println!(
            "RAMP case={case} expected={expected:?} alpha={alpha} raw={raw:?} composed={composed:?} regions={regions:?}"
        );
    }
    println!(
        "RAMP packing, finite coverage, full ownership and paint-opacity controls PASS; tolerance={TOL}"
    );
    assert!(
        failures.is_empty(),
        "authored wall RGB suppressed (case, channel, raw, composed, authored, alpha): {failures:?}"
    );
}

#[test]
fn rgb_offset_legacy_blend() {
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let dst = vec3f(0.25, 0.5, 0.75);
    var src = vec3f(0.5);
    var alpha = 1.0;
    switch p.x {
        case 1u: { src = vec3f(0.25, 0.75, 0.625); alpha = 0.5; }
        case 2u: { src = vec3f(0.0, 1.0, 0.5); }
        case 3u: { src = vec3f(1.0, 0.0, 0.0); alpha = 0.0; }
        default: {}
    }
    // Decode the unchanged wire value, then run the actual shared blend math.
    let blend = instr_blend(vec4u(0u, 0u, 0u, 5u << 21u));
    textureStore(result, p.xy, vec4f(apply_blend(dst, LayerSample(src, alpha), blend), 1.0));
}
"#,
        4,
        1,
    );
    let expected = [
        [0.25, 0.5, 0.75, 1.0],  // Source 0.5 identity.
        [0.0, 0.75, 0.875, 1.0], // Signed component offset at half alpha.
        [0.0, 1.0, 0.75, 1.0],   // Low/high clamp and neutral component.
        [0.25, 0.5, 0.75, 1.0],  // Alpha zero identity for in-range dst.
    ];
    assert_eq!(pixels.len(), expected.len());
    for (case, (actual, expected)) in pixels.iter().zip(expected).enumerate() {
        for c in 0..4 {
            // Binary fractions are exact in rgba16float; no broad image tolerance.
            assert!(
                actual[c].is_finite() && (actual[c] - expected[c]).abs() <= 0.000001,
                "RGB Offset case {case} channel {c}: {actual:?} != {expected:?}"
            );
        }
    }
    println!("RGB Offset legacy value 5: {pixels:?}");
}

#[test]
fn patches_angular_seam() {
    // At paired directions separated by 2e-6 radians, smooth PATCHES must agree.
    // Sweep both angular domains, smooth variants, elevations, scales and seeds.
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let domain = 1u + p.y / 80u;
    let variant = (p.y / 16u) % 5u;
    // Test STREAKS (5), not intentionally discontinuous STATIC (4).
    let v = select(variant, 5u, variant == 4u);
    let seed = p.y % 16u;
    let axis_bits = 0xff80u;
    let axis = decode_dir16(axis_bits);
    let ref_axis = select(vec3f(0.,1.,0.), vec3f(1.,0.,0.), abs(axis.y) > 0.9);
    let t = normalize(cross(ref_axis, axis)); let b = normalize(cross(axis, t));
    let h = mix(-0.8, 0.8, f32(seed) / 15.0);
    let phi = select(-PI + 0.000001, PI - 0.000001, p.x == 1u);
    let dir = normalize(axis * h + sqrt(1.0-h*h) * (t*cos(phi)+b*sin(phi)));
    let instr = vec4u((seed << 24u) | (axis_bits << 8u) | 255u,
        ((17u + seed*13u) << 16u) | (128u << 8u) | 64u,
        0xff000000u, (6u << 27u) | (7u << 24u) | (((domain << 3u) | v) << 16u) | 0xffffu);
    let value = eval_patches(dir, instr, RegionWeights(1.,0.,0.));
    textureStore(result, p.xy, vec4f(value.regions.sky, value.regions.wall, value.regions.floor, 1.));
}
"#,
        2,
        160,
    );
    let mut max_error = 0.0f32;
    let min_sky = pixels.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min);
    let max_sky = pixels
        .iter()
        .map(|p| p[0])
        .fold(f32::NEG_INFINITY, f32::max);
    assert!(
        max_sky - min_sky > 0.1,
        "constant output cannot satisfy seam coverage"
    );
    for (row, pair) in pixels.as_chunks::<2>().0.iter().enumerate() {
        if (pair[0][0] - pair[1][0])
            .abs()
            .max((pair[0][1] - pair[1][1]).abs())
            > 0.005
        {
            println!("seam row={row} pair={pair:?}");
        }
        for c in 0..3 {
            assert!(pair[0][c].is_finite() && pair[1][c].is_finite());
            max_error = max_error.max((pair[0][c] - pair[1][c]).abs());
        }
    }
    println!("PATCHES angular seam max region difference = {max_error}");
    assert!(
        max_error < 0.005,
        "smooth angular domains must wrap: {max_error}"
    );
}

#[test]
fn patches_lattice_faces_and_corners() {
    // Finite extension of the original one-Z-face regression. Actual GPU hash,
    // all smooth noise families; STATIC deliberately has a floor/hash contract.
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let n = f32(i32(p.y / 4u) - 16);
    let kind = p.y % 4u;
    let eps = select(-0.000001, 0.000001, p.x == 1u);
    var coord = vec3f(n + eps, 0.32, 0.2);
    if kind == 1u { coord = coord.yxz; }
    if kind == 2u { coord = coord.yzx; }
    if kind == 3u { coord = vec3f(n + eps); }
    textureStore(result,p.xy,vec4f(patches_value_noise3(coord),
        patches_fbm_blobs(coord,3u),patches_fbm_islands(coord,3u),patches_fbm_debris(coord,3u)));
}
"#,
        2,
        132,
    );
    let mut error = 0.0f32;
    let mut lo = [f32::INFINITY; 4];
    let mut hi = [f32::NEG_INFINITY; 4];
    for (row, pair) in pixels.as_chunks::<2>().0.iter().enumerate() {
        for c in 0..4 {
            assert!(pair[0][c].is_finite() && pair[1][c].is_finite());
            if (pair[0][c] - pair[1][c]).abs() >= 0.001 {
                println!(
                    "PATCHES RED row={row} integer={} kind={} channel={c} pair={pair:?}",
                    row as i32 / 4 - 16,
                    row % 4
                );
            }
            error = error.max((pair[0][c] - pair[1][c]).abs());
            lo[c] = lo[c].min(pair[0][c]);
            hi[c] = hi[c].max(pair[0][c]);
        }
    }
    println!(
        "PATCHES 132 lattice faces/corners max_jump={error} ranges={lo:?}..{hi:?} tolerance=.001"
    );
    assert!(error < 0.001);
    for c in 0..4 {
        assert!(hi[c] - lo[c] > 0.1, "constant family {c}");
    }
}

#[test]
fn patches_noise_is_continuous_across_lattice_boundary() {
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let eps = select(-0.000001, 0.000001, p.x==1u);
 let coord = vec3f(0.32,0.2,eps);
 textureStore(result,p.xy,vec4f(patches_value_noise3(coord),patches_fbm_blobs(coord,3u),eps,1.));
}
"#,
        2,
        1,
    );
    println!("noise boundary diagnostic: {pixels:?}");
    for channel in 0..2 {
        assert!(
            (pixels[0][channel] - pixels[1][channel]).abs() < 0.001,
            "smooth noise must agree across a lattice boundary: {pixels:?}"
        );
    }
}

fn gpu() -> (wgpu::Device, wgpu::Queue) {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::default().with_env(),
        ..Default::default()
    });
    // Explicit portability runs must not silently fall back to another adapter.
    let adapter = if std::env::var_os("WGPU_ADAPTER_NAME").is_some() {
        assert!(
            !std::env::var("WGPU_ADAPTER_NAME")
                .unwrap()
                .trim()
                .is_empty()
        );
        wgpu::util::initialize_adapter_from_env(&instance, None)
            .expect("requested GPU adapter required")
    } else {
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        }))
        .expect("GPU required: do not silently skip behavioral regression")
    };
    println!("GPU {:?}", adapter.get_info());
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: wgpu::Features::TEXTURE_COMPRESSION_BC,
        ..Default::default()
    }))
    .unwrap()
}
fn buffer(d: &wgpu::Device, words: &[u32], usage: wgpu::BufferUsages) -> wgpu::Buffer {
    d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(words),
        usage,
    })
}
fn texture(
    d: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
) -> wgpu::Texture {
    d.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}
fn read(d: &wgpu::Device, q: &wgpu::Queue, t: &wgpu::Texture) -> Vec<[f32; 4]> {
    let stride = (t.width() * 8).div_ceil(256) * 256;
    let b = d.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (stride * t.height()) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut e = d.create_command_encoder(&Default::default());
    e.copy_texture_to_buffer(
        t.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &b,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(stride),
                rows_per_image: None,
            },
        },
        t.size(),
    );
    q.submit([e.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    b.slice(..)
        .map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
    d.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    rx.recv().unwrap().unwrap();
    let bytes = b.slice(..).get_mapped_range();
    let mut out = Vec::new();
    for y in 0..t.height() {
        for x in 0..t.width() {
            let offset = (y * stride + x * 8) as usize;
            out.push(std::array::from_fn(|c| {
                half::f16::from_bits(u16::from_le_bytes([
                    bytes[offset + c * 2],
                    bytes[offset + c * 2 + 1],
                ]))
                .to_f32()
            }));
        }
    }
    out
}
fn pipeline(d: &wgpu::Device, src: String, entry: &str) -> wgpu::ComputePipeline {
    let module = d.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(entry),
        source: wgpu::ShaderSource::Wgsl(src.into()),
    });
    d.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(entry),
        layout: None,
        module: &module,
        entry_point: Some(entry),
        compilation_options: Default::default(),
        cache: None,
    })
}
// Packed ABI words: lo low/high followed by hi low/high, as in EpuConfig.
fn instr(op: u32, mask: u32, dir: u32, paint: u32, rgb: u32) -> [u32; 4] {
    [
        dir << 8 | paint << 4,
        128 << 24 | 20 << 16,
        rgb << 24 | rgb,
        op << 27 | mask << 24 | rgb >> 8,
    ]
}
#[test]
fn first_and_sequential_bounds_match_cache() {
    let (d, q) = gpu();
    const N: u32 = 32;
    let cache = pipeline(
        &d,
        format!("{EPU_COMMON}\n{EPU_BOUNDS}\n{EPU_FEATURES}\n{EPU_COMPUTE_ENV}"),
        "epu_build",
    );
    // Execute the direct entry point at four subtexel directions. CPU area
    // averaging below compares equal footprints; no evaluator is copied.
    let direct = pipeline(
        &d,
        format!(
            "{}\n{}",
            sources::COMMON
                .split("// Unified Vertex Input/Output")
                .next()
                .unwrap(),
            r#"
@group(0) @binding(14) var result: texture_storage_2d_array<rgba16float, write>;
@compute @workgroup_size(8,8) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let dir=octahedral_decode((vec2f(p.xy)+0.5)/64.0*2.0-1.0);
 textureStore(result,p.xy,0,vec4f(epu_eval_hi(0u,dir),1.0));
}"#
        ),
        "probe",
    );
    let active = buffer(&d, &[0], wgpu::BufferUsages::STORAGE);
    let frame = buffer(&d, &[1, N, 0, 0], wgpu::BufferUsages::UNIFORM);
    let mut failures = Vec::new();
    for sequential in [false, true] {
        let mut words = [[0u32; 4]; 8];
        words[0] = instr(4, 7, 0xff80, 0, 0); // SPLIT/HALF, zero paint still owns regions.
        if sequential {
            words[1] = instr(4, 7, 0x0080, 0, 0);
        }
        words[3] = instr(18, 4, 0x0080, 15, 0xffffff); // broad sky-masked LOBE aimed down
        words[3][1] = 128 << 24; // broad exponent, no waveform
        let state = buffer(
            &d,
            bytemuck::cast_slice(&words),
            wgpu::BufferUsages::STORAGE,
        );
        let mut outputs = Vec::new();
        for (p, is_cache) in [(&cache, true), (&direct, false)] {
            let size = if is_cache { N } else { N * 2 };
            let t = texture(
                &d,
                size,
                size,
                wgpu::TextureFormat::Rgba16Float,
                wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
            );
            let view = t.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                ..Default::default()
            });
            let entries = if is_cache {
                vec![
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: state.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: active.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: frame.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                ]
            } else {
                vec![
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: state.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 14,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                ]
            };
            let group = d.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &p.get_bind_group_layout(0),
                entries: &entries,
            });
            let mut e = d.create_command_encoder(&Default::default());
            {
                let mut pass = e.begin_compute_pass(&Default::default());
                pass.set_pipeline(p);
                pass.set_bind_group(0, &group, &[]);
                pass.dispatch_workgroups(size / 8, size / 8, 1);
            }
            q.submit([e.finish()]);
            outputs.push(read(&d, &q, &t));
        }
        // Compare angular-cell averages, including folds/corners previously excluded.
        // The unchanged tolerance covers half-float conversion, not region errors.
        let mut max_error = 0f32;
        let mut direct_peak = 0f32;
        let mut cache_peak = 0f32;
        for y in 0..N {
            for x in 0..N {
                let mut sum = 0.0f64;
                let mut weights = 0.0f64;
                for dy in 0..2 {
                    for dx in 0..2 {
                        let sx = x * 2 + dx;
                        let sy = y * 2 + dy;
                        let ox = (sx as f64 + 0.5) / (N * 2) as f64 * 2.0 - 1.0;
                        let oy = (sy as f64 + 0.5) / (N * 2) as f64 * 2.0 - 1.0;
                        let mut v = glam::DVec3::new(ox, oy, 1.0 - ox.abs() - oy.abs());
                        if v.z < 0.0 {
                            v.x = (1.0 - oy.abs()) * ox.signum();
                            v.y = (1.0 - ox.abs()) * oy.signum();
                        }
                        let weight = 1.0 / v.length().powi(3);
                        sum += outputs[1][(sy * N * 2 + sx) as usize][0] as f64 * weight;
                        weights += weight;
                    }
                }
                let expected = (sum / weights) as f32;
                let i = (y * N + x) as usize;
                max_error = max_error.max((outputs[0][i][0] - expected).abs());
                direct_peak = direct_peak.max(expected);
                cache_peak = cache_peak.max(outputs[0][i][0]);
            }
        }
        println!(
            "sequential={sequential} cache/direct max_error={max_error} direct_peak={direct_peak} cache_peak={cache_peak}"
        );
        if max_error > 0.002 {
            failures.push((sequential, max_error));
        }
        if sequential {
            assert!(direct_peak > 0.2, "second source must own down-facing sky");
            assert!(cache_peak > 0.2, "cache must preserve second-source sky");
        } else {
            assert!(cache_peak < 0.002, "cache must not leak bootstrap sky");
            assert!(
                direct_peak < 0.002,
                "zero paint must not leak bootstrap sky"
            );
        }
    }
    assert!(
        failures.is_empty(),
        "cache/direct region disagreement: {failures:?}"
    );
}

#[test]
fn bc5_normal_sampling_and_skip() {
    let (d, q) = gpu();
    let module = d.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("production normal helper"),
        source: wgpu::ShaderSource::Wgsl(
            format!(
                "{}\n{}",
                sources::COMMON
                    .split("// Unified Vertex Input/Output")
                    .next()
                    .unwrap(),
                r#"
@vertex fn vs(@builtin(vertex_index) i:u32)->@builtin(position) vec4f {
 let p=array<vec2f,3>(vec2f(-1.,-1.),vec2f(3.,-1.),vec2f(-1.,3.));return vec4f(p[i],0.,1.);
}
@fragment fn fs(@builtin(position) p:vec4f)->@location(0) vec4f {
 let flags=select(0u,FLAG_SKIP_NORMAL_MAP,p.x>=1.0);
 let n=sample_normal_map(slot3,vec2f(0.5),build_tbn(vec3f(1.,0.,0.),vec3f(0.,0.,1.),1.),flags);
 return vec4f(n,1.);
}"#
            )
            .into(),
        ),
    });
    // Explicit bindings keep this executable even while the RED shader ignores its texture.
    let empty = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[],
    });
    let layout = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let pl = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&empty, &layout],
        push_constant_ranges: &[],
    });
    let p = d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pl),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs"),
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
    let normal = texture(
        &d,
        4,
        4,
        wgpu::TextureFormat::Bc5RgUnorm,
        wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    );
    // One BC5 block, constant R=204/255 and G=128/255, index zero for every texel.
    q.write_texture(
        normal.as_image_copy(),
        &[204, 204, 0, 0, 0, 0, 0, 0, 128, 128, 0, 0, 0, 0, 0, 0],
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(16),
            rows_per_image: Some(4),
        },
        normal.size(),
    );
    let nv = normal.create_view(&Default::default());
    let sampler = d.create_sampler(&Default::default());
    let bg = d.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&nv),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    let eg = d.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &empty,
        entries: &[],
    });
    let t = texture(
        &d,
        2,
        1,
        wgpu::TextureFormat::Rgba16Float,
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    );
    let tv = t.create_view(&Default::default());
    let mut e = d.create_command_encoder(&Default::default());
    {
        let mut pass = e.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &tv,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&p);
        pass.set_bind_group(0, &eg, &[]);
        pass.set_bind_group(1, &bg, &[]);
        pass.draw(0..3, 0..1);
    }
    q.submit([e.finish()]);
    let pixels = read(&d, &q, &t);
    println!("BC5 mapped/skip = {pixels:?}");
    assert!(
        (pixels[0][0] - 0.6).abs() < 0.01 && (pixels[0][2] - 0.8).abs() < 0.01,
        "enabled map must perturb normal: {pixels:?}"
    );
    assert_eq!(
        pixels[1],
        [0., 0., 1., 1.],
        "explicit skip must preserve base normal"
    );
}

fn push_f16(out: &mut Vec<u8>, value: f32) {
    out.extend_from_slice(&half::f16::from_f32(value).to_bits().to_le_bytes());
}

fn generated_material_source(mode: u8) -> String {
    assert!(matches!(mode, 2 | 3));
    shader_generator::generate_shader(mode, 21).expect("mode 2/3 tangent+UV shader")
}

fn material_vertex_bytes(tangent: u32) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(3 * 20);
    for [x, y, z] in [[-1.0, -1.0, 0.0], [3.0, -1.0, 0.0], [-1.0, 3.0, 0.0]] {
        push_f16(&mut bytes, x);
        push_f16(&mut bytes, y);
        push_f16(&mut bytes, z);
        push_f16(&mut bytes, 1.0);
        bytes.extend_from_slice(&0x8000u16.to_le_bytes());
        bytes.extend_from_slice(&0x8000u16.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes()); // +Z normal
        bytes.extend_from_slice(&tangent.to_le_bytes());
    }
    bytes
}

fn bc5_constant_block(r: u8, g: u8) -> [u8; 16] {
    [r, r, 0, 0, 0, 0, 0, 0, g, g, 0, 0, 0, 0, 0, 0]
}

fn render_generated_material(
    d: &wgpu::Device,
    q: &wgpu::Queue,
    mode: u8,
    normal_rg: [u8; 2],
    skip_normal_map: bool,
) -> [f32; 4] {
    render_generated_material_tangent(d, q, mode, normal_rg, skip_normal_map, 0x00007fff)
}

fn render_generated_material_tangent(
    d: &wgpu::Device,
    q: &wgpu::Queue,
    mode: u8,
    normal_rg: [u8; 2],
    skip_normal_map: bool,
    tangent: u32,
) -> [f32; 4] {
    let frame_layout = production_bind_groups::create_frame_bind_group_layout(d, mode);
    let texture_layout = production_bind_groups::create_texture_bind_group_layout(d);
    let pipeline_layout = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("generated material test layout"),
        bind_group_layouts: &[&frame_layout, &texture_layout],
        push_constant_ranges: &[],
    });
    let module = d.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("generated material mode 2/3"),
        source: wgpu::ShaderSource::Wgsl(generated_material_source(mode).into()),
    });
    let pipeline = d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("generated material mode 2/3"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs"),
            compilation_options: Default::default(),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: 20,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: production_attributes::VERTEX_ATTRIBUTES[21],
            }],
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

    let vertex = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("generated material vertices"),
        contents: &material_vertex_bytes(tangent),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let mut transforms = Vec::<f32>::new();
    transforms.extend_from_slice(&[
        1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
    ]);
    transforms.extend_from_slice(&[
        1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., -2., 1.,
    ]);
    // View-space z=-2 must lie inside WebGPU's [0,w] clip-depth range.
    // Orthographic projection maps it to z=0.5, while preserving XY coverage.
    transforms.extend_from_slice(&[
        1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 0.25, 0., 0., 0., 1., 1.,
    ]);
    let transforms = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("generated material transforms"),
        contents: bytemuck::cast_slice(&transforms),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let indices = buffer(d, &[0, 1, 2, 0], wgpu::BufferUsages::STORAGE);

    let mut shading_words = [0u32; 20];
    shading_words[0] = 0xffff_ffff; // white material, alpha 1
    shading_words[3] = 2 | 4 | 8 | 16 | 32 | (15 << 8) | if skip_normal_map { 0x10000 } else { 0 };
    // Direct lights remain disabled: isolate the actual material SH ambient path.
    let shading = buffer(d, &shading_words, wgpu::BufferUsages::STORAGE);
    let animation = buffer(d, &[0; 12], wgpu::BufferUsages::STORAGE);
    let quads = buffer(d, &[0; 4], wgpu::BufferUsages::STORAGE);
    let epu_states = buffer(d, &[0; 32], wgpu::BufferUsages::STORAGE);
    let epu_frame = buffer(d, &[0; 4], wgpu::BufferUsages::UNIFORM);
    let source_kinds = buffer(d, &[0], wgpu::BufferUsages::STORAGE);
    let mut sh9_words = [0u32; 36];
    sh9_words[0..2].fill(2.0f32.to_bits()); // positive offset keeps signed X/Y unclamped
    sh9_words[5] = 1.0f32.to_bits(); // green: Y
    sh9_words[12] = 1.0f32.to_bits(); // red: X
    sh9_words[10] = 1.0f32.to_bits(); // blue: Z
    let sh9 = buffer(d, &sh9_words, wgpu::BufferUsages::STORAGE);
    let face_bases = buffer(d, &[0xffff_ffff], wgpu::BufferUsages::STORAGE);

    let env = texture(
        d,
        1,
        1,
        wgpu::TextureFormat::Rgba16Float,
        wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    );
    let env_faces = texture(
        d,
        1,
        1,
        wgpu::TextureFormat::Rgba16Float,
        wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    );
    let env_view = env.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        ..Default::default()
    });
    let env_faces_view = env_faces.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        ..Default::default()
    });
    let env_sampler = d.create_sampler(&wgpu::SamplerDescriptor::default());
    let frame_group = d.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("generated material frame group"),
        layout: &frame_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: transforms.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: indices.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: shading.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: animation.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: quads.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: wgpu::BindingResource::TextureView(&env_view),
            },
            wgpu::BindGroupEntry {
                binding: 7,
                resource: wgpu::BindingResource::Sampler(&env_sampler),
            },
            wgpu::BindGroupEntry {
                binding: 8,
                resource: epu_states.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 9,
                resource: epu_frame.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 10,
                resource: source_kinds.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 11,
                resource: sh9.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 12,
                resource: wgpu::BindingResource::TextureView(&env_faces_view),
            },
            wgpu::BindGroupEntry {
                binding: 13,
                resource: face_bases.as_entire_binding(),
            },
        ],
    });

    let white = texture(
        d,
        1,
        1,
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    );
    q.write_texture(
        white.as_image_copy(),
        &[255, 255, 255, 255],
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4),
            rows_per_image: Some(1),
        },
        white.size(),
    );
    let normal = texture(
        d,
        4,
        4,
        wgpu::TextureFormat::Bc5RgUnorm,
        wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    );
    q.write_texture(
        normal.as_image_copy(),
        &bc5_constant_block(normal_rg[0], normal_rg[1]),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(16),
            rows_per_image: Some(4),
        },
        normal.size(),
    );
    let white_view = white.create_view(&Default::default());
    let normal_view = normal.create_view(&Default::default());
    let sampler = d.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let texture_group = d.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("generated material texture group"),
        layout: &texture_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&white_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&white_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&white_view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&normal_view),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    let target = texture(
        d,
        4,
        4,
        wgpu::TextureFormat::Rgba16Float,
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    );
    let target_view = target.create_view(&Default::default());
    let mut encoder = d.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("generated material render"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &frame_group, &[]);
        pass.set_bind_group(1, &texture_group, &[]);
        pass.set_vertex_buffer(0, vertex.slice(..));
        pass.draw(0..3, 0..1);
    }
    q.submit([encoder.finish()]);
    let pixels = read(d, q, &target);
    assert!(
        pixels
            .iter()
            .all(|p| p[3] == 1.0 && p.iter().all(|v| v.is_finite())),
        "generated material must cover the target with finite opaque pixels"
    );
    pixels[5] // Fixed pixel in every variant; never select a different brightest point.
}

#[test]
fn generated_material_normal_mapping_modes_2_and_3() {
    let (d, q) = gpu();
    for mode in [2u8, 3u8] {
        let mapped = render_generated_material(&d, &q, mode, [204, 128], false);
        let neutral = render_generated_material(&d, &q, mode, [128, 128], false);
        let skipped = render_generated_material(&d, &q, mode, [204, 128], true);
        println!(
            "generated material mode={mode} mapped={mapped:?} neutral={neutral:?} skip={skipped:?}"
        );
        for sample in [mapped, neutral, skipped] {
            assert!(
                sample.iter().all(|value| value.is_finite()),
                "non-finite material output: {sample:?}"
            );
        }
        assert!(skipped[2] > 0.01, "ambient must be nonzero in mode {mode}");
        // Independent BC5 UNORM decode; fixed SH fields expose axis/sign swaps.
        // Same-sample RGB ratios cancel the normal-dependent diffuse Fresnel gain.
        // RGBA16F error budget: 0.005.
        for rg in [[204u8, 128u8], [51, 128], [128, 204], [128, 51]] {
            let sample = render_generated_material(&d, &q, mode, rg, false);
            let x = f32::from(rg[0]) / 255.0 * 2.0 - 1.0;
            let y = f32::from(rg[1]) / 255.0 * 2.0 - 1.0;
            let expected = [x, y, (1.0 - x * x - y * y).sqrt()];
            let observed = [
                sample[0] / sample[2] * expected[2] - 2.0 * 0.282095 / 0.488603,
                sample[1] / sample[2] * expected[2] - 2.0 * 0.282095 / 0.488603,
                sample[2] / skipped[2],
            ];
            for axis in 0..3 {
                assert!(
                    (observed[axis] - expected[axis]).abs() < 0.005,
                    "mode={mode} rg={rg:?} axis={axis} observed={observed:?} expected={expected:?}"
                );
            }
            println!(
                "signed normal mode={mode} rg={rg:?} observed={observed:?} expected={expected:?}"
            );
        }
        // Rotated tangent and both handedness signs: T=+Y, B=-X*handedness.
        let rg = [204u8, 204u8];
        let x = f32::from(rg[0]) / 255.0 * 2.0 - 1.0;
        let y = f32::from(rg[1]) / 255.0 * 2.0 - 1.0;
        let z = (1.0 - x * x - y * y).sqrt();
        let offset = 2.0 * 0.282095 / 0.488603;
        for handedness in [1.0f32, -1.0] {
            let tangent = zx_common::pack_tangent([0.0, 1.0, 0.0], handedness);
            let sample = render_generated_material_tangent(&d, &q, mode, rg, false, tangent);
            let observed = [
                sample[0] / sample[2] * z - offset,
                sample[1] / sample[2] * z - offset,
            ];
            let expected = [-handedness * y, x];
            for axis in 0..2 {
                assert!(
                    (observed[axis] - expected[axis]).abs() < 0.005,
                    "rotated TBN mode={mode} hand={handedness} observed={observed:?} expected={expected:?}"
                );
            }
        }
        // SH is proportional to N.z here; the tilted map has z approximately 0.8.
        let ratio = mapped[2] / skipped[2];
        assert!(
            (0.7..0.9).contains(&ratio),
            "tilted SH response must decrease: mode={mode} ratio={ratio}"
        );
        assert!(
            (neutral[2] / skipped[2] - 1.0).abs() < 0.005,
            "neutral map must match explicit skip in mode {mode}: {neutral:?} vs {skipped:?}"
        );
    }
}

#[path = "epu_gpu/cell_hex_2d_medium.rs"]
mod cell_hex_2d_medium;

#[path = "epu_gpu/cell_hex_2d_high.rs"]
mod cell_hex_2d_high;

#[path = "epu_gpu/split_prism.rs"]
mod split_prism;

#[path = "epu_gpu/split_prism_poles.rs"]
mod split_prism_poles;

#[path = "epu_gpu/split_planar.rs"]
mod split_planar;

#[path = "epu_gpu/split_bands.rs"]
mod split_bands;

#[path = "epu_gpu/split_bands_repair.rs"]
mod split_bands_repair;

#[path = "epu_gpu/surface_support.rs"]
mod surface_support;

#[path = "epu_gpu/cell_fractional.rs"]
mod cell_fractional;
