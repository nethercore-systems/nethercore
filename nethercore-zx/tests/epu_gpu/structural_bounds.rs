//! Bounded structural continuity: preserve authored roofs/wedges, not a smoothing gate.
use super::*;

#[test]
fn structural_bounds_packed_wrap_and_endpoints() {
    // Fixed before execution: half-float readback + 2e-6-radian paired directions.
    // 32 endpoint corners, exact neutral-composition controls, strength 0 and 7.
    const TOL: f32 = 0.001;
    const ROWS: usize = 11 * 35 * 3 * 11;
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) tile:vec3u) {
    let p = vec3u(tile.x%16u,tile.y+(tile.x/16u)*(33u*35u),0u);
    let elevation_id = p.y % 11u;
    let axis_id = (p.y / 11u) % 3u;
    let control = (p.y / 33u) % 35u;
    let path = p.y / (33u * 35u);
    let opcode = select(OP_SECTOR, OP_SILHOUETTE, path >= 3u);
    let skyline_variants = array<u32,8>(1u,6u,0u,2u,3u,4u,5u,7u);
    let variant = select(path, skyline_variants[max(path,3u)-3u], path >= 3u);
    let axes = array<u32,3>(0xff80u, 0x8080u, 0xa060u);
    let axis_bits = axes[axis_id];
    let up = decode_dir16(axis_bits);
    let ref_axis = select(vec3f(0.,1.,0.), vec3f(1.,0.,0.), abs(up.y) > 0.9);
    let t = normalize(cross(ref_axis, up)); let b = normalize(cross(up, t));
    var intensity = select(0u,255u,(control & 1u)!=0u);
    var a = select(0u,255u,(control & 2u)!=0u);
    var bb = select(0u,255u,(control & 4u)!=0u);
    var c = select(0u,240u,(control & 8u)!=0u);
    var d = select(0u,255u,(control & 16u)!=0u);
    var alpha = 15u;
    if control >= 32u {
        intensity = select(255u,4u,path>=3u);
        a = select(0u,select(112u,49u,path==4u),path>=3u);
        bb = select(137u,select(135u,75u,path==4u),path>=3u);
        c = 0u; d = 210u;
        if control == 33u { alpha=0u; }
        if control == 34u { alpha=7u; }
    }
    let domain = (p.x / 2u) % 4u;
    let instr = vec4u((d<<24u)|(axis_bits<<8u)|(alpha<<4u),
        (intensity<<24u)|(a<<16u)|(bb<<8u)|c,
        0x903060a0u, (opcode<<27u)|(7u<<24u)|(3u<<21u)|(((domain<<3u)|variant)<<16u)|0x7885u);
    let phi = select(-PI+0.000001,PI-0.000001,(p.x%2u)==1u);
    var y = -1.0 + f32(elevation_id)*0.25;
    // At chart wrap both selected skylines have zero raw height. Exercise the
    // narrow roof/base transition centers, not only the coarse elevation grid.
    if elevation_id == 9u { y=mix(-0.3,0.5,f32(a)/255.); }
    if elevation_id == 10u { y=max(-1.,mix(-0.3,0.5,f32(a)/255.)-mix(0.05,1.2,f32(d)/255.)); }
    // Added profiles can have a nonzero wrap height. Both sides use the same
    // on-cut roof/base elevation; do not follow each side independently.
    if path >= 5u && elevation_id >= 9u {
        let raw = silhouette_height(0.,variant,1u+(c>>4u)/2u,f32(variant)*13.7+42.0);
        let roof = clamp(mix(-0.3,0.5,f32(a)/255.)+raw*mix(0.1,1.,f32(bb)/255.)*0.5,-1.,1.);
        y = select(roof,clamp(roof-mix(0.05,1.2,f32(d)/255.),-1.,1.),elevation_id==10u);
    }
    let dir = normalize(up*y + sqrt(max(0.,1.-y*y))*(t*cos(phi)+b*sin(phi)));
    let v = evaluate_bounds_layer(dir,instr,opcode,up,RegionWeights(1.,0.,0.));
    var layers: array<vec4u,8>; layers[0]=instr;
    let composed = evaluate_epu_layers(dir,layers);
    let err = max(max(abs(composed.x-v.sample.rgb.x*v.sample.w),abs(composed.y-v.sample.rgb.y*v.sample.w)),abs(composed.z-v.sample.rgb.z*v.sample.w));
    var value = vec4f(v.regions.sky,v.regions.wall,v.regions.floor,v.region_mix);
    if p.x>=8u { value=vec4f(composed,err); }
    textureStore(result,tile.xy,value);
}
"#,
        16 * 11,
        35 * 3 * 11,
    );
    // Tile paths across X to stay below the device texture-height limit.
    let pixels: Vec<_> = (0..ROWS)
        .flat_map(|row| {
            let start = (row % (35 * 3 * 11)) * (16 * 11) + (row / (35 * 3 * 11)) * 16;
            pixels[start..start + 16].iter().copied()
        })
        .collect();
    assert_eq!(pixels.len(), ROWS * 16);
    let mut maximum = 0.0f32;
    let mut owned = [[0.0f32; 3]; 11];
    let mut brightness = [0.0f32; 11];
    for row in 0..ROWS {
        let path = row / (35 * 3 * 11);
        for x in 0..16 {
            let p = pixels[row * 16 + x];
            assert!(
                p.iter()
                    .all(|v| v.is_finite() && *v >= -TOL && *v <= 1. + TOL),
                "row={row} x={x}: {p:?}"
            );
            if x < 8 {
                assert!(
                    (p[0] + p[1] + p[2] - 1.).abs() <= TOL && p[3] == 1.,
                    "normalization row={row}: {p:?}"
                );
                for c in 0..3 {
                    owned[path][c] = owned[path][c].max(p[c]);
                }
                if path >= 3 && (row / 33) % 35 == 33 {
                    assert_eq!(p, [1., 0., 0., 1.], "zero silhouette strength");
                }
                if path < 3 && (row / 33) % 35 < 32 && ((row / 33) % 35) & 1 == 0 {
                    assert_eq!(p, [0., 0., 1., 1.], "closed sector");
                }
            } else {
                assert!(
                    p[3] <= TOL,
                    "full evaluator paint mismatch row={row}: {p:?}"
                );
                brightness[path] = brightness[path].max(p[0] + p[1] + p[2]);
            }
            // Domain bits are reserved for these axis-cylinder bounds: exact same direction.
            let reference = pixels[row * 16 + (x % 2) + (if x >= 8 { 8 } else { 0 })];
            assert_eq!(
                p, reference,
                "reserved domain altered path={path} row={row} x={x}"
            );
        }
        for x in (0..16).step_by(2) {
            for c in 0..4 {
                let error = (pixels[row * 16 + x][c] - pixels[row * 16 + x + 1][c]).abs();
                maximum = maximum.max(error);
                assert!(
                    error <= TOL,
                    "periodic seam row={row} x={x} channel={c} error={error}"
                );
            }
        }
    }
    for path in 0..11 {
        assert!(
            owned[path].iter().all(|v| *v > 0.1) && brightness[path] > 0.1,
            "vacuous path={path}: {:?}",
            owned[path]
        );
    }
    println!(
        "STRUCTURAL packed rows={ROWS} samples={} max_wrap={maximum} tolerance={TOL} ownership={owned:?} brightness={brightness:?}",
        pixels.len()
    );
}

#[test]
fn structural_bounds_skyline_cell_identity() {
    const TOL: f32 = 0.001;
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let variant=select(1u,6u,p.y>=16u);
    let period=select(16.0,8.0,variant==6u);
    let cell=f32(p.y%16u)%period;
    let positions=array<f32,9>(0.00001,0.05,0.1,0.25,0.5,0.75,0.9,0.95,0.99999);
    let u=(cell+positions[p.x])/period;
    let seed=f32(variant)*13.7+42.0;
    let h=silhouette_height(u,variant,8u,seed);
    textureStore(result,p.xy,vec4f(h,silhouette_height(u+1.0,variant,8u,seed),silhouette_height(u-1.0,variant,8u,seed),silhouette_height((cell+0.5)/period,variant,8u,seed)));
}
"#,
        9,
        32,
    );
    let mut max_error = 0.0f32;
    let mut maxima = [0.0f32; 2];
    for row in 0..32 {
        for x in 0..9 {
            let p = pixels[row * 9 + x];
            assert!(p.iter().all(|v| v.is_finite() && *v >= 0. && *v <= 1.));
            for c in 1..3 {
                let error = (p[0] - p[c]).abs();
                max_error = max_error.max(error);
                assert!(
                    error <= TOL,
                    "matching skyline identity row={row} x={x}: {p:?}"
                );
            }
            maxima[row / 16] = maxima[row / 16].max(p[0]);
            // Only actual cell endpoints have zero support; roofs within cells need not be smooth.
            if x == 0 || x == 8 {
                assert!(p[0] <= TOL, "cell join row={row}: {p:?}");
            }
        }
    }
    assert!(
        maxima.iter().all(|v| *v > 0.1),
        "deleted skyline: {maxima:?}"
    );
    println!(
        "STRUCTURAL skyline cells samples={} max_identity={max_error} tolerance={TOL} cores={maxima:?}",
        pixels.len()
    );
}

#[test]
fn structural_bounds_all_silhouette_profiles() {
    // Actual runtime-fed profiles: all mountain lattice joins, cell joins,
    // periodic sinusoidal samples and spire centres/antipodes. Fixed .001 limit.
    let mut inputs = Vec::new();
    for (variant, period) in [512u32, 16, 12, 48, 72, 10, 8, 6].into_iter().enumerate() {
        for cell in 0..period {
            inputs.extend([variant as u32, cell, period, 0]);
            if variant == 7 {
                inputs.extend([variant as u32, cell, period, 1]);
            }
        }
    }
    let rows = inputs.len() / 4;
    let pixels = probe_image_input_bounds(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float,write>;
@group(0) @binding(1) var<storage,read> inputs: array<vec4u>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let row=inputs[p.y]; let variant=row.x; let seed=f32(variant)*13.7+42.0;
    var cut=f32(row.y)/f32(row.z);
    if variant==7u { cut=fract(silhouette_hash(f32(row.y),seed)+f32(row.w)*0.5); }
    if p.x==8u {
        let u=fract((f32(row.y)+0.5)/f32(row.z));
        textureStore(result,p.xy,vec4f(silhouette_height(u,variant,8u,seed),0.,0.,0.));
        return;
    }
    let eps=array<f32,4>(0.001,0.0001,0.00001,0.000001)[p.x%4u];
    let left=silhouette_height(fract(cut-eps),variant,8u,seed);
    let centre=silhouette_height(cut,variant,8u,seed);
    let right=silhouette_height(fract(cut+eps),variant,8u,seed);
    let same=silhouette_height(fract(cut+2.0*eps),variant,8u,seed);
    // The exact same GPU path with an injected constant jump must reject.
    textureStore(result,p.xy,vec4f(left,centre,right+select(0.,0.05,p.x>=4u),same));
}
"#,
        EPU_BOUNDS,
        9,
        rows as u32,
        &inputs,
    );
    assert_eq!(pixels.len(), rows * 9);
    let mut maxima = [0.0f32; 8];
    let mut ranges = [[f32::INFINITY, f32::NEG_INFINITY]; 8];
    let mut rejected = 0;
    for row in 0..rows {
        let variant = inputs[row * 4] as usize;
        let final_pair = pixels[row * 9 + 3];
        for point in &pixels[row * 9..row * 9 + 9] {
            assert!(point.iter().all(|v| v.is_finite()));
        }
        let err = (final_pair[0] - final_pair[2])
            .abs()
            .max((final_pair[0] - final_pair[1]).abs())
            .max((final_pair[2] - final_pair[1]).abs());
        assert!(
            (final_pair[2] - final_pair[3]).abs() <= 0.001,
            "same-side profile gradient row={row}: {final_pair:?}"
        );
        maxima[variant] = maxima[variant].max(err);
        assert!(
            err <= 0.001,
            "profile join variant={variant} row={row}: {final_pair:?}"
        );
        let first = pixels[row * 9];
        let coarse = (first[0] - first[2])
            .abs()
            .max((first[0] - first[1]).abs())
            .max((first[2] - first[1]).abs());
        if coarse > 0.001 {
            assert!(err < coarse, "noncontracting profile row={row}");
        }
        let negative = pixels[row * 9 + 7];
        if (negative[0] - negative[2]).abs() > 0.001 {
            rejected += 1;
        }
        for value in [final_pair[1], pixels[row * 9 + 8][0]] {
            ranges[variant][0] = ranges[variant][0].min(value);
            ranges[variant][1] = ranges[variant][1].max(value);
        }
    }
    assert_eq!(rejected, rows, "constant-jump discriminator");
    for (variant, range) in ranges.iter().enumerate() {
        assert!(
            range[1] - range[0] > 0.05,
            "flattened profile {variant}: {range:?}"
        );
    }
    println!(
        "SILHOUETTE_PROFILES rows={rows} samples={} final_errors={maxima:?} ranges={ranges:?} injected_jumps_rejected={rejected} tolerance=.001",
        pixels.len()
    );
}

#[test]
fn structural_bounds_sector_authored_boundaries() {
    const TOL: f32 = 0.001;
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let variant=p.y%3u;
    let center=array<u32,3>(0u,127u,255u)[p.y/3u];
    let width=137u;
    let up=decode_dir16(0xff80u);
    let t=normalize(cross(vec3f(1.,0.,0.),up));let b=normalize(cross(up,t));
    let offsets=array<f32,7>(-0.4,-0.26862746,-0.1,0.,0.1,0.26862746,0.4);
    let turns=f32(center)/255.0+offsets[p.x%7u]+select(0.,1.,p.x>=7u);
    let phi=(turns-0.5)*TAU;
    // Slightly downward keeps CAVE visibly open and avoids the axis singularity.
    let dir=normalize(-0.25*up+sqrt(1.-0.0625)*(t*cos(phi)+b*sin(phi)));
    let instr=vec4u((0xff80u<<8u)|240u,(255u<<24u)|(center<<16u)|(width<<8u),0x903060a0u,(OP_SECTOR<<27u)|(7u<<24u)|(3u<<21u)|(variant<<16u)|0x7885u);
    let v=evaluate_bounds_layer(dir,instr,OP_SECTOR,up,RegionWeights(0.,1.,0.));
    textureStore(result,p.xy,vec4f(v.regions.sky,v.regions.wall,v.regions.floor,1.));
}
"#,
        14,
        9,
    );
    let mut maximum = 0.0f32;
    for row in 0..9 {
        for x in 0..7 {
            let a = pixels[row * 14 + x];
            let b = pixels[row * 14 + x + 7];
            assert!(a.iter().chain(b.iter()).all(|v| v.is_finite()));
            assert!((a[0] + a[1] + a[2] - 1.).abs() <= TOL);
            for c in 0..4 {
                let e = (a[c] - b[c]).abs();
                maximum = maximum.max(e);
                assert!(e <= TOL, "sector identity row={row} x={x}: {a:?} {b:?}");
            }
        }
        assert!(pixels[row * 14 + 3][0] > 0.8, "missing opening");
        assert!(
            pixels[row * 14][2] > 0.99 && pixels[row * 14 + 6][2] > 0.99,
            "missing outside ownership"
        );
        assert!(
            pixels[row * 14 + 1][1] > 0.4 && pixels[row * 14 + 5][1] > 0.4,
            "missing authored boundary band"
        );
    }
    println!(
        "STRUCTURAL sector divisions samples={} max_identity={maximum} tolerance={TOL}",
        pixels.len()
    );
}

#[path = "../../../examples/3-inspectors/epu-showcase/src/presets/set_01_02.rs"]
mod city_preset;

fn check_showcase_skyline(program: [[u64; 2]; 8], name: &str, legacy: bool) {
    const W: usize = 240;
    const H: usize = 135;
    let words: Vec<u32> = program
        .iter()
        .flat_map(|[hi, lo]| {
            [
                *lo as u32,
                (*lo >> 32) as u32,
                *hi as u32,
                (*hi >> 32) as u32,
            ]
        })
        .collect();
    let pixels = probe_image_input_bounds(
        r#"
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@group(0) @binding(1) var<storage,read> words:array<vec4u>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 var layers:array<vec4u,8>;for(var i=0u;i<8u;i++){layers[i]=words[i];}
 // Same unchanged initial showcase camera; this is a ray-structure check,
 // not an image-parity claim or a substitute for the real player's camera.
 let elev=15.0*PI/180.0;
 let camera=vec3f(0.0,5.0*sin(elev)+1.0,5.0*cos(elev));
 let forward=normalize(-camera);let right=normalize(cross(forward,vec3f(0.,1.,0.)));
 let up=cross(right,forward);
 let uv=vec2f((f32(p.x)+0.5)/240.0*2.0-1.0,1.0-(f32(p.y)+0.5)/135.0*2.0);
 let dir=normalize(forward+tan(30.0*PI/180.0)*(right*uv.x*(16.0/9.0)+up*uv.y));
 let value=evaluate_bounds_layer(dir,layers[2],OP_SILHOUETTE,decode_dir16(instr_dir16(layers[2])),RegionWeights(1.,0.,0.));
 let full=evaluate_epu_layers(dir,layers);layers[2]=vec4u(0u);
 let delta=abs(full-evaluate_epu_layers(dir,layers));
 textureStore(result,p.xy,vec4f(value.regions.sky,value.regions.wall,value.regions.floor,max(delta.x,max(delta.y,delta.z))));
}
"#,
        EPU_BOUNDS,
        W as u32,
        H as u32,
        &words,
    );
    assert_eq!(pixels.len(), W * H);
    assert!(pixels.iter().flatten().all(|v| v.is_finite()));
    assert!(
        pixels
            .iter()
            .all(|p| (p[0] + p[1] + p[2] - 1.).abs() <= 0.001
                && p[..3].iter().all(|v| *v >= 0. && *v <= 1.))
    );
    let counts: Vec<usize> = (0..3)
        .map(|c| pixels.iter().filter(|p| p[c] > 0.9).count())
        .collect();
    let mut roofs = Vec::new();
    for x in 0..W {
        if (0..H).any(|y| pixels[y * W + x][0] > 0.9)
            && let Some(y) = (0..H).find(|y| pixels[y * W + x][1] > 0.9)
        {
            roofs.push(y);
        }
    }
    let variation = roofs.iter().max().unwrap_or(&0) - roofs.iter().min().unwrap_or(&0);
    let influence = pixels.iter().filter(|p| p[3] > 2.0 / 255.0).count();
    let visible =
        counts.iter().all(|n| *n >= 128) && roofs.len() >= 60 && variation >= 4 && influence >= 128;
    println!(
        "{name}_STRUCTURE legacy={legacy} records={} region_counts={counts:?} skyline_columns={} roofline_row_range={variation} influenced={influence} visible={visible}",
        pixels.len(),
        roofs.len()
    );
    assert_eq!(
        visible, !legacy,
        "skyline and ground must exist in the unchanged view; human quality is separate"
    );
}

#[test]
fn showcase_grove_has_visible_skyline_and_ground() {
    for legacy in [false, true] {
        let mut program = grove_preset::PRESET_ENCHANTED_GROVE;
        if legacy {
            program[2][1] = 0xffa4d64a00ff80fd;
        }
        constants::animate_phases(&mut program, &[0, 0, 0, 0, 0, 0, 0, 1], 32);
        check_showcase_skyline(program, "GROVE", legacy);
    }
}

#[test]
fn showcase_city_has_visible_skyline_and_ground() {
    for legacy in [false, true] {
        let mut program = city_preset::PRESET_NEON_METROPOLIS;
        if legacy {
            program[2][1] = 0xffbcee8200ff80fe;
        }
        constants::animate_phases(&mut program, &[0, 0, 0, 1, 0, 0, 1, 1], 32);
        check_showcase_skyline(program, "CITY", legacy);
    }
}

#[test]
fn arcade_plane_survives_later_bounds() {
    for old_order in [false, true] {
        let mut program = arcade_preset::PRESET_NEON_ARCADE;
        let mut slot = program.iter().position(|p| p[0] >> 59 == 15).unwrap();
        if old_order {
            program.swap(slot, 2);
            slot = 2;
            program[slot][0] = (program[slot][0] & !(7u64 << 56)) | (1u64 << 56);
        }
        constants::animate_phases(&mut program, &[0, 0, 0, 0, 0, 1, 1, 1], 32);
        let mut words: Vec<u32> = program
            .iter()
            .flat_map(|[hi, lo]| {
                [
                    *lo as u32,
                    (*lo >> 32) as u32,
                    *hi as u32,
                    (*hi >> 32) as u32,
                ]
            })
            .collect();
        words.extend([slot as u32, 0, 0, 0]);
        let pixels = probe_image_input_bounds(
            r#"
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@group(0) @binding(1) var<storage,read> words:array<vec4u>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 var layers:array<vec4u,8>;for(var i=0u;i<8u;i++){layers[i]=words[i];}
 let elev=15.0*PI/180.0;let camera=vec3f(0.,5.*sin(elev)+1.,5.*cos(elev));
 let forward=normalize(-camera);let right=normalize(cross(forward,vec3f(0.,1.,0.)));let up=cross(right,forward);
 let uv=vec2f((f32(p.x)+0.5)/240.*2.-1.,1.-(f32(p.y)+0.5)/135.*2.);
 let dir=normalize(forward+tan(30.*PI/180.)*(right*uv.x*(16./9.)+up*uv.y));
 let slot=words[8].x;let facing=dot(dir,decode_dir16(instr_dir16(layers[slot])));
 let full=evaluate_epu_layers(dir,layers);layers[slot]=vec4u(0u);
 let delta=abs(full-evaluate_epu_layers(dir,layers));let difference=max(delta.x,max(delta.y,delta.z));
 textureStore(result,p.xy,vec4f(difference,facing,full.x,full.y));
}
"#,
            EPU_BOUNDS,
            240,
            135,
            &words,
        );
        assert!(pixels.iter().flatten().all(|v| v.is_finite()));
        let changed = pixels.iter().filter(|p| p[0] > 2.0 / 255.0).count();
        let outside = pixels.iter().filter(|p| p[1] <= 0.0 && p[0] != 0.0).count();
        println!(
            "ARCADE_FLOOR old_order={old_order} records={} visible_samples={changed} outside_plane_changes={outside}",
            pixels.len()
        );
        assert_eq!(outside, 0, "floor must not affect the opposite hemisphere");
        assert_eq!(
            changed >= 128,
            !old_order,
            "floor was overpainted by a later bounds source"
        );
    }
}
