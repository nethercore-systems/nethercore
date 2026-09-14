//! Opt-in point modulation: accepted full-range motion, legacy encoding preserved.
use super::*;

#[test]
fn phased_scatter_dispatch_phase_and_legacy_controls() {
    // Raw opcode 0x18 deliberately makes the pre-integration RED behavioral, not a compile error.
    let pixels = probe_image(
        r#"
fn fixture(op:u32,phase:u32,depth:u32)->vec4u {
 return vec4u((53u<<24u)|(0x8080u<<8u)|0xf0u|depth,
  (255u<<24u)|(15u<<16u)|(phase&255u),0xffffffffu,(op<<27u)|(7u<<24u)|65535u);
}
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let x=f32(i32(p.y%8u)-4);let y=f32(i32(p.y/8u)-4);
 let cell=vec3f(x,y,floor(sqrt(256.-x*x-y*y)));
 let h=hash3(cell+vec3f(53.));let dir=normalize(cell+h.xyz-.5);
 let regions=RegionWeights(1.,0.,0.);let axis=vec3f(0.,1.,0.);
 let a=evaluate_layer(dir,fixture(24u,p.x,15u),axis,regions);
 let legacy=evaluate_layer(dir,fixture(10u,0u,15u),axis,regions);
 let held=evaluate_layer(dir,fixture(24u,p.x,0u),axis,regions);
 let reserved=evaluate_layer(dir,fixture(31u,p.x,15u),axis,regions);
 textureStore(result,p.xy,vec4f(a.w,legacy.w,held.w,reserved.w));
}
"#,
        257,
        64,
    );
    assert_eq!(pixels.len(), 257 * 64);
    assert!(pixels.iter().flatten().all(|v| v.is_finite()));
    let maximum = pixels.iter().map(|p| p[0]).fold(0.0f32, f32::max);
    println!("PHASED_SCATTER dispatch maximum={maximum}; points=64 phases=257");
    assert!(
        maximum >= 0.999,
        "new instruction must emit the authored point field"
    );
    let mut peaks = std::collections::BTreeSet::new();
    for row in pixels.chunks_exact(257) {
        let lo = row[..256]
            .iter()
            .map(|p| p[0])
            .fold(f32::INFINITY, f32::min);
        let hi = row[..256].iter().map(|p| p[0]).fold(0.0f32, f32::max);
        assert!(
            hi - lo > 0.35,
            "phase must vary individual point brightness"
        );
        peaks.insert(row[..256].iter().position(|p| p[0] == hi).unwrap());
        assert_eq!(row[0], row[256], "wrapped instruction is identical");
        for p in row {
            assert!(p[1] >= 0.999, "legacy point core is retained");
            assert_eq!(p[1], p[2], "zero depth is the unmodulated legacy field");
            assert_eq!(p[3], 0.0, "reserved opcode31 remains no-output");
        }
    }
    assert!(
        peaks.len() >= 16,
        "independent point phases, not one global flash"
    );
}

#[test]
fn phased_scatter_full_range_at_accepted_stride() {
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let h=hash3(vec3f(f32(p.x%32u)-16.,f32(p.x/32u)-15.,20.)+vec3f(53.));
 let phase=f32((p.y*4u)&255u)/256.;
 let global=smoothstep(.05,.95,.5+.5*cos(TAU*phase));
 textureStore(result,p.xy,vec4f(scatter_phased_mod(h,phase),global,1.+floor(h.z*4.),1.));
}
"#,
        960,
        65,
    );
    let mut peaks = std::collections::BTreeSet::new();
    let mut rates = std::collections::BTreeSet::new();
    for point in 0..960 {
        let values: Vec<_> = (0..64)
            .map(|frame| pixels[frame * 960 + point][0])
            .collect();
        assert_eq!(values.iter().copied().fold(f32::INFINITY, f32::min), 0.0);
        assert_eq!(values.iter().copied().fold(0.0f32, f32::max), 1.0);
        peaks.insert(values.iter().position(|&v| v == 1.0).unwrap());
        rates.insert(pixels[point][2] as u32);
        assert_eq!(pixels[point], pixels[64 * 960 + point]);
    }
    assert_eq!(rates, std::collections::BTreeSet::from([1, 2, 3, 4]));
    assert!(peaks.len() >= 16);
    let swing = |channel: usize| {
        let sums: Vec<f32> = pixels
            .chunks_exact(960)
            .take(64)
            .map(|row| row.iter().map(|p| p[channel]).sum())
            .collect();
        (sums.iter().copied().fold(0.0f32, f32::max)
            - sums.iter().copied().fold(f32::INFINITY, f32::min))
            / (sums.iter().sum::<f32>() / 64.0)
    };
    println!(
        "PHASED_SCATTER 960 points reach exact off/on; peak bins={}; field swing={} global control={}",
        peaks.len(),
        swing(0),
        swing(1)
    );
    assert!(swing(0) <= 0.15);
    assert!(
        swing(1) > 0.15,
        "global-flash control must fail the same discriminator"
    );
}

#[test]
fn phased_scatter_angular_wrap_all_shapes() {
    // Retain the existing angular-wrap rays/budget, extending shape and guest phase.
    let pixels = probe_image(
        r#"
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let case_id=p.y%64u;let domain=1u+(p.y/64u)%2u;
 let variant=(p.y/128u)%8u;let phase=(p.y/1024u)*63u;
 let axis_bits=select(0xff80u,0x80ffu,domain==2u);let axis=decode_dir16(axis_bits);
 var right=cross(axis,vec3f(0.,0.,1.));if length(right)<.01{right=cross(axis,vec3f(1.,0.,0.));}
 right=normalize(right);let forward=cross(right,axis);
 let height=mix(-.75,.75,f32(case_id%4u)/3.);
 let eps=select(.0001,.000001,p.x>=2u);
 var phi=select(-PI+eps,PI-eps,(p.x&1u)==1u);
 if p.x==4u{phi=-PI;}
 let dir=normalize(axis*height+sqrt(1.-height*height)*(right*cos(phi)+forward*sin(phi)));
 let instr=vec4u(((case_id/4u)<<24u)|(axis_bits<<8u)|255u,
 (255u<<24u)|((4u+case_id%5u)<<16u)|((96u+(case_id*29u)%96u)<<8u)|phase,
 0xffffffffu,(OP_SCATTER_PHASED<<27u)|(7u<<24u)|(((domain<<3u)|variant)<<16u)|65535u);
 let sample=evaluate_layer(dir,instr,axis,RegionWeights(1.,0.,0.));
 let value=sample.rgb.x*epu_saturate(sample.w);
 textureStore(result,p.xy,vec4f(value,value+select(0.,.02,(p.x&1u)==1u),0.,1.));
}
"#,
        5,
        4096,
    );
    let mut maxima = [0.0f32; 3];
    let mut energy = [0.0f32; 7];
    for (i, row) in pixels.chunks_exact(5).enumerate() {
        assert!(row.iter().flatten().all(|v| v.is_finite()));
        let variant = (i / 128) % 8;
        if variant == 7 {
            assert!(row.iter().all(|p| p[0] == 0.0));
            continue;
        }
        energy[variant] += row.iter().map(|p| p[0]).sum::<f32>();
        let coarse = (row[0][0] - row[1][0]).abs();
        let fine = (row[2][0] - row[3][0]).abs();
        let on_cut = (row[4][0] - row[2][0])
            .abs()
            .max((row[4][0] - row[3][0]).abs());
        for (m, v) in maxima.iter_mut().zip([coarse, fine, on_cut]) {
            *m = m.max(v);
        }
        assert!(
            fine < 0.005 && on_cut < 0.005,
            "row {i}: fine={fine} on-cut={on_cut}"
        );
        assert!(
            fine <= coarse + 0.001,
            "row {i}: no contraction toward quantization floor"
        );
        assert!(
            (row[2][1] - row[3][1]).abs() > 0.005,
            "injected jump must fail"
        );
    }
    assert!(energy.iter().all(|&v| v > 0.01));
    println!(
        "PHASED_SCATTER angular domains x 8 variants x 4 phases: max coarse/fine/on-cut={maxima:?}; energy={energy:?}"
    );
}
