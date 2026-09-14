@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
fn emitted(s:LayerSample)->vec3f{return s.rgb*clamp(s.w,0.0,1.0);}
@compute @workgroup_size(32,1)
fn probe(@builtin(global_invocation_id) p:vec3u){
 if p.x>=32u||p.y>=864u{return;}
 let variant=p.y%8u;let axis_bits=array<u32,3>(0x8080u,0xff80u,0xe0a0u)[(p.y/8u)%3u];
 let size_byte=array<u32,3>(0u,127u,255u)[(p.y/24u)%3u];
 let limb_byte=array<u32,3>(0u,127u,255u)[(p.y/72u)%3u];
 let phase_byte=array<u32,4>(0u,64u,128u,192u)[p.y/216u];
 let axis=decode_dir16(axis_bits);
 let hint=select(vec3f(0.,1.,0.),vec3f(1.,0.,0.),abs(axis.y)>.9);
 let tangent=normalize(cross(hint,axis));
 let radius=f32(p.x)*3.0/31.0;
 let angular_size=mix(.5,45.,f32(size_byte)/255.)*PI/180.;
 let angle=radius*angular_size;let dir=normalize(axis*cos(angle)+tangent*sin(angle));
 let low=(127u<<24u)|(axis_bits<<8u)|(8u<<4u)|7u;
 let high=(32u<<24u)|(size_byte<<16u)|(limb_byte<<8u)|phase_byte;
 let i=vec4u(low,high,0xc0502010u,(16u<<27u)|(7u<<24u)|(variant<<16u)|0x6080u);
 var twice=i;twice.y=(twice.y&0x00ffffffu)|(64u<<24u);
 let a=emitted(eval_celestial(dir,i,1.));let b=emitted(eval_celestial(dir,twice,1.));
 let d=abs(b-a*2.0);
 textureStore(result,vec2i(p.xy),vec4f(max(d.x,max(d.y,d.z)),max(b.x,max(b.y,b.z)),radius,f32(variant)));
}
