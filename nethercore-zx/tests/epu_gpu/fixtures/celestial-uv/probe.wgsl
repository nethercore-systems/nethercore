@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(32,1)
fn probe(@builtin(global_invocation_id) p:vec3u){
 if p.x>=64u||p.y>=768u{return;}
 let size_byte=p.y%256u;let axis_bits=array<u32,3>(0x8080u,0xff80u,0xe0a0u)[p.y/256u];let axis=decode_dir16(axis_bits);
 let hint=select(vec3f(0.,1.,0.),vec3f(1.,0.,0.),abs(axis.y)>.9);let t=normalize(cross(hint,axis));let b=cross(axis,t);
 let radius=array<f32,4>(0.,.25,.5,1.)[p.x%4u];let bearing=f32(p.x/4u)*TAU/16.;
 let size=mix(0.5f,45.0f,f32(size_byte)/255.0f)*PI/180.0f;let angle=radius*size;
 let dir=normalize(axis*cos(angle)+(t*cos(bearing)+b*sin(bearing))*sin(angle));
 let i=vec4u((127u<<24u)|(axis_bits<<8u)|255u,(128u<<24u)|(size_byte<<16u)|(127u<<8u),0xc0502010u,(16u<<27u)|(7u<<24u)|0x6080u);
 let observed=inspect_celestial_uv(dir,i,1.);
 let expected=vec2f(dot(dir,t),dot(dir,b))/sin(size);
 let error=abs(observed.rgb.xy-expected);
 textureStore(result,vec2i(p.xy),vec4f(max(error.x,error.y),length(observed.rgb.xy),length(expected),radius));
}
