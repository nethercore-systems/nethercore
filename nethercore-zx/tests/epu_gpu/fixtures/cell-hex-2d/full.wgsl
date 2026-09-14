@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let byte=array<u32,3>(1u,2u,0u)[(p.y%54u)%3u];let seed=((p.y%54u)/3u)%3u*127u+1u;
 let fill=array<u32,3>(255u,128u,0u)[((p.y%54u)/9u)%3u];let gap=array<u32,2>(0u,32u)[(p.y%54u)/27u];
 let density=mix(4.,64.,u8_to_01(byte));
 let point=p.x/3u;let field=p.x%3u;
 let delta=array<f32,8>(.0001,-.0001,.00001,-.00001,0.,.00002,-.00002,1.5)[point];
 let phase=array<f32,4>(.5,.99,0.,.0001)[p.y/54u];
 let v=(2.+phase)/density;
 let instr=vec4u((seed<<24u)|(0x80ffu<<8u)|255u,(191u<<24u)|(byte<<16u)|(fill<<8u)|gap,0x4080c060u,(OP_CELL<<27u)|(REGION_ALL<<24u)|(1u<<16u)|0x2040u);
 let axis=decode_dir16(instr_dir16(instr));let reference_axis=select(vec3f(0.,1.,0.),vec3f(1.,0.,0.),abs(axis.y)>.9);
 let t=normalize(cross(reference_axis,axis));let b=normalize(cross(axis,t));let h=v*2.-1.;
 let dir=normalize(axis*h+sqrt(1.-h*h)*(-t*cos(delta)-b*sin(delta)));
 let uv=cell_axis_cylinder_uv(dir,axis);let info=cell_hex_2d(uv,density);
 let value=eval_cell_2d(dir,instr,RegionWeights(.2,.3,.5));
 var out=vec4f(info,cell_hash2(info.xy,f32(seed)));
 let old=eval_cell(dir,instr,RegionWeights(.2,.3,.5));
 let integral_ok=byte!=0u || (all(value.sample.rgb==old.sample.rgb) && value.regions.sky==old.regions.sky && value.regions.wall==old.regions.wall && value.regions.floor==old.regions.floor);
 if field==1u {out=vec4f(value.regions.sky,value.regions.wall,value.regions.floor,select(0.,1.,instr_d(instr)==seed && instr_variant_id(instr)==1u && integral_ok));}
 if field==2u {out=vec4f(value.sample.rgb,shortest_periodic_delta(uv.x,0.,1.)*1000000.);}
 textureStore(result,p.xy,out);
}