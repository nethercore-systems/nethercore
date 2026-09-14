
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let byte=p.y%256u;let variant=0u;
 let seed=array<u32,4>(0u,1u,127u,255u)[(p.y/256u)%4u];
 let density=mix(4.,64.,u8_to_01(byte));
 let v=array<f32,4>(.13,.37,.67,.89)[p.y/1024u];
 let path=p.x/20u;let sample=p.x%20u;
 var delta=0.;
 if sample<15u {delta=f32(i32(sample%5u)-2)*array<f32,3>(.0001,.00001,.000001)[sample/5u];}
 else {delta=array<f32,5>(.13,.4,.71,1.9,2.8)[sample-15u];}
 let instr=vec4u((seed<<24u)|(0xff80u<<8u)|255u,(96u<<24u)|(byte<<16u)|(128u<<8u),
 0x4080c060u,(OP_CELL<<27u)|(REGION_ALL<<24u)|(3u<<21u)|(variant<<16u)|0x2040u);
 let axis=decode_dir16(instr_dir16(instr));
 let reference=select(vec3f(0.,1.,0.),vec3f(1.,0.,0.),abs(axis.y)>.9);
 let t=normalize(cross(reference,axis));let b=normalize(cross(axis,t));
 let h=v*2.-1.;let dir=normalize(axis*h+sqrt(1.-h*h)*(-t*cos(delta)-b*sin(delta)));
 let uv=cell_axis_cylinder_uv(dir,axis);
 let value=evaluate_bounds_layer(dir,instr,OP_CELL,axis,RegionWeights(.2,.3,.5));
 var layers:array<vec4u,8>;layers[0]=instr;
 var out=vec4f(value.regions.sky,value.regions.wall,value.regions.floor,value.sample.w);
 if path==1u {out=vec4f(value.sample.rgb*value.sample.w,value.sample.w);}
 if path==2u {out=vec4f(evaluate_epu_layers(dir,layers),1.);}
 if path==4u {out=vec4f(apply_blend(vec3f(0.),value.sample,BLEND_LERP),1.);}
 if path==3u {
  var info=cell_grid(uv,density);
  if variant==4u {info=cell_shatter(uv,density,0.);}
  if variant==5u {info=cell_brick(uv,density,0.);}
  out=vec4f(shortest_periodic_delta(uv.x,0.,1.)*10000.,info.z,info.xy);
 }
 // INJECT_WRAP_JUMP
 textureStore(result,p.xy,out);
}
