@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(32,1)
fn probe(@builtin(global_invocation_id) p:vec3u){
 if p.x>=256u||p.y>=192u{return;}
 let variant=p.y%8u;let radius=array<f32,3>(0.,.5,1.5)[(p.y/8u)%3u];let blend=p.y/24u;
 let axis=decode_dir16(0x8080u);let tangent=normalize(cross(vec3f(0.,1.,0.),axis));
 let angle=radius*mix(0.5f,45.0f,127.0f/255.0f)*PI/180.;let dir=normalize(axis*cos(angle)+tangent*sin(angle));
 let region:f32=select(0.37f,1.0f,blend%2u==0u);
 let low=(127u<<24u)|(0x8080u<<8u)|(8u<<4u)|7u;
 let i=vec4u(low,(p.x<<24u)|(127u<<16u)|(127u<<8u),0xc0502010u,(16u<<27u)|(7u<<24u)|(variant<<16u)|0x6080u);
 var base=i;base.y=(base.y&0x00ffffffu)|(128u<<24u);
 let a=eval_celestial(dir,i,region);let b=eval_celestial(dir,base,region);
 let expected=LayerSample(b.rgb,b.w*(f32(p.x)/128.0));
 let d=abs(a.rgb-b.rgb);let e=abs(apply_blend(vec3f(.1,.2,.3),a,blend)-apply_blend(vec3f(.1,.2,.3),expected,blend));
 let zero=eval_celestial(dir,vec4u(i.x,i.y&0x00ffffffu,i.z,i.w),region);
 let off=abs(apply_blend(vec3f(.1,.2,.3),zero,blend)-vec3f(.1,.2,.3));
 textureStore(result,vec2i(p.xy),vec4f(max(d.x,max(d.y,d.z)),abs(a.w-expected.w),max(e.x,max(e.y,e.z)),max(off.x,max(off.y,off.z))));
}
