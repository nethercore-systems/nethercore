// Sample gradient environment (Mode 0)
// data[0]: zenith color (RGBA8)
// data[1]: sky_horizon color (RGBA8)
// data[2]: ground_horizon color (RGBA8)
// data[3]: nadir color (RGBA8)
// data[4]: cloud_phase:u16 (low16) | shift:f16 (high16)
// data[5]: ext0 (u32): sun_dir_oct16(16) + sun_disk(8) + sun_halo(8)
// data[6]: ext1 (u32): sun_intensity(8) + horizon_haze(8) + sun_warmth(8) + cloudiness(8)
fn sample_gradient(data: array<u32, 14>, offset: u32, direction: vec3<f32>) -> vec4<f32> {
    let zenith = unpack_rgba8(data[offset]);
    let sky_horizon = unpack_rgba8(data[offset + 1u]);
    let ground_horizon = unpack_rgba8(data[offset + 2u]);
    let nadir = unpack_rgba8(data[offset + 3u]);

    // w4: cloud_phase:u16 (low16) | shift:f16 (high16)
    let w4 = data[offset + 4u];
    let shift = unpack2x16float(w4).y;
    let cloud_phase = w4 & 0xFFFFu;
    let cloud_t = f32(cloud_phase) / 65536.0; // loopable u16 driver (t ∈ [0,1))

    // w5: sun_dir_oct16 (low16) | sun_disk:u8 | sun_halo:u8
    // w6: sun_intensity:u8 | horizon_haze:u8 | sun_warmth:u8 | cloudiness:u8
    let ext0 = data[offset + 5u];
    let ext1 = data[offset + 6u];

    let cloudiness = f32((ext1 >> 24u) & 0xFFu) / 255.0;
    let horizon_haze = f32((ext1 >> 8u) & 0xFFu) / 255.0;
    let warmth = f32((ext1 >> 16u) & 0xFFu) / 255.0;
    let sun_intensity = f32(ext1 & 0xFFu) / 255.0 * 8.0;

    let dir = safe_normalize(direction, vec3<f32>(0.0, 0.0, 1.0));

    // Calculate blend factor with shift applied
    let y = clamp(dir.y + shift, -1.0, 1.0);

    // Smooth 4-color gradient: nadir (-1) -> ground_horizon -> sky_horizon -> zenith (+1)
    // Small horizon band for smooth ground/sky transition.
    var result: vec4<f32>;
    if (y < -0.1) {
        let t = (y + 1.0) / 0.9;
        result = mix(nadir, ground_horizon, t);
    } else if (y < 0.1) {
        let t = (y + 0.1) / 0.2;
        result = mix(ground_horizon, sky_horizon, t);
    } else {
        let t = (y - 0.1) / 0.9;
        result = mix(sky_horizon, zenith, t);
    }

    var rgb = result.rgb;

    // Horizon haze: gently pull sky/ground toward their respective horizon colors near y ~= 0.
    if (horizon_haze > 0.0) {
        let h = 1.0 - abs(y); // 1 at horizon, 0 at poles
        let haze_amount = horizon_haze * (h * h);
        let haze_color = select(ground_horizon.rgb, sky_horizon.rgb, y >= 0.0);
        rgb = mix(rgb, haze_color, haze_amount);
    }

    // Decode sun direction once (used for both sun + band orientation).
    let sun_dir = unpack_octahedral_u16(ext0 & 0xFFFFu);

    // Cloudiness / stylized bands (sky only): horizontal bands with trig-free drift.
    if (cloudiness > 0.0) {
        let sky_mask = smoothstep(0.0, 0.15, y); // fade in above horizon

        // Oriented coordinates: derive a horizontal axis from sun_dir and world up.
        let up = vec3<f32>(0.0, 1.0, 0.0);
        let right = safe_normalize(cross(up, sun_dir), vec3<f32>(1.0, 0.0, 0.0));
        let forward = cross(sun_dir, right);

        // Loopable drift (closed, trig-free) driven by cloud_phase.
        let drift = vec2<f32>(tri(cloud_t), tri(cloud_t + 0.25));
        let uv = vec2<f32>(dot(dir, right), dot(dir, forward)) * mix(3.0, 9.0, cloudiness) + drift * mix(0.2, 1.2, cloudiness);

        // Cheap wave-noise warp (no hash / no trig).
        let n = tri(dot(uv, vec2<f32>(1.7, 9.2)) + 0.35 * tri(dot(uv, vec2<f32>(-8.3, 2.8))));
        let warp = n * cloudiness * 0.25;

        let freq = mix(2.0, 12.0, cloudiness);
        let coord = (y * 0.5 + 0.5) * freq + warp + cloud_t * 2.0;

        let band = 1.0 - triwave(coord); // peaks at band centers
        let band_soft = smoothstep(0.15, 1.0, band);
        let band_amount = cloudiness * band_soft * sky_mask;

        // Tint bands using existing sky colors (keeps palettes art-directable)
        let band_color = mix(sky_horizon.rgb, zenith.rgb, 0.65);
        rgb = mix(rgb, band_color, band_amount * 0.6);
    }

    // Sun disc + halo (featured sky). World-space direction is supplied via env_gradient packing.
    if (sun_intensity > 0.0) {
        let sun_disk = f32((ext0 >> 16u) & 0xFFu) / 255.0;
        let sun_halo = f32((ext0 >> 24u) & 0xFFu) / 255.0;

        let sun_color = mix(vec3<f32>(1.0, 0.98, 0.9), vec3<f32>(1.0, 0.6, 0.2), warmth);

        let dot_sun = clamp(dot(dir, sun_dir), -1.0, 1.0);
        let sun_dist = 1.0 - dot_sun; // 0 at center
        let aa = fwidth(sun_dist) + 1e-6;

        let disk_r = mix(0.0005, 0.02, sun_disk);
        let halo_r = mix(disk_r * 2.0, 0.25, sun_halo);

        let disk = 1.0 - smoothstep(disk_r, disk_r + aa, sun_dist);
        let halo = 1.0 - smoothstep(disk_r, halo_r + aa, sun_dist);
        let halo_only = max(halo - disk, 0.0);

        rgb = rgb + sun_color * (sun_intensity * disk);
        rgb = rgb + sun_color * (sun_intensity * 0.25 * halo_only);
    }

    // Subtle world-locked dither to reduce banding in large gradients.
    let dither = (hash31(dir * 512.0) - 0.5) * (1.0 / 255.0);
    rgb = rgb + vec3<f32>(dither);

    // Mode 0 alpha is always opaque.
    return vec4<f32>(rgb, 1.0);
}

