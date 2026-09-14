//! Read-only production-shader diagnostics; .005 regions/.01 RGB, no repair.
use super::*;
fn capture(name: &str, body: &str, width: u32, height: u32) {
    let pixels = if name == "stored" || name == "axis" {
        stored_image(body, width, height)
    } else {
        probe_image(body, width, height)
    };
    // Reuse the opt-in evidence pattern; existing receipts never block normal tests.
    if let Some(root) = std::env::var_os("PRISM_POLES_OUTPUT_DIR") {
        let root = std::path::PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        let dest = root.join(format!("split-prism-poles-{name}.csv"));
        assert!(!dest.exists(), "preserve receipts: {}", dest.display());
        let mut csv = String::from("x,y,r,g,b,a\n");
        for (i, v) in pixels.iter().enumerate() {
            csv += &format!(
                "{},{},{},{},{},{}\n",
                i % width as usize,
                i / width as usize,
                v[0],
                v[1],
                v[2],
                v[3]
            );
        }
        std::fs::write(dest, csv).unwrap();
    }
    assert!(
        pixels.iter().flatten().all(|x| x.is_finite()),
        "nonfinite retained, not zeroed"
    );
    println!("POLES exercised {name}: {width}x{height}; diagnostic, NOT continuity acceptance");
}
#[test]
fn legacy() {
    let text = include_str!("split_prism.rs");
    let body = text
        .split("r#\"")
        .nth(1)
        .unwrap()
        .split("\"#;")
        .next()
        .unwrap();
    capture("legacy", body, 396, 504);
}
#[test]
fn paths() {
    let body = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/epu_gpu/fixtures/split-prism-poles/split-prism-poles.wgsl"),
    )
    .unwrap();
    capture("paths", &body, 58 * 26, 72 * 6);
}
#[test]
fn stored() {
    let body = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/epu_gpu/fixtures/split-prism-poles/split-prism-poles-stored.wgsl"),
    )
    .unwrap();
    capture("stored", &body, 58 * 26, 72 * 6);
}
#[test]
fn axis() {
    let body = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/epu_gpu/fixtures/split-prism-poles/split-prism-poles-axis.wgsl"),
    )
    .unwrap();
    capture("axis", &body, 90, 72);
}
fn stored_image(body: &str, width: u32, height: u32) -> Vec<[f32; 4]> {
    let (d, q) = gpu();
    let p = pipeline(
        &d,
        format!("{EPU_COMMON}\n{EPU_BOUNDS}\n{EPU_FEATURES}\n{body}"),
        "probe",
    );
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/epu_gpu/fixtures/split-prism-poles/split-prism-poles-input.bin"),
    )
    .unwrap();
    let input = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: &bytes,
        usage: wgpu::BufferUsages::STORAGE,
    });
    let t = texture(
        &d,
        width,
        height,
        wgpu::TextureFormat::Rgba16Float,
        wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
    );
    let view = t.create_view(&Default::default());
    let group = d.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &p.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: input.as_entire_binding(),
            },
        ],
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
