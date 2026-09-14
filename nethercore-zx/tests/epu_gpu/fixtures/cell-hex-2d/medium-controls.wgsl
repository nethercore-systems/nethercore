@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let family=p.y/7u;let step=p.y%7u;
 let e=array<f32,7>(.0001,-.0001,.00002,-.00002,.00001,-.00001,0.)[step];
 let byte=select(1u,0u,family==4u);let density=mix(4.,64.,u8_to_01(byte));
 var xy=vec2f(e,2.5);
 if family==1u {xy=vec2f(1.5+e,2.5);}
 if family==2u {xy=vec2f(.05,3.+e);}
 if family==3u {xy=vec2f(1.49+e,2.1);}
 if family==4u {xy=vec2f(e,2.5);}
 if family==5u {xy=vec2f(1.+e,2.5);}
 if family==6u {xy=vec2f(1.5,3.+e);}
 let seed=1u;let fill=select(128u,255u,p.x/8u>=2u);let gap=select(0u,32u,(p.x/8u)%2u==1u);
 let instr=vec4u((seed<<24u)|(0x80ffu<<8u)|255u,(191u<<24u)|(byte<<16u)|(fill<<8u)|gap,0x4080c060u,(OP_CELL<<27u)|(REGION_ALL<<24u)|(1u<<16u)|0x2040u);
 let axis=decode_dir16(instr_dir16(instr));let refaxis=select(vec3f(0.,1.,0.),vec3f(1.,0.,0.),abs(axis.y)>.9);
 let t=normalize(cross(refaxis,axis));let b=normalize(cross(axis,t));let h=xy.y/density*2.-1.;let angle=xy.x/density*TAU;
 let dir=normalize(axis*h+sqrt(1.-h*h)*(-t*cos(angle)-b*sin(angle)));
 let uv=cell_axis_cylinder_uv(dir,axis);let a=cell_hex(uv,density);let z=cell_hex_2d(uv,density);
 let old=eval_cell(dir,instr,RegionWeights(.2,.3,.5));let candidate=eval_cell_2d(dir,instr,RegionWeights(.2,.3,.5));
 var out=vec4f(a,cell_hash2(a.xy,1.));let field=p.x%8u;
 if field==1u {out=vec4f(z,cell_hash2(z.xy,1.));}
 if field==2u {out=vec4f(old.regions.sky,old.regions.wall,old.regions.floor,1.);}
 if field==3u {out=vec4f(candidate.regions.sky,candidate.regions.wall,candidate.regions.floor,1.);}
 var layers:array<vec4u,8>;layers[0]=instr;
 // Real feature dispatch consumes returned CELL masks; no test-side mask override.
 let mask=array<u32,3>(REGION_SKY,REGION_WALLS,REGION_FLOOR)[min(field,7u)-min(field,5u)];
 layers[1]=vec4u((0x80ffu<<8u)|255u,0x08084020u,0xff80ff80u,(OP_GRID<<27u)|(mask<<24u));
 if field==4u {out=vec4f(evaluate_epu_layers(dir,layers),1.);}
 if field>=5u {out=vec4f(medium_scene(dir,layers),1.);}
 textureStore(result,p.xy,out);
}