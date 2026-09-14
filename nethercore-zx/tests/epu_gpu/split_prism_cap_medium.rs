//! Isolated frozen-source cap experiment; original witnesses remain untouched.
use super::*;
#[test]
fn capture() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../tmp/epu-review");
    let name = std::env::var("PRISM_CAP_SOURCE").unwrap();
    let label = std::env::var("PRISM_CAP_LABEL").unwrap_or(name);
    let dest = root.join(format!("split-prism-cap/{label}.csv"));
    assert!(!dest.exists());
    let body_path = std::env::var("PRISM_CAP_BODY").unwrap_or("split-prism-poles-stored.wgsl".into());
    let body = std::fs::read_to_string(root.join(body_path)).unwrap();
    let pixels = stored_image(&body, 58 * 26, 72 * 6);
    let mut csv = String::from("x,y,r,g,b,a\n");
    for (i, v) in pixels.iter().enumerate() {
        csv += &format!(
            "{},{},{},{},{},{}\n",
            i % (58 * 26),
            i / (58 * 26),
            v[0],
            v[1],
            v[2],
            v[3]
        );
    }
    std::fs::write(dest, csv).unwrap();
    assert!(pixels.iter().flatten().all(|v| v.is_finite()));
}
fn stored_image(body: &str, width: u32, height: u32) -> Vec<[f32; 4]> {
    let (d, q) = gpu();
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../tmp/epu-review/split-prism-cap");
    let old = std::fs::read_to_string(root.join("medium.wgsl")).unwrap();
    let candidate = std::fs::read_to_string(root.join(format!(
        "{}.wgsl",
        std::env::var("PRISM_CAP_SOURCE").unwrap()
    )))
    .unwrap();
    let source = format!("{EPU_COMMON}\n{EPU_BOUNDS}\n{EPU_FEATURES}\n{body}");
    assert_eq!(
        source.matches(&old).count(),
        1,
        "frozen shader must match actual assembly"
    );
    let p = pipeline(&d, source.replacen(&old, &candidate, 1), "probe");
    let input_path = std::env::var("PRISM_CAP_INPUT")
        .unwrap_or("../tmp/epu-review/split-prism-poles-input.bin".into());
    let bytes =
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(input_path)).unwrap();
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
