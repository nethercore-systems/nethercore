//! Fixed-axis CELL chart-wrap audit. No CPU reimplementation of the evaluator.
use super::*;

// Frozen before execution; comparisons happen before rgba16float readback when
// checking normalization. Deliberate material/fill boundaries are not smoothed.
const EDGE_TOL: f32 = 0.005;
const OUTPUT_TOL: f32 = 0.01;
const NAMES: [&str; 6] = ["GRID", "HEX", "VORONOI", "RADIAL", "SHATTER", "BRICK"];
const DENSITIES: [u32; 5] = [0, 1, 127, 254, 255];

fn samples() -> Vec<[f32; 4]> {
    samples_with_direction("")
}

fn samples_with_direction(direction_probe: &str) -> Vec<[f32; 4]> {
    probe_image(
        &r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let variant=p.y/160u;
    let density_byte=array<u32,5>(0u,1u,127u,254u,255u)[(p.y/32u)%5u];
    let seed=array<u32,4>(0u,1u,127u,255u)[(p.y/8u)%4u];
    let v=array<f32,8>(.13,.21,.33,.43,.57,.67,.79,.87)[p.y%8u];
    let field=p.x%4u;
    let point=(p.x/4u)%14u;
    let control=p.x/56u;
    let fill=array<u32,3>(0u,128u,255u)[control/3u];
    let gap=array<u32,3>(0u,64u,255u)[control%3u];
    let instr=vec4u((seed<<24u)|(0x80ffu<<8u)|255u,
        (191u<<24u)|(density_byte<<16u)|(fill<<8u)|gap,
        0x4080c060u,(OP_CELL<<27u)|(REGION_ALL<<24u)|(variant<<16u)|0x2040u);
    let axis=decode_dir16(instr_dir16(instr));
    let reference_axis=select(vec3f(0.,1.,0.),vec3f(1.,0.,0.),abs(axis.y)>.9);
    let t=normalize(cross(reference_axis,axis));let b=normalize(cross(axis,t));
    var phi=(f32(point-6u)+.37)/8.*TAU-PI;
    if point<6u {
        let epsilon=array<f32,3>(.001,.0001,.00001)[point/2u];
        phi=select(-PI+epsilon,PI-epsilon,point%2u==1u);
    }
    let h=v*2.-1.;
    var dir=normalize(axis*h+sqrt(1.-h*h)*(t*cos(phi)+b*sin(phi)));
    var chart_probe=false;
    // DIRECTION_PROBE
    let uv=cell_axis_cylinder_uv(dir,axis);
    let density=cell_density(instr_a(instr));
    var info=vec3f(0.);
    switch variant {
        case 0u: {info=cell_grid(uv,density);}
        case 1u: {info=cell_hex(uv,density);}
        case 2u: {info=cell_voronoi(uv,density,f32(seed));}
        case 3u: {info=cell_radial(uv,density,axis,dir);}
        case 4u: {info=cell_shatter(uv,density,f32(seed));}
        default: {info=cell_brick(uv,density,f32(seed));}
    }
    let value=cell_hash2(info.xy,f32(seed));
    let s=evaluate_bounds_layer(dir,instr,OP_CELL,axis,RegionWeights(.2,.3,.5));
    var output=vec4f(info,value);
    if field==1u {output=vec4f(s.sample.rgb,s.sample.w);}
    if field==2u {output=vec4f(s.regions.sky,s.regions.wall,s.regions.floor,s.region_mix);}
    if field==3u {
        let sum=s.regions.sky+s.regions.wall+s.regions.floor;
        let expected_open=value>=u8_to_01(instr_b(instr));
        var open_ok=!expected_open || (s.regions.sky==1. && s.regions.wall==0. && s.regions.floor==0.);
        let gap_width=u8_to_01(instr_c(instr))*.2;
        let expected=regions_from_signed_distance(info.z-gap_width,max(.005,gap_width*.5));
        var solid_ok=expected_open || all(vec3f(s.regions.sky,s.regions.wall,s.regions.floor)==vec3f(expected.sky,expected.wall,expected.floor));
        if variant<=5u {
            // Filled polygons/sites include a neighbour's AA edge, not only the selected owner.
            // Preserve every original ray; use the fill-aware geometric field for this audit.
            var f=cell_hex_fields(uv,density,f32(seed),u8_to_01(instr_b(instr)));
            if variant==0u {f=cell_grid_fields(uv,density,f32(seed),u8_to_01(instr_b(instr)));}
            if variant==2u {f=cell_site_fields(uv,round(density),f32(seed),.8,f32(seed),u8_to_01(instr_b(instr)));}
            if variant==3u {f=cell_radial_fields(uv,density,axis,dir,f32(seed),u8_to_01(instr_b(instr)));}
            if variant==4u {f=cell_site_fields(uv,round(density),f32(seed)+42.,1.,f32(seed),u8_to_01(instr_b(instr)));}
            if variant==5u {f=cell_brick_fields(uv,density,f32(seed),u8_to_01(instr_b(instr)));}
            var expected_hex=RegionWeights(1.,0.,0.);
            if f.w > -99. {expected_hex=regions_from_signed_distance(f.w-gap_width,max(.005,gap_width*.5));}
            open_ok=true;
            solid_ok=all(vec3f(s.regions.sky,s.regions.wall,s.regions.floor)==vec3f(expected_hex.sky,expected_hex.wall,expected_hex.floor));
        }
        output=vec4f(select(0.,1.,abs(sum-1.)<=.00001),select(0.,1.,open_ok && solid_ok),uv);
        if chart_probe {
            // Preserve sub-ULP-at-one chart offsets through rgba16float readback.
            output.z=shortest_periodic_delta(uv.x,0.,1.)*1000000.;
            output.w=dot(dir,b)*1000000.;
        }
    }
    textureStore(result,p.xy,output);
}
"#.replace("// DIRECTION_PROBE", direction_probe),
        504,
        960,
    )
}

#[test]
fn cell_chart_controls_and_audit() {
    let pixels = samples();
    assert_eq!(pixels.len(), 504 * 960);
    let mut peaks = [[0.0f32; 3]; 6];
    for (row, p) in pixels.chunks_exact(504).enumerate() {
        let variant = row / 160;
        assert!(
            p.iter().flatten().all(|v| v.is_finite()),
            "nonfinite row {row}"
        );
        for c in 0..9 {
            for point in 0..14 {
                let i = c * 56 + point * 4;
                assert_eq!(p[i + 3][0], 1., "normalization row={row} control={c}");
                assert_eq!(
                    p[i + 3][1],
                    1.,
                    "expected opening ownership row={row} control={c}"
                );
                assert_eq!(p[i + 1][3], 1., "nonzero sample ownership");
                assert_eq!(p[i + 2][3], 1., "full region ownership");
                assert!(p[i + 1][..3].iter().any(|x| *x > 0.), "blank content");
                for r in 0..3 {
                    peaks[variant][r] = peaks[variant][r].max(p[i + 2][r]);
                }
                if c < 3 {
                    assert_eq!(&p[i + 2][..3], &[1., 0., 0.], "fill zero");
                }
            }
            // Raw shrinking pairs retained for independent audit, including paths
            // not certified by the deliberately narrower continuity assertion.
            println!(
                "CELL_DATA row={row} variant={} density={} seed={} v={} fill={} gap={} pairs={:?}",
                NAMES[variant],
                DENSITIES[(row / 32) % 5],
                [0, 1, 127, 255][(row / 8) % 4],
                [0.13, 0.21, 0.33, 0.43, 0.57, 0.67, 0.79, 0.87][row % 8],
                [0, 128, 255][c / 3],
                [0, 64, 255][c % 3],
                &p[c * 56..c * 56 + 24]
            );
        }
    }
    for v in 0..6 {
        println!("CELL_CONTROLS {} region_peaks={:?}", NAMES[v], peaks[v]);
        assert!(
            peaks[v].iter().all(|p| *p > 0.01),
            "missing sky/wall/floor control"
        );
    }
}

#[test]
fn cell_grid_chart_gap_continuation() {
    let pixels = samples();
    let mut failures = Vec::new();
    let mut endpoint_sides = 0;
    for (row, p) in pixels.chunks_exact(504).take(160).enumerate() {
        let density_byte = DENSITIES[(row / 32) % 5];
        let a = p[16];
        let b = p[20];
        if density_byte == 0 || density_byte == 255 {
            // The cut IS an authored GRID boundary: different owners expected.
            // Never force their hashes, fill or material values to match.
            let last = if density_byte == 0 { 3. } else { 63. };
            assert_eq!(a[0], 0.);
            assert_eq!(b[0], last);
            assert_eq!(a[1], b[1]);
            endpoint_sides += 1;
        }
        // All cells filled isolates the geometric gap from intentional fill steps.
        // At a grid boundary mortar must approach the same edge from either side.
        let delta = (a[2] - b[2]).abs();
        let mut regions = 0.0f32;
        for control in 6..9 {
            for k in 0..3 {
                regions = regions.max((p[control * 56 + 18][k] - p[control * 56 + 22][k]).abs());
            }
        }
        if delta > EDGE_TOL || regions > OUTPUT_TOL {
            failures.push(row);
            println!(
                "CELL_GRID_RED row={row} density={density_byte} edge={delta} regions={regions} shrinking_geometry={:?}",
                [p[0], p[4], p[8], p[12], p[16], p[20]]
            );
        }
    }
    println!(
        "CELL_GRID checked=160 expected_boundary_sides={endpoint_sides} failures={}",
        failures.len()
    );
    assert_eq!(endpoint_sides, 64);
    assert!(
        failures.is_empty(),
        "GRID gap continuation at fractional circumference: {failures:?}"
    );
}

// Full packed-density periodic boundary oracle; no reconstructed CELL evaluator.
#[test]
fn cell_grid_partial_boundary_oracle() {
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let d=mix(4.,64.,u8_to_01(p.x));
    let u=array<f32,9>(0.,.000001,.013,.13,.37,.71,.97,.999999,.999)[p.y%9u];
    let v=array<f32,3>(.13,.33,.57)[p.y/9u];
    let uv=vec2f(u,v);let x=u*d;let y=fract(v*d);
    var oracle=min(y,1.-y);
    for(var k=0.;k<ceil(d);k+=1.) {
        let delta=abs(x-k);oracle=min(oracle,min(delta,d-delta));
    }
    let info=cell_grid(uv,d);let f=fract(uv*d);
    let old=min(min(f.x,1.-f.x),min(f.y,1.-f.y));
    let parity=(p.x%17u!=0u && x>=floor(d)) || abs(info.z-old)<.00001;
    let owner=all(info.xy==floor(uv*d));
    let interior=cell_grid(vec2f(.5/d,.5/d),d).z;
    let left=cell_grid(vec2f((1.-.001)/d,.5/d),d).x;
    let right=cell_grid(vec2f((1.+.001)/d,.5/d),d).x;
    textureStore(result,p.xy,vec4f(abs(info.z-oracle),select(0.,1.,owner && parity),interior,select(0.,1.,left==0. && right==1.)));
}
"#,
        256,
        27,
    );
    let mut failures = 0;
    let mut peak = 0.0f32;
    for (i, p) in pixels.iter().enumerate() {
        assert!(p.iter().all(|v| v.is_finite()));
        assert_eq!(&p[1..], &[1., 0.5, 1.], "owner/interior/parity sample={i}");
        peak = peak.max(p[0]);
        if p[0] > EDGE_TOL {
            failures += 1;
        }
    }
    println!(
        "GRID_ORACLE samples={} failures={failures} peak={peak}",
        pixels.len()
    );
    assert_eq!(failures, 0);
}

#[test]
fn cell_grid_partial_cut_limits() {
    let pixels = endpoint_limit_samples();
    let mut failures = Vec::new();
    for (row, p) in pixels.chunks_exact(504).take(160).enumerate() {
        let d = 4. + 60. * DENSITIES[(row / 32) % 5] as f32 / 255.;
        let last = d.ceil() - 1.;
        for side in 0..2 {
            let offsets = [
                p[side * 4 + 3][2],
                p[(side + 2) * 4 + 3][2],
                p[(side + 4) * 4 + 3][2],
            ];
            assert!(
                offsets[0] * offsets[2] > 0.
                    && offsets[0].abs() > offsets[1].abs()
                    && offsets[1].abs() > offsets[2].abs()
            );
        }
        assert!(p[3][2] > 0. && p[7][2] < 0.);
        assert!(p[27][2].abs() < p[19][2].abs().min(p[23][2].abs()));
        let mut ok = true;
        for c in 0..9 {
            let at = |i: usize, f: usize| p[c * 56 + i * 4 + f];
            for i in 0..14 {
                let info = at(i, 0);
                // Cut is intentionally an owner boundary, not a hash match.
                if i < 6 {
                    assert_eq!(info[0], if i % 2 == 0 { 0. } else { last });
                }
                assert_eq!(&at(i, 3)[..2], &[1., 1.]);
                assert!(at(i, 1)[..3].iter().any(|x| *x > 0.));
            }
            for (a, b) in [
                (0, 1),
                (2, 3),
                (4, 5),
                (0, 6),
                (1, 6),
                (2, 6),
                (3, 6),
                (4, 6),
                (5, 6),
                (0, 7),
                (1, 8),
                (2, 9),
                (3, 10),
                (4, 11),
                (5, 12),
            ] {
                ok &= (at(a, 0)[2] - at(b, 0)[2]).abs() <= EDGE_TOL;
                // Filled controls isolate geometry; mixed-fill owners may differ.
                if c >= 6 {
                    for f in 1..3 {
                        ok &= (0..4).all(|k| (at(a, f)[k] - at(b, f)[k]).abs() <= OUTPUT_TOL);
                    }
                }
            }
        }
        if !ok {
            failures.push(row);
        }
    }
    println!("GRID_LIMIT checked=160 failures={}", failures.len());
    assert!(
        failures.is_empty(),
        "GRID bilateral/on-cut/same-side: {failures:?}"
    );
}

#[test]
fn cell_offset_identity_scope_and_authored_owners() {
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let density=mix(4.,64.,u8_to_01(p.x));
    let raw=f32(p.y)-1.;
    let id=cell_offset_id_x(raw,density);
    var expected=raw;
    if p.x%17u==0u {
        let n=i32(4u+4u*(p.x/17u));expected=f32((i32(raw)+n)%n); // raw >= -1: nonnegative independent oracle
    }
    // Exact GPU-side equality, including fractional seeds (no integer cast).
    let seed=f32(p.y)*.37;
    let hash_ok=cell_hash2(vec2f(id,3.),seed)==cell_hash2(vec2f(expected,3.),seed);
    // Actual local callers, on each side of an authored offset-cell ID edge.
    let d=select(4.,64.,p.x>=128u);let owner_seed=f32(p.y);
    let row=2.;let y=(row+.5)*0.8660254/d;
    let hex_left=cell_hex(vec2f((1.5-.001)/d,y),d);
    let hex_right=cell_hex(vec2f((1.5+.001)/d,y),d);
    // The regular half-cell stagger puts this edge at 1.5/d.
    // The retained sides bracket it without reconstructing a seeded warp.
    let brick_left=cell_brick(vec2f(1.25/d,5./d),d,owner_seed);
    let brick_right=cell_brick(vec2f(1.75/d,5./d),d,owner_seed);
    let owners=hex_left.x==1. && hex_right.x==2. && brick_left.x==1. && brick_right.x==2.;
    textureStore(result,p.xy,vec4f(select(0.,1.,id==expected),select(0.,1.,hash_ok),select(0.,1.,owners),select(0.,1.,density==floor(density))));
}
"#,
        256,
        68,
    );
    assert_eq!(pixels.len(), 256 * 68);
    let mut fractional = 0;
    for (i, p) in pixels.into_iter().enumerate() {
        assert_eq!(
            &p[..3],
            &[1., 1., 1.],
            "identity scope or authored owners density={} seed-index={}",
            i % 256,
            i / 256
        );
        if p[3] == 0. {
            fractional += 1;
        }
    }
    assert!(fractional > 0);
    println!(
        "CELL_ID_SCOPE all 256 packed densities, raw IDs -1..66, fractional cases={fractional}; authored HEX/BRICK owners 1/2 retained"
    );
}

// Frozen from focused-03 RED, not reselected using repaired identity/geometry.
const ENDPOINT_ROWS: &[usize] = &[
    160, 161, 164, 165, 168, 169, 172, 173, 176, 177, 180, 181, 184, 185, 188, 189, 288, 292, 293,
    294, 296, 300, 301, 302, 304, 308, 309, 310, 312, 316, 317, 318, 800, 801, 802, 803, 804, 805,
    806, 807, 808, 809, 810, 811, 816, 817, 818, 819, 820, 821, 822, 823, 824, 825, 826, 827, 828,
    829, 830, 928, 929, 930, 931, 932, 933, 935, 936, 937, 938, 939, 940, 941, 944, 945, 946, 948,
    950, 951, 952, 953, 954, 956, 957, 958, 959,
];

// Independent identity/edge contract for the declared regular staggered grid.
// Every historical ray is retained; unshifted rows have a real mortar edge at u=0.
fn assert_brick_cut_sample(row: usize, info: [f32; 4], positive_side: bool) {
    let density = 4.0 + (DENSITIES[(row / 32) % 5] as f32 / 255.0) * 60.0;
    let count = density.round();
    let v = [0.13f32, 0.21, 0.33, 0.43, 0.57, 0.67, 0.79, 0.87][row % 8];
    let y = v * count * 0.5;
    let owner_y = y.floor();
    let shifted = (owner_y as u32).is_multiple_of(2);
    let owner_x = if shifted || positive_side {
        0.0
    } else {
        count - 1.0
    };
    assert_eq!(
        &info[..2],
        &[owner_x, owner_y],
        "regular BRICK owner row={row}"
    );
    let edge = if shifted {
        0.5f32.min(y.fract()).min(1.0 - y.fract())
    } else {
        0.0
    };
    assert!(
        info[2] >= -EDGE_TOL && (info[2] - edge).abs() <= EDGE_TOL,
        "regular BRICK geometry row={row}: got={} expected={edge}",
        info[2]
    );
}

#[test]
fn cell_hex_brick_endpoint_identity_geometry() {
    let pixels = samples();
    for &row in ENDPOINT_ROWS {
        let p = &pixels[row * 504..(row + 1) * 504];
        let a = p[16];
        let b = p[20];
        assert!((a[2] - b[2]).abs() <= EDGE_TOL, "geometry row={row}");
        if row / 160 == 1 {
            // These fixed rays may now cross a real edge of the regular HEX lattice.
            assert!(a[2] >= -EDGE_TOL && b[2] >= -EDGE_TOL);
            if a[..2] == b[..2] {
                assert_eq!(a[3], b[3]);
            } else {
                assert!(a[2].abs() <= EDGE_TOL && b[2].abs() <= EDGE_TOL);
            }
        } else {
            assert_brick_cut_sample(row, a, p[19][2] < 0.5);
            assert_brick_cut_sample(row, b, p[23][2] < 0.5);
            if a[..2] == b[..2] {
                assert_eq!(a[3], b[3], "canonical hash row={row}");
            }
        }
    }
    println!(
        "CELL_ENDPOINT_IDENTITY_GEOMETRY 85 frozen witnesses; full color acceptance remains a separate mandatory test"
    );
}

#[test]
fn cell_hex_brick_endpoint_continuity() {
    same_periodic_cell_continuity(true);
}

fn endpoint_limit_samples() -> Vec<[f32; 4]> {
    // Rotate about the cut directly; PI +/- epsilon loses small f32 offsets.
    // Three bilateral pairs, on-cut, then equal-distance same-side controls.
    samples_with_direction(
        r#"
    let delta=array<f32,14>(.00001,-.00001,.000003,-.000003,.000001,-.000001,
        0.,.00002,-.00002,.000006,-.000006,.000002,-.000002,0.)[point];
    dir=normalize(axis*h+sqrt(1.-h*h)*(-t*cos(delta)-b*sin(delta)));
    chart_probe=true;
    "#,
    )
}

fn endpoint_output_converges(row: usize, pixels: &[[f32; 4]]) -> bool {
    let p = &pixels[row * 504..(row + 1) * 504];
    let distance = |a: [f32; 4], b: [f32; 4]| {
        a.into_iter()
            .zip(b)
            .map(|(x, y)| (x - y).abs())
            .fold(0.0f32, f32::max)
    };
    assert!(p.iter().flatten().all(|v| v.is_finite()));
    for side in 0..2 {
        // Each direction AND resulting chart coordinate must still be distinct,
        // on the intended side, and approach the cut (not a rounding plateau).
        for k in 2..4 {
            let offsets = [
                p[side * 4 + 3][k],
                p[(2 + side) * 4 + 3][k],
                p[(4 + side) * 4 + 3][k],
            ];
            assert!(
                offsets[0] * offsets[2] > 0.
                    && offsets[0].abs() > offsets[1].abs()
                    && offsets[1].abs() > offsets[2].abs(),
                "unresolved offsets row={row}: {offsets:?}"
            );
        }
    }
    assert!(p[3][2] > 0. && p[7][2] < 0.);
    assert!(
        p[27][2].abs() < p[19][2].abs().min(p[23][2].abs()),
        "on-cut row={row}"
    );
    let mut ok = true;
    for c in 0..9 {
        let at = |point: usize, field: usize| p[c * 56 + point * 4 + field];
        for point in 0..14 {
            let info = at(point, 0);
            if row / 160 == 1 {
                assert!(info[2] >= -EDGE_TOL && (info[2] - at(6, 0)[2]).abs() <= EDGE_TOL);
                if info[..2] == at(6, 0)[..2] {
                    assert_eq!(info[3], at(6, 0)[3]);
                } else {
                    assert!(info[2].abs() <= EDGE_TOL && at(6, 0)[2].abs() <= EDGE_TOL);
                }
            } else if row / 160 == 5 {
                assert_brick_cut_sample(row, info, at(point, 3)[2] >= 0.0);
                let same_owner = if info[..2] == at(6, 0)[..2] { 6 } else { 1 };
                assert_eq!(&info[..2], &at(same_owner, 0)[..2]);
                assert_eq!(
                    info[3],
                    at(same_owner, 0)[3],
                    "canonical BRICK hash row={row}"
                );
            } else {
                assert_eq!(&info[..2], &at(6, 0)[..2], "limit owner row={row}");
                assert_eq!(info[3], at(6, 0)[3], "limit hash row={row}");
                // The old >.02 margin selected rays before the hash correction; keep every ray.
                assert!(info[2] > 0.0 && (info[2] - at(6, 0)[2]).abs() <= EDGE_TOL);
            }
            assert_eq!(&at(point, 3)[..2], &[1., 1.], "limit ownership row={row}");
            assert!(at(point, 1)[..3].iter().any(|x| *x > 0.));
        }
        for field in 1..3 {
            let bilateral = [0, 2, 4].map(|i| distance(at(i, field), at(i + 1, field)));
            let on_cut = [0, 2, 4].map(|i| {
                distance(at(i, field), at(6, field)).max(distance(at(i + 1, field), at(6, field)))
            });
            let same_side = [(0, 7), (2, 9), (4, 11)].map(|(i, j)| {
                distance(at(i, field), at(j, field)) + distance(at(i + 1, field), at(j + 1, field))
            });
            // A finite-distance gradient must contract at both smaller scales,
            // reach the unchanged budget on both sides AND at the cut, and be
            // accounted for by equal-distance same-side gradients. A constant
            // jump above budget cannot pass simply by using a smaller epsilon.
            ok &= bilateral[2] <= OUTPUT_TOL && on_cut[2] <= OUTPUT_TOL;
            if bilateral[0] > OUTPUT_TOL {
                ok &= bilateral[1] <= bilateral[0] * 0.5 && bilateral[2] <= bilateral[1] * 0.5;
                ok &= (0..3).all(|i| bilateral[i] <= same_side[i] + OUTPUT_TOL);
                println!(
                    "CELL_GRADIENT row={row} control={c} field={field} bilateral={bilateral:?} on_cut={on_cut:?} same_side={same_side:?}"
                );
            }
        }
    }
    ok
}

// All original focused-03 VORONOI/SHATTER REDs are endpoint densities;
// no fractional row met that original same-owner positive-interior predicate.
const CANDIDATE_ROWS: &[usize] = &[
    320, 321, 325, 326, 332, 334, 335, 336, 341, 345, 348, 349, 350, 351, 459, 476, 642, 644, 652,
    655, 656, 658, 668, 773, 775, 799,
];

#[test]
fn cell_candidate_frozen_continuity() {
    let pixels = endpoint_limit_samples();
    let mut failures = Vec::new();
    for &row in CANDIDATE_ROWS {
        println!(
            "CELL_CANDIDATE row={row} geometry={:?}",
            (0..7)
                .map(|i| pixels[row * 504 + i * 4])
                .collect::<Vec<_>>()
        );
        // The frozen >.02 criterion selected ORIGINAL witnesses, not a
        // promise that repaired sites retain their old distance or owner.
        // Keep every row; assert positive interior and exact same-owner/hash
        // across the repaired chart, without touching the HEX/BRICK gate.
        let p = &pixels[row * 504..(row + 1) * 504];
        for side in 0..2 {
            for field in 2..4 {
                let offsets = [
                    p[side * 4 + 3][field],
                    p[(side + 2) * 4 + 3][field],
                    p[(side + 4) * 4 + 3][field],
                ];
                assert!(
                    offsets[0] * offsets[2] > 0.
                        && offsets[0].abs() > offsets[1].abs()
                        && offsets[1].abs() > offsets[2].abs()
                );
            }
        }
        let mut ok = true;
        for c in 0..9 {
            let at = |i: usize, f: usize| p[c * 56 + i * 4 + f];
            for i in 0..14 {
                let info = at(i, 0);
                assert!(info[2] > 0., "repaired positive interior row={row}");
                ok &= info[..2] == at(6, 0)[..2] && info[3] == at(6, 0)[3];
                ok &= (info[2] - at(6, 0)[2]).abs() <= EDGE_TOL;
                assert_eq!(&at(i, 3)[..2], &[1., 1.]);
                assert!(at(i, 1)[..3].iter().any(|x| *x > 0.));
            }
        }
        // A finite-width gradient is not a seam. Keep all original rays and
        // budgets; require contraction, both on-cut limits and same-side controls.
        ok &= endpoint_output_converges(row, &pixels);
        if !ok {
            failures.push(row);
        }
    }
    assert!(
        failures.is_empty(),
        "frozen candidate discontinuities: {failures:?}"
    );
}

#[test]
fn cell_candidate_frozen_continuity_rejects_seam() {
    let mut pixels = endpoint_limit_samples();
    for &row in CANDIDATE_ROWS {
        assert!(endpoint_output_converges(row, &pixels));
        for control in 0..9 {
            // Include same-side controls: a real step must not pass as a gradient.
            for point in [0, 2, 4, 7, 9, 11] {
                pixels[row * 504 + control * 56 + point * 4 + 1][0] += 0.03125;
            }
        }
        assert!(
            !endpoint_output_converges(row, &pixels),
            "injected candidate seam row={row}"
        );
    }
}

#[test]
fn cell_chart_same_periodic_cell_continuity() {
    same_periodic_cell_continuity(false);
}

#[test]
fn cell_voronoi_residual_limits_reject_jump() {
    let pixels = endpoint_limit_samples();
    // Frozen residuals: integral density 64, seeds 0/1, v=.43/.33.
    // The same-owner positive-interior domain matches the endpoint limit gate.
    for row in [451, 458] {
        assert!(endpoint_output_converges(row, &pixels));
        for points in [&[1usize, 3, 5, 8, 10, 12][..], &[6usize][..]] {
            let mut jumped = pixels.clone();
            for &point in points {
                jumped[row * 504 + 7 * 56 + point * 4 + 2][0] += 0.02;
            }
            assert!(!endpoint_output_converges(row, &jumped));
        }
    }
}

#[test]
fn cell_endpoint_convergence_rejects_jump() {
    let pixels = endpoint_limit_samples();
    assert!(endpoint_output_converges(935, &pixels));
    // Deliberate negative controls on actual GPU samples, not shader changes:
    // a one-sided finite jump and an isolated on-cut mismatch must both fail.
    for points in [&[1usize, 3, 5, 8, 10, 12][..], &[6usize][..]] {
        let mut jumped = pixels.clone();
        for &point in points {
            jumped[935 * 504 + 7 * 56 + point * 4 + 2][0] += 0.02;
        }
        assert!(!endpoint_output_converges(935, &jumped));
    }
}

fn same_periodic_cell_continuity(endpoint_only: bool) {
    let pixels = samples();
    let mut limits = None;
    let mut failures = Vec::new();
    let mut checked = [0usize; 6];
    for (row, p) in pixels.chunks_exact(504).enumerate() {
        let variant = row / 160;
        let a = p[16];
        let b = p[20];
        // Keep every originally applicable witness, including relief failures.
        let same_periodic = ENDPOINT_ROWS.contains(&row);
        // Voronoi/shatter with the same returned owner and positive interior:
        // candidate churn cannot justify a finite jump inside that owner.
        let same_candidate =
            (variant == 2 || variant == 4) && a[..2] == b[..2] && a[2] > 0.02 && b[2] > 0.02;
        if variant != 1 && !same_periodic && (endpoint_only || !same_candidate) {
            continue;
        }
        checked[variant] += 1;
        if variant == 1 || variant == 5 {
            if !endpoint_output_converges(row, limits.get_or_insert_with(endpoint_limit_samples)) {
                failures.push((
                    row,
                    variant,
                    (a[2] - b[2]).abs(),
                    (a[3] - b[3]).abs(),
                    f32::INFINITY,
                ));
            }
            continue;
        }
        let edge = (a[2] - b[2]).abs();
        let hash = (a[3] - b[3]).abs();
        let mut err = 0.0f32;
        for c in 0..9 {
            for field in 1..3 {
                let x = p[c * 56 + 16 + field];
                let y = p[c * 56 + 20 + field];
                for k in 0..4 {
                    err = err.max((x[k] - y[k]).abs());
                }
            }
        }
        // Keep identity and geometry checks. Every packed density now selects a
        // whole periodic lattice; classify steep gradients with the same existing
        // shrinking-ray discriminator and unchanged final error budget.
        let periodic_candidate = same_candidate;
        let output_ok = err <= OUTPUT_TOL
            || ((same_periodic || periodic_candidate)
                && edge <= EDGE_TOL
                && hash == 0.
                && a[..2] == b[..2]
                && endpoint_output_converges(
                    row,
                    limits.get_or_insert_with(endpoint_limit_samples),
                ));
        if edge > EDGE_TOL || hash != 0. || !output_ok || (same_periodic && a[..2] != b[..2]) {
            failures.push((row, variant, edge, hash, err));
            println!(
                "CELL_RED row={row} variant={} edge={edge} hash={hash} output={err} shrinking_geometry={:?}",
                NAMES[variant],
                [p[0], p[4], p[8], p[12], p[16], p[20]]
            );
        }
    }
    println!(
        "CELL_SAME_CELL checked={checked:?} failures={} edge_tolerance={EDGE_TOL} output_tolerance={OUTPUT_TOL}",
        failures.len()
    );
    assert_eq!(checked[1], 160); // Every HEX row, not only the old same-owner subset.
    assert_eq!(checked[5], 53);
    if !endpoint_only {
        assert!(
            checked[2] > 0 && checked[4] > 0,
            "fixture missed a priority path"
        );
    }
    assert!(
        failures.is_empty(),
        "CELL actual chart-cut same-cell continuity RED: {failures:?}"
    );
}
