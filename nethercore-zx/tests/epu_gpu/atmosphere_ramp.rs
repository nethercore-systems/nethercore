use super::*;

// Runtime-fed words exercise the shared evaluators. No engine change is justified by a passing audit.
#[test]
fn atmosphere_finite_variants_gain_masks_and_inactive_storage() {
    let mut inputs = Vec::new();
    for variant in 0u32..8 {
        for domain in 0..4 {
            for alpha in [0, 7, 15] {
                for gain in 0..256 {
                    inputs.extend([
                        (128 << 24) | (0x8080 << 8) | (alpha << 4) | 13,
                        (gain << 24) | (128 << 16) | (128 << 8) | 192,
                        (0x66 << 24) | 0x993355,
                        (14 << 27) | (7 << 24) | (((domain << 3) | variant) << 16) | 0x3366,
                    ]);
                }
            }
        }
    }
    let body = r#"
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@group(0) @binding(1) var<storage,read> inputs:array<vec4u>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let i=inputs[p.y*256u+p.x];let v=instr_variant_id(i);
 let sun=decode_dir16(0x8080u);let up=decode_dir16(0x80ffu);
 let ray=array<vec3f,4>(sun,-sun,up,-up)[(p.y/3u)%4u];
 let ca=vec3f(51.,102.,102.)/255.;let cb=vec3f(153.,51.,85.)/255.;
 let t=pow(clamp((dot(ray,up)-(2.*128./255.-1.)+1.)*.5,0.,1.),.5+7.5*128./255.);
 let mie=pow(max(0.,dot(ray,sun)),4.+124.*128./255.)*(2.*192./255.);
 let gain=f32(p.x)/255.;let alpha=instr_alpha_a_f32(i);let region=.375;
 var rgb=vec3f(0.);var weight=0.;
 switch v {
  case 0u:{rgb=mix(cb,ca,t);weight=(1.-t)*gain;}
  case 1u:{rgb=mix(cb,ca,t);weight=gain;}
  case 2u:{rgb=ca;weight=mie*gain;}
  case 3u:{rgb=mix(cb,ca,t)+ca*mie*.5;weight=gain;}
  case 4u:{rgb=mix(cb,ca,sin(t*PI)*.5+.5);weight=gain;}
  default:{}
 }
 let expected=rgb*weight*alpha*region;
 let value=eval_atmosphere(ray,i,up,region);
 var actual=value.rgb*value.w; // injected-gain-control
 var changed=i;changed.x^=15u; // Color B alpha is unused everywhere.
 if v==2u {changed.y^=0x00ffff00u;changed.z^=0x00ffffffu;} // MIE ignores gradient controls/Color B.
 if v!=2u && v!=3u {changed.y^=255u;changed.x^=0xffffff00u;} // No Mie fields/direction/phase.
 let same=eval_atmosphere(ray,changed,up,region);
 let hidden=eval_atmosphere(ray,i,up,0.);
 let error=max(max(abs(actual.x-expected.x),abs(actual.y-expected.y)),abs(actual.z-expected.z));
 let ignored=max(max(abs(value.rgb.x*value.w-same.rgb.x*same.w),abs(value.rgb.y*value.w-same.rgb.y*same.w)),abs(value.rgb.z*value.w-same.rgb.z*same.w));
 textureStore(result,vec2i(p.xy),vec4f(error,ignored,hidden.w,length(expected)));
}
"#;
    for negative in [false, true] {
        let source = if negative {
            body.replace("// injected-gain-control", "actual *= gain;")
        } else {
            body.to_string()
        };
        let pixels = probe_image_input_bounds(&source, EPU_BOUNDS, 256, 96, &inputs);
        assert!(pixels.iter().flatten().all(|x| x.is_finite()));
        let failures = pixels
            .iter()
            .filter(|r| r[0] > 0.002 || r[1] > 0.002 || r[2] != 0.)
            .count();
        let nonzero = pixels.iter().filter(|r| r[3] > 0.002).count();
        println!(
            "ATMOSPHERE_FINITE negative={negative} records={} failures={failures} nonzero={nonzero} limit=.002",
            pixels.len()
        );
        assert!(nonzero > 1000, "non-vacuous supported variants");
        assert_eq!(failures == 0, !negative);
    }
}

#[test]
fn ramp_all_threshold_pairs_softness_and_axes_partition() {
    let mut inputs = Vec::new();
    for direction in [0x80ffu32, 0x8080, 0x2088] {
        for softness in [0, 85, 170, 255] {
            for thresholds in 0..256 {
                inputs.extend([
                    (thresholds << 24) | (direction << 8) | 0x7d,
                    (softness << 24) | 0x336699,
                    (0xaa << 24) | 0x113355,
                    (1 << 27) | (7 << 24) | (3 << 21) | 0x6644,
                ]);
            }
        }
    }
    let body = r#"
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@group(0) @binding(1) var<storage,read> inputs:array<vec4u>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let i=inputs[p.y];let up=decode_dir16(instr_dir16(i));
 let tangent=normalize(cross(up,vec3f(.2,.3,.9)));
 let y=f32(p.x)/128.-1.;let dir=normalize(up*y+tangent*sqrt(max(0.,1.-y*y)));
 let v=eval_ramp(dir,i);var w=vec3f(v.regions.sky,v.regions.wall,v.regions.floor); // injected-region-control
 let expected=vec3f(102.,68.,170.)/255.*w.x+vec3f(51.,102.,153.)/255.*w.y+vec3f(17.,51.,85.)/255.*w.z;
 var opposite=i;opposite.x=(i.x&0x00ffffffu)|(((instr_d(i)&15u)<<4u|instr_d(i)>>4u)<<24u);
 let swapped=eval_ramp(dir,opposite);
 let delta=abs(expected-v.sample.rgb);let order=abs(v.sample.rgb-swapped.sample.rgb);
 textureStore(result,vec2i(p.xy),vec4f(max(max(-min(w.x,min(w.y,w.z)),abs(w.x+w.y+w.z-1.)),abs(v.region_mix-1.)),max(max(delta.x,delta.y),delta.z),max(max(order.x,order.y),order.z),abs(v.sample.w-7./15.)));
}
"#;
    for negative in [false, true] {
        let source = if negative {
            body.replace("// injected-region-control", "w *= 0.5;")
        } else {
            body.to_string()
        };
        let pixels = probe_image_input_bounds(&source, EPU_BOUNDS, 257, 3072, &inputs);
        assert!(pixels.iter().flatten().all(|x| x.is_finite()));
        let failures = pixels
            .iter()
            .filter(|r| r.iter().any(|x| *x > 0.002))
            .count();
        println!(
            "RAMP_FINITE negative={negative} records={} failures={failures} limit=.002",
            pixels.len()
        );
        assert_eq!(failures == 0, !negative);
    }
}
