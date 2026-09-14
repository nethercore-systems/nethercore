@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let byte=p.y%256u;let seed=array<u32,4>(0u,1u,127u,255u)[(p.y/256u)%4u];
 let sign=select(1.,-1.,(p.y/1024u)%2u==1u);let fill=array<u32,3>(0u,128u,255u)[p.y/2048u];
 let path=p.x/160u;let level=(p.x/32u)%5u;let angle=f32(p.x%32u)*TAU/32.;
 let radius=array<f32,5>(0.,.01,.001,.0001,.00001)[level];
 let axis_bits=array<u32,3>(0xff80u,0x80ffu,0xe0a0u)[byte%3u];let axis=decode_dir16(axis_bits);
 let reference=select(vec3f(0.,1.,0.),vec3f(1.,0.,0.),abs(axis.y)>.9);let t=normalize(cross(reference,axis));let b=normalize(cross(axis,t));
 let dir=normalize(axis*sign*sqrt(1.-radius*radius)+radius*(t*cos(angle)+b*sin(angle)));
 let i=vec4u((seed<<24u)|(axis_bits<<8u)|255u,(byte<<16u)|(fill<<8u),
  0x58c09050u,(OP_CELL<<27u)|(7u<<24u)|(3u<<21u)|(3u<<16u)|0x1828u);
 let q=evaluate_bounds_layer(dir,i,OP_CELL,axis,RegionWeights(.2,.3,.5));
 var value=vec4f(q.sample.rgb*q.sample.w,q.sample.w);
 if path==1u {value=vec4f(q.regions.sky,q.regions.wall,q.regions.floor,1.);}
 if path==2u {value=vec4f(length(cross(dir,axis))*1000000.,dot(dir,axis)*sign,0.,1.);}
 textureStore(result,p.xy,value);
}
