//! Production direct-light BRDF must retain its declared linear color gain.

#[test]
fn direct_diffuse_is_independent_of_specular_controls() {
    let helpers = include_str!("../../shaders/common/30_lighting.wgsl")
        .split("// Smooth distance attenuation")
        .next()
        .unwrap();
    let body = r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p: vec3u) {
    let r = f32(p.x) / 4.0;
    let spec = select(0.04, 1.0, p.y == 1u);
    let value = lambert_diffuse_probe_tamed(vec3f(0.,0.,1.),
        vec3f(-0.6,0.,-0.8), vec3f(0.25), r, vec3f(spec), vec3f(1.));
    textureStore(result,p.xy,vec4f(value,1.));
}
"#;
    let pixels = super::probe_image(&format!("{helpers}\n{body}"), 5, 2);
    let mut max_error = 0.0f32;
    for p in pixels {
        assert!(p[0].is_finite());
        // N dot L = 0.8, albedo = 0.25. Specular controls cannot alter Lambert.
        max_error = max_error.max((p[0] - 0.2).abs());
    }
    println!("direct diffuse max error={max_error}; tolerance=0.001");
    assert!(
        max_error < 0.001,
        "hidden specular/roughness gate dims direct diffuse"
    );
}

use super::{gpu, pipeline, read, texture};

#[test]
fn direct_specular_has_no_hidden_material_attenuation() {
    let (d, q) = gpu();
    let helpers = super::sources::BLINNPHONG_COMMON
        .split("// Fragment Shader")
        .next()
        .unwrap();
    let body = r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p: vec3u) {
    let roughness = f32(p.x) / 4.0;
    let shininess = 1.0 + 255.0 * (1.0 - roughness);
    let color = select(0.04, 1.0, p.y == 1u);
    let value = normalized_blinn_phong_specular(
        vec3f(0.,0.,1.), vec3f(0.,0.,1.), vec3f(0.,0.,-1.),
        shininess, roughness, vec3f(color), vec3f(1.)
    );
    textureStore(result, p.xy, vec4f(value, 1.));
}
"#;
    let p = pipeline(&d, format!("{helpers}\n{body}"), "probe");
    let output = texture(
        &d,
        5,
        2,
        wgpu::TextureFormat::Rgba16Float,
        wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
    );
    let view = output.create_view(&Default::default());
    let group = d.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &p.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&view),
        }],
    });
    let mut encoder = d.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_compute_pass(&Default::default());
        pass.set_pipeline(&p);
        pass.set_bind_group(0, &group, &[]);
        pass.dispatch_workgroups(5, 2, 1);
    }
    q.submit([encoder.finish()]);
    let pixels = read(&d, &q, &output);
    let mut max_error = 0.0f32;
    for y in 0..2 {
        for x in 0..5 {
            let n = 1.0 + 255.0 * (1.0 - x as f32 / 4.0);
            // At N=V=L the angular terms are one: declared Gotanda gain times color.
            let expected = (n * 0.0397436 + 0.0856832) * if y == 0 { 0.04 } else { 1.0 };
            let actual = pixels[y * 5 + x][0];
            assert!(actual.is_finite());
            max_error = max_error.max((actual - expected).abs());
        }
    }
    println!("direct BRDF max absolute error={max_error}; tolerance=0.01 (RGBA16F)");
    assert!(
        max_error < 0.01,
        "hidden roughness/material gain violates declared direct BRDF"
    );
}
