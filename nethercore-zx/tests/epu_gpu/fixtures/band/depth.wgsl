@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
fn band_ray(a:f32,axis:vec3f)->vec3f {
 let ref_vec=select(vec3f(0.,1.,0.),vec3f(1.,0.,0.),abs(axis.y)>.9);
 let t=normalize(cross(axis,ref_vec));let b=cross(axis,t);let h=127.0f/255.0f-0.5f;
 return normalize(axis*h+sqrt(1.-h*h)*(cos(a)*t+sin(a)*b));
}
fn paint(d:vec3f,i:vec4u)->f32 {
 let s=evaluate_layer(d,i,vec3f(0.,1.,0.),RegionWeights(0.,1.,0.));return s.rgb.x*s.w;
}
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u){
 let axes=array<u32,3>(0x8080u,0xff80u,0xe0a0u);
 let widths=array<u32,3>(0u,127u,255u);
 let depth=p.y%16u;let c=p.y/16u;let phase=p.x;
 let axis=decode_dir16(axes[c/6u]);let width=widths[(c/2u)%3u];let soft=(c%2u)*255u;
 let i=vec4u((phase<<24u)|(axes[c/6u]<<8u)|240u|depth,(255u<<24u)|(width<<16u)|(127u<<8u)|soft,0xffffffffu,(19u<<27u)|(7u<<24u)|(3u<<21u)|0xffffu);
 var plain_i=i;plain_i.x=i.x&0xfffffff0u;
 var full_i=i;full_i.x=i.x|15u;
 var zero_i=plain_i;zero_i.x=plain_i.x&0x00ffffffu;
 let ray=band_ray(.337,axis);
 let plain=paint(ray,plain_i);let full=paint(ray,full_i);
 let actual=paint(ray,i);
 textureStore(result,p.xy,vec4f(abs(actual-mix(plain,full,f32(depth)/15.)),abs(plain-paint(ray,zero_i)),abs(full-plain),plain));
}
