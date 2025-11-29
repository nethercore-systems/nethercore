const PI: f32 = 3.14159265359;

/// PBR-lite: Single sun + ambient + emissive.
/// Use alongside matcap/cubemap for ambient reflections.
///
/// All directions normalized, same coordinate space.
/// Returns linear RGB — apply tonemapping + gamma before display.
///
/// MRE texture packing: R=Metallic, G=Roughness, B=Emissive
fn pbr_lite(
    surface_normal: vec3f,
    view_dir: vec3f,       // surface TO camera
    light_dir: vec3f,      // surface TO light
    albedo: vec3f,         // linear RGB
    metallic: f32,
    roughness: f32,
    emissive: f32,
    light_color: vec3f,
    ambient_color: vec3f
) -> vec3f {
    let alpha = roughness * roughness;
    let alpha2 = alpha * alpha;
    
    let half_vec = normalize(light_dir + view_dir);
    let n_dot_l = max(dot(surface_normal, light_dir), 0.0);
    let n_dot_h = dot(surface_normal, half_vec);
    let v_dot_h = max(dot(view_dir, half_vec), 0.0);
    
    // F0: 4% for dielectrics, albedo for metals
    let f0 = mix(vec3f(0.04), albedo, metallic);
    
    // D: GGX distribution
    let d = n_dot_h * n_dot_h * (alpha2 - 1.0) + 1.0;
    let D = alpha2 / (PI * d * d);
    
    // F: Schlick fresnel (Karis exp2 approximation)
    let F = f0 + (1.0 - f0) * exp2((-5.55473 * v_dot_h - 6.98316) * v_dot_h);
    
    // Specular (G term omitted — cubemaps handle edge darkening)
    let specular = D * F;
    
    // Diffuse: energy-conserving Lambert
    let diffuse = (1.0 - f0) * (1.0 - metallic) * albedo / PI;
    
    // Combine
    let direct = (diffuse + specular) * light_color * n_dot_l;
    let ambient = ambient_color * albedo * (1.0 - metallic * 0.9);
    let glow = albedo * emissive;
    
    return direct + ambient + glow;
}