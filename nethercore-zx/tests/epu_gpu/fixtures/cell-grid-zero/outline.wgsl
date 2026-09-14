// Diagnostic: physical rays at ordinary owner transitions, not just chart cuts.
fn support_dir(uv:vec2f,axis:vec3f)->vec3f {
    let ref_vec=select(vec3f(0.,1.,0.),vec3f(1.,0.,0.),abs(axis.y)>.9);
    let t=normalize(cross(ref_vec,axis));let b=normalize(cross(axis,t));
    let h=uv.y*2.-1.;let phi=(uv.x-.5)*TAU;
    return normalize(axis*h+sqrt(max(0.,1.-h*h))*(t*cos(phi)+b*sin(phi)));
}
fn support_info(dir:vec3f,i:vec4u)->vec3f {
    let axis=decode_dir16(instr_dir16(i));let uv=cell_axis_cylinder_uv(dir,axis);
    let density=mix(4.,64.,u8_to_01(instr_a(i)));let seed=f32(instr_d(i));
    switch instr_variant_id(i) {
        case 0u:{return cell_grid(uv,density);}
        case 1u:{return cell_hex(uv,density);}
        case 2u:{return cell_voronoi(uv,density,seed);}
        case 3u:{return cell_radial(uv,density,axis,dir);}
        case 4u:{return cell_shatter(uv,density,seed);}
        default:{return cell_brick(uv,density,seed);}
    }
}

fn site_max(v:vec3f)->f32 { return max(v.x,max(v.y,v.z)); }
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u){
 let variant=p.y/16u;let row=p.y%16u;let seed=array<u32,4>(0u,1u,127u,255u)[row/4u];
 let i=vec4u((seed<<24u)|(0xff80u<<8u)|255u,(p.x<<16u)|(128u<<8u),
  (0x58u<<24u)|0xc09050u,(OP_CELL<<27u)|(7u<<24u)|(3u<<21u)|(variant<<16u)|0x1828u);
 let axis=decode_dir16(instr_dir16(i));let dir=support_dir(vec2f((f32(p.x)+.137)/256.,array<f32,4>(.13,.37,.67,.89)[row%4u]),axis);
 var bright=i;bright.y|=255u<<24u;
 let q=eval_cell(dir,i,RegionWeights(.2,.3,.5));let b=eval_cell(dir,bright,RegionWeights(.2,.3,.5));
 textureStore(result,p.xy,vec4f(site_max(abs(q.sample.rgb-b.sample.rgb)),site_max(abs(vec3f(q.regions.sky,q.regions.wall,q.regions.floor)-vec3f(b.regions.sky,b.regions.wall,b.regions.floor))),q.sample.w,site_max(q.sample.rgb)));
}
