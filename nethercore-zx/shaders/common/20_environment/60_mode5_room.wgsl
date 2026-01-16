// ============================================================================
// Mode 5: Room (Interior Box / Liminal Space)
// ============================================================================
// w0: color_ceiling_RGB(24) | viewer_x_snorm8(8)
// w1: color_floor_RGB(24) | viewer_y_snorm8(8)
// w2: color_walls_RGB(24) | viewer_z_snorm8(8)
// w3: panel_size:f16 (low16) | panel_gap:u8 | corner_darken:u8
// w4: light_dir_oct16 (low16) | light_intensity:u8 | room_scale_u8
// w5: accent_mode:u8 | accent:u8 | phase:u16 (high16)
// w6: light_tint_RGB(24) | roughness:u8
fn sample_room(data: array<u32, 14>, offset: u32, direction: vec3<f32>) -> vec4<f32> {
    // Colors (RGB only; low byte stores viewer snorm8).
    let ceiling = unpack_rgb8(data[offset]);
    let floor_c = unpack_rgb8(data[offset + 1u]);
    let walls = unpack_rgb8(data[offset + 2u]);

    // Viewer snorm8 → [-1,1], then scale by room_scale.
    let vx = f32(bitcast<i32>((data[offset] & 0xFFu) << 24u) >> 24) / 127.0;
    let vy = f32(bitcast<i32>((data[offset + 1u] & 0xFFu) << 24u) >> 24) / 127.0;
    let vz = f32(bitcast<i32>((data[offset + 2u] & 0xFFu) << 24u) >> 24) / 127.0;

    let w3 = data[offset + 3u];
    let panel_size_in = unpack2x16float(w3).x;
    let panel_gap01 = f32((w3 >> 16u) & 0xFFu) / 255.0;
    let corner_darken = f32((w3 >> 24u) & 0xFFu) / 255.0;

    let w4 = data[offset + 4u];
    let light_ray = unpack_octahedral_u16(w4); // direction rays travel
    let light_intensity = f32((w4 >> 16u) & 0xFFu) / 255.0;
    let room_scale = max(0.1, f32((w4 >> 24u) & 0xFFu) / 10.0);

    let viewer = vec3<f32>(vx, vy, vz) * room_scale;

    let w5 = data[offset + 5u];
    let accent_mode = w5 & 0xFFu;
    let accent = f32((w5 >> 8u) & 0xFFu) / 255.0;
    let phase01 = f32((w5 >> 16u) & 0xFFFFu) / 65536.0;

    let w6 = data[offset + 6u];
    let light_tint = unpack_rgb8(w6);
    let roughness = f32(w6 & 0xFFu) / 255.0;

    let dir = safe_normalize(direction, vec3<f32>(0.0, 0.0, 1.0));

    // Ray-box intersection (viewer inside an axis-aligned box).
    let inv_dir = 1.0 / dir;
    let t_min = (-room_scale - viewer) * inv_dir;
    let t_max = (room_scale - viewer) * inv_dir;

    let t1 = min(t_min, t_max);
    let t2 = max(t_min, t_max);

    let t_near = max(max(t1.x, t1.y), t1.z);
    let t_far = min(min(t2.x, t2.y), t2.z);

    if (t_far < 0.0) {
        return vec4<f32>(0.0);
    }

    let t = select(t_far, t_near, t_near > 0.0);
    let hit = viewer + dir * t;

    // Select face + UV coordinates (face_pos in world-ish units).
    let abs_hit = abs(hit);
    var base_col: vec3<f32>;
    var N: vec3<f32>;
    var face_pos: vec2<f32>;

    if (abs_hit.y > abs_hit.x && abs_hit.y > abs_hit.z) {
        if (hit.y < 0.0) {
            base_col = floor_c;
            N = vec3<f32>(0.0, 1.0, 0.0);
        } else {
            base_col = ceiling;
            N = vec3<f32>(0.0, -1.0, 0.0);
        }
        face_pos = hit.xz;
    } else if (abs_hit.x > abs_hit.z) {
        base_col = walls;
        N = vec3<f32>(select(1.0, -1.0, hit.x > 0.0), 0.0, 0.0);
        face_pos = hit.zy;
    } else {
        base_col = walls;
        N = vec3<f32>(0.0, 0.0, select(1.0, -1.0, hit.z > 0.0));
        face_pos = hit.xy;
    }

    // Anti-moiré clamp for panel_size (world units).
    let pix = max(fwidth(face_pos.x), fwidth(face_pos.y)) + 1e-6;
    let panel_size = max(max(panel_size_in, 0.05), pix * 2.0);

    // Panel seams (SDF distance to gridlines in cell space).
    let panel_uv = face_pos / panel_size;
    let f = fract(panel_uv);
    let edge = min(min(f.x, 1.0 - f.x), min(f.y, 1.0 - f.y));
    let edge_aa = fwidth(edge) + 1e-6;
    let seam_w = panel_gap01 * 0.25;
    let seam = select(0.0, 1.0 - smoothstep(seam_w, seam_w + edge_aa, edge), panel_gap01 > 0.0);

    // Slight per-panel variation (kept cheap).
    let pid = vec2<i32>(i32(floor(panel_uv.x)), i32(floor(panel_uv.y)));
    let ph = hash_cell_u32(pid, hash_u32(bitcast<u32>(w3) ^ bitcast<u32>(w4)), 0x31415926u);
    let panel_var = 0.92 + 0.08 * hash01_u32(ph);

    // Base albedo shaping: seams are darker; panels slightly vary.
    var albedo = base_col * panel_var;
    albedo = albedo * (1.0 - seam * 0.15);

    // Corner/edge occlusion (cheap AO cue).
    let uvn = face_pos / room_scale; // [-1,1]
    let edge_d = 1.0 - max(abs(uvn.x), abs(uvn.y));
    let ao = 1.0 - corner_darken * (1.0 - edge_d) * (1.0 - edge_d);

    // Lighting (ray direction convention).
    let L = -safe_normalize(light_ray, vec3<f32>(0.0, -1.0, 0.0));
    let ndl = saturate(dot(N, L));
    let diffuse = ndl * light_intensity;

    // Cheap spec-like highlight shaped by roughness (avoid pow).
    let V = -dir;
    let R = reflect(light_ray, N);
    let s = saturate(dot(R, V));
    let s2 = s * s;
    let s4 = s2 * s2;
    let s8 = s4 * s4;
    let s16 = s8 * s8;
    let s32 = s16 * s16;
    let gloss = 1.0 - roughness;
    let spec = mix(s4, s32, gloss) * light_intensity * (0.15 + 0.35 * gloss);

    // Accent energy (loopable; tinted by light_tint).
    var accent_mask = 0.0;
    if (accent > 0.0) {
        let pulse = 0.5 + 0.5 * tri(phase01);
        let sweep_coord = (face_pos.x + face_pos.y) * (0.11 / room_scale) + phase01;
        let sweep_tri = tri(sweep_coord) * 0.5 + 0.5;
        let sweep = smoothstep(0.65, 1.0, sweep_tri);

        if (accent_mode == 0u) { // Seams
            accent_mask = seam;
        } else if (accent_mode == 1u) { // Sweep
            accent_mask = sweep;
        } else if (accent_mode == 2u) { // Seams+Sweep
            accent_mask = max(seam, sweep);
        } else if (accent_mode == 3u) { // Pulse
            accent_mask = pulse * 0.6;
        }
    }

    let ambient = 0.22;
    // Directional light is tinted by light_tint.
    let lit = albedo * ambient * ao + (albedo * diffuse) * (light_tint * ao);
    let lit_spec = light_tint * spec;
    let lit_acc = light_tint * (accent * accent_mask * 2.25);

    return vec4<f32>(lit + lit_spec + lit_acc, 1.0);
}
