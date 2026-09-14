//! Ignored hardware-timestamp cost probe for the production SCATTER evaluator.
use super::*;

const WIDTH: u32 = 32;
const HEIGHT: u32 = 32;
const SAMPLES: usize = 5;
const SEED: u32 = 17;
const INTENSITY: u32 = 255;

fn source(variant: u32, density: u32) -> String {
    let prefix = format!(
        "{EPU_COMMON}\n{EPU_BOUNDS}\n{EPU_FEATURES}\nconst PROBE_VARIANT:u32 = {variant}u;\nconst PROBE_DENSITY:u32 = {density}u;\nconst PROBE_SEED:u32 = {SEED}u;\nconst PROBE_INTENSITY:u32 = {INTENSITY}u;\n"
    );
    prefix
        + r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(8, 8) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let axis = decode_dir16(0xff80u);
    var right = cross(axis, vec3f(0., 0., 1.));
    if length(right) < 0.01 { right = cross(axis, vec3f(1., 0., 0.)); }
    right = normalize(right);
    let forward = cross(right, axis);
    let local = ((vec2f(p.xy) + 0.5) / 32.0 * 2.0 - 1.0) * 0.35;
    let radius_sq = dot(local, local);
    let dir = normalize(axis * sqrt(max(0.0, 1.0 - radius_sq))
        + right * local.x + forward * local.y);
    let instr = vec4u(
        (PROBE_SEED << 24u) | (0xff80u << 8u) | PROBE_INTENSITY,
        (255u << 24u) | (PROBE_DENSITY << 16u) | (255u << 8u) | 255u,
        0xffffffffu,
        (OP_SCATTER << 27u) | (7u << 24u)
            | (((3u << 3u) | PROBE_VARIANT) << 16u) | 65535u);
    let sample = eval_scatter(dir, instr, 1.0);
    textureStore(result, p.xy, vec4f(sample.rgb * epu_saturate(sample.w), 1.0));
}
"#
}

fn timestamp_pair(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &wgpu::ComputePipeline,
    group: &wgpu::BindGroup,
    query_set: &wgpu::QuerySet,
    workgroups_x: u32,
    workgroups_y: u32,
) -> (u64, u64) {
    let resolve = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("scatter timestamp resolve"),
        size: 16,
        usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::QUERY_RESOLVE,
        mapped_at_creation: false,
    });
    let timestamps = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("scatter timestamp readback"),
        size: 16,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("scatter cost sample"),
            timestamp_writes: Some(wgpu::ComputePassTimestampWrites {
                query_set,
                beginning_of_pass_write_index: Some(0),
                end_of_pass_write_index: Some(1),
            }),
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, group, &[]);
        pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
    }
    encoder.resolve_query_set(query_set, 0..2, &resolve, 0);
    encoder.copy_buffer_to_buffer(&resolve, 0, &timestamps, 0, 16);
    queue.submit([encoder.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    timestamps
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| tx.send(result).unwrap());
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    rx.recv().unwrap().unwrap();
    let bytes = timestamps.slice(..).get_mapped_range();
    let pair = [
        u64::from_le_bytes(bytes[0..8].try_into().unwrap()),
        u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
    ];
    drop(bytes);
    timestamps.unmap();
    (pair[0], pair[1])
}

#[test]
#[ignore = "explicit hardware cost probe"]
fn scatter_gpu_cost_probe() {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let adapter = match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        ..Default::default()
    })) {
        Ok(adapter) => adapter,
        Err(error) => {
            println!("UNSUPPORTED no GPU adapter: {error:?}");
            return;
        }
    };
    let info = adapter.get_info();
    println!("GPU adapter={info:?}");
    if !adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
        println!("UNSUPPORTED TIMESTAMP_QUERY adapter_feature_missing");
        return;
    }
    let (device, queue) =
        match pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            required_features: wgpu::Features::TIMESTAMP_QUERY,
            ..Default::default()
        })) {
            Ok(device) => device,
            Err(error) => {
                println!("UNSUPPORTED TIMESTAMP_QUERY device_request_failed: {error:?}");
                return;
            }
        };
    let period_ns = queue.get_timestamp_period() as f64;
    if !period_ns.is_finite() || period_ns <= 0.0 {
        println!("UNSUPPORTED invalid_timestamp_period_ns={period_ns}");
        return;
    }
    let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
        label: Some("scatter cost timestamps"),
        ty: wgpu::QueryType::Timestamp,
        count: 2,
    });
    println!(
        "SCATTER cost resolution={WIDTH}x{HEIGHT} workgroup=8x8 sampling=warmup:1 samples:{SAMPLES} timestamp_period_ns={period_ns} seed={SEED} intensity_u8={INTENSITY} domain=3"
    );

    for density in [255u32, 64u32] {
        for variant in 0u32..7 {
            let pipeline = pipeline(&device, source(variant, density), "probe");
            let target = texture(
                &device,
                WIDTH,
                HEIGHT,
                wgpu::TextureFormat::Rgba16Float,
                wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
            );
            let view = target.create_view(&Default::default());
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("scatter cost output"),
                layout: &pipeline.get_bind_group_layout(0),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                }],
            });
            let mut warmup = device.create_command_encoder(&Default::default());
            {
                let mut pass = warmup.begin_compute_pass(&Default::default());
                pass.set_pipeline(&pipeline);
                pass.set_bind_group(0, &group, &[]);
                pass.dispatch_workgroups(WIDTH / 8, HEIGHT / 8, 1);
            }
            queue.submit([warmup.finish()]);
            device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

            let mut samples_ns = Vec::with_capacity(SAMPLES);
            for _ in 0..SAMPLES {
                let (start, end) = timestamp_pair(
                    &device,
                    &queue,
                    &pipeline,
                    &group,
                    &query_set,
                    WIDTH / 8,
                    HEIGHT / 8,
                );
                assert!(
                    end >= start,
                    "timestamp order invalid: start={start} end={end}"
                );
                samples_ns.push((end - start) as f64 * period_ns);
            }
            let pixels = read(&device, &queue, &target);
            assert!(!pixels.is_empty(), "scatter cost output readback is empty");
            assert!(
                pixels.iter().all(|p| p.iter().all(|v| v.is_finite())),
                "scatter cost output contains non-finite values"
            );
            assert!(
                pixels
                    .iter()
                    .any(|p| p[..3].iter().any(|v| v.abs() > 0.0001)),
                "scatter cost output is empty/black"
            );
            let min = samples_ns.iter().copied().fold(f64::INFINITY, f64::min);
            let max = samples_ns.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let mut sorted = samples_ns.clone();
            sorted.sort_by(f64::total_cmp);
            let median = sorted[SAMPLES / 2];
            println!(
                "SCATTER cost variant={variant} density_u8={density} size_u8=255 samples={} median_ns={median:.0} min_ns={min:.0} max_ns={max:.0}",
                samples_ns.len()
            );
        }
    }
}

const CACHE_SMOKE_RESOLUTION: u32 = 32;
const CACHE_DEFAULT_RESOLUTION: u32 = 128;
const CACHE_MODERATE_DENSITY: u32 = 64;
const CACHE_MAX_DENSITY: u32 = 255;
const CACHE_SEED: u64 = 17;
const CACHE_SIZE: u64 = 255;
const CACHE_TWINKLE: u64 = 8;

fn cache_scatter_state(density: u32) -> [u32; 32] {
    // Production EpuLayer encoding: TANGENT_LOCAL + BUBBLES, white/white,
    // nonzero parameters, and no compile-time specialization in the shader.
    let meta5 = (3u64 << 3) | 3;
    let hi = (0x0Au64 << 59)
        | (7u64 << 56)
        | ((meta5 >> 1) << 49)
        | ((meta5 & 1) << 48)
        | (0xFF_FFFFu64 << 24)
        | 0xFF_FFFFu64;
    let lo = (255u64 << 56)
        | ((density as u64) << 48)
        | (CACHE_SIZE << 40)
        | ((CACHE_TWINKLE * 16) << 32)
        | (CACHE_SEED << 24)
        | (0xFF80u64 << 8)
        | (15u64 << 4)
        | 15;
    let mut words = [0u32; 32];
    words[0] = lo as u32;
    words[1] = (lo >> 32) as u32;
    words[2] = hi as u32;
    words[3] = (hi >> 32) as u32;
    words
}

fn cache_case(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    query_set: &wgpu::QuerySet,
    period_ns: f64,
    resolution: u32,
    label: &str,
    state_words: &[u32; 32],
    scatter: bool,
) -> Vec<f64> {
    let cache = pipeline(
        device,
        format!("{EPU_COMMON}\n{EPU_BOUNDS}\n{EPU_FEATURES}\n{EPU_COMPUTE_ENV}"),
        "epu_build",
    );
    let state = buffer(device, state_words, wgpu::BufferUsages::STORAGE);
    let active = buffer(device, &[0], wgpu::BufferUsages::STORAGE);
    let frame = buffer(device, &[1, resolution, 0, 0], wgpu::BufferUsages::UNIFORM);
    let target = texture(
        device,
        resolution,
        resolution,
        wgpu::TextureFormat::Rgba16Float,
        wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
    );
    let view = target.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        ..Default::default()
    });
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("EPU cache cost output"),
        layout: &cache.get_bind_group_layout(0),
        entries: &[
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
        ],
    });
    let workgroups = resolution.div_ceil(8);
    let mut warmup = device.create_command_encoder(&Default::default());
    {
        let mut pass = warmup.begin_compute_pass(&Default::default());
        pass.set_pipeline(&cache);
        pass.set_bind_group(0, &group, &[]);
        pass.dispatch_workgroups(workgroups, workgroups, 1);
    }
    queue.submit([warmup.finish()]);
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

    let mut samples_ns = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let (start, end) = timestamp_pair(
            device, queue, &cache, &group, query_set, workgroups, workgroups,
        );
        assert!(
            end >= start,
            "timestamp order invalid: start={start} end={end}"
        );
        samples_ns.push((end - start) as f64 * period_ns);
    }

    let pixels = read(device, queue, &target);
    assert!(!pixels.is_empty(), "{label} cache output readback is empty");
    assert!(
        pixels.iter().all(|p| p.iter().all(|v| v.is_finite())),
        "{label} cache output contains non-finite values"
    );
    if scatter {
        assert!(
            pixels
                .iter()
                .any(|p| p[..3].iter().any(|v| v.abs() > 0.0001)),
            "{label} scatter cache output is empty/black"
        );
    } else {
        assert!(
            pixels
                .iter()
                .all(|p| p[..3].iter().all(|v| v.abs() <= 0.0001)),
            "{label} NOP cache control is not black"
        );
    }
    let mut sorted = samples_ns.clone();
    sorted.sort_by(f64::total_cmp);
    let median = sorted[SAMPLES / 2];
    let min = samples_ns.iter().copied().fold(f64::INFINITY, f64::min);
    let max = samples_ns.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    println!(
        "EPU_COMPUTE_ENV cost case={label} resolution={resolution}x{resolution} workgroups={workgroups}x{workgroups} warmup=1 samples={} median_ns={median:.0} min_ns={min:.0} max_ns={max:.0}",
        samples_ns.len()
    );
    samples_ns
}

#[test]
#[ignore = "explicit hardware cost probe"]
fn epu_cache_gpu_cost_probe() {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let adapter = match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        ..Default::default()
    })) {
        Ok(adapter) => adapter,
        Err(error) => {
            println!("UNSUPPORTED no GPU adapter: {error:?}");
            return;
        }
    };
    let info = adapter.get_info();
    println!("EPU_COMPUTE_ENV adapter={info:?}");
    if !adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
        println!("UNSUPPORTED TIMESTAMP_QUERY adapter_feature_missing");
        return;
    }
    let (device, queue) =
        match pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            required_features: wgpu::Features::TIMESTAMP_QUERY,
            ..Default::default()
        })) {
            Ok(device) => device,
            Err(error) => {
                println!("UNSUPPORTED TIMESTAMP_QUERY device_request_failed: {error:?}");
                return;
            }
        };
    let period_ns = queue.get_timestamp_period() as f64;
    if !period_ns.is_finite() || period_ns <= 0.0 {
        println!("UNSUPPORTED invalid_timestamp_period_ns={period_ns}");
        return;
    }
    let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
        label: Some("EPU cache cost timestamps"),
        ty: wgpu::QueryType::Timestamp,
        count: 2,
    });
    println!(
        "EPU_COMPUTE_ENV config=one SCATTER/TANGENT_LOCAL/BUBBLES layer white/white seed={} size_u8={} twinkle_q={} default_resolution={}x{} timestamp_period_ns={period_ns} excludes=SH,mips,frame orchestration",
        CACHE_SEED, CACHE_SIZE, CACHE_TWINKLE, CACHE_DEFAULT_RESOLUTION, CACHE_DEFAULT_RESOLUTION
    );

    // Small supported real-cache smoke before the default runtime resolution.
    cache_case(
        &device,
        &queue,
        &query_set,
        period_ns,
        CACHE_SMOKE_RESOLUTION,
        "smoke_moderate",
        &cache_scatter_state(CACHE_MODERATE_DENSITY),
        true,
    );

    let moderate = cache_case(
        &device,
        &queue,
        &query_set,
        period_ns,
        CACHE_DEFAULT_RESOLUTION,
        "default128_moderate",
        &cache_scatter_state(CACHE_MODERATE_DENSITY),
        true,
    );
    let max = cache_case(
        &device,
        &queue,
        &query_set,
        period_ns,
        CACHE_DEFAULT_RESOLUTION,
        "default128_max",
        &cache_scatter_state(CACHE_MAX_DENSITY),
        true,
    );
    let control = cache_case(
        &device,
        &queue,
        &query_set,
        period_ns,
        CACHE_DEFAULT_RESOLUTION,
        "default128_nop_control",
        &[0; 32],
        false,
    );
    let median = |samples: &[f64]| {
        let mut sorted = samples.to_vec();
        sorted.sort_by(f64::total_cmp);
        sorted[SAMPLES / 2]
    };
    let control_median = median(&control);
    println!(
        "EPU_COMPUTE_ENV overhead default128 moderate_minus_nop_ns={:.0} max_minus_nop_ns={:.0} control_median_ns={control_median:.0}",
        median(&moderate) - control_median,
        median(&max) - control_median,
    );
}
