@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(32,1)
fn probe(@builtin(global_invocation_id) p:vec3u){
 if p.x>=65u||p.y>=48u{return;}
 let variant=p.y%8u;let axis_bits=array<u32,3>(0x8080u,0xff80u,0xe0a0u)[(p.y/8u)%3u];
 let size_byte=array<u32,2>(127u,255u)[p.y/24u];let axis=decode_dir16(axis_bits);
 let hint=select(vec3f(0.,1.,0.),vec3f(1.,0.,0.),abs(axis.y)>.9);let tangent=normalize(cross(hint,axis));
 let angle=f32(p.x)*PI/64.;let dir=normalize(axis*cos(angle)+tangent*sin(angle));
 let i=vec4u((255u<<24u)|(axis_bits<<8u)|(8u<<4u)|7u,(255u<<24u)|(size_byte<<16u)|(127u<<8u)|64u,0xc0502010u,(16u<<27u)|(7u<<24u)|(variant<<16u)|0x6080u);
 let a=apply_blend(vec3f(0.),eval_celestial(dir,i,1.),0u);
 let b=apply_blend(vec3f(0.),metric_eval_celestial(dir,i,1.),0u);
 let d=abs(a-b);
 textureStore(result,vec2i(p.xy),vec4f(max(d.x,max(d.y,d.z)),max(a.x,max(a.y,a.z)),max(b.x,max(b.y,b.z)),f32(variant)));
}
