@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@group(0) @binding(1) var<storage,read> inputs:array<vec4u>;
@compute @workgroup_size(32) fn probe(@builtin(global_invocation_id) p:vec3u){
 if p.x>=64u||p.y>=240u{return;}
 let coords=array<f32,8>(-0.7,-0.5,-0.3,-0.1,0.1,0.3,0.5,0.7);let uv=vec2f(coords[p.x%8u],coords[p.x/8u]);
 let phase=array<u32,8>(0u,32u,64u,96u,128u,160u,192u,255u)[p.y%8u];
 let size_byte=array<u32,5>(0u,16u,64u,160u,255u)[(p.y/8u)%5u];
 let axis_bits=array<u32,3>(0x8080u,0xff80u,0xe0a0u)[(p.y/40u)%3u];let variant=(p.y/120u)*2u;
 let i=inputs[p.y];
 let expected_i=vec4u((127u<<24u)|(axis_bits<<8u)|240u,(128u<<24u)|(size_byte<<16u)|(127u<<8u)|phase,0xaa8c5a28u,(16u<<27u)|(7u<<24u)|(3u<<21u)|(variant<<16u)|0x466eu);
 let size=mix(0.5f,45.0f,f32(size_byte)/255.0)*PI/180.0;let axis=decode_dir16(axis_bits);
 let hint=select(vec3f(0.,1.,0.),vec3f(1.,0.,0.),abs(axis.y)>0.9);let tangent=normalize(cross(hint,axis));let bitangent=cross(axis,tangent);
 let projected=(tangent*uv.x+bitangent*uv.y)*sin(size);let dir=normalize(axis*sqrt(1.0-dot(projected,projected))+projected);
 let phase_rad=f32(phase)/255.0*TAU;let lit=max(0.0,sqrt(1.0-dot(uv,uv))*cos(phase_rad)-uv.y*sin(phase_rad));
 // Keep unchanged detail/limb/masks in the actual helpers; only phase has an independent oracle.
 let r=acos(clamp(dot(dir,axis),-1.0,1.0))/size;let actual_uv=celestial_surface_uv(dir,axis,sin(size));
 let limb=pow(epu_saturate(1.0-r),mix(0.5f,4.0f,127.0f/255.0f));
 var expected=eval_celestial_moon(r,actual_uv,lit,limb,instr_color_a(i),instr_color_b(i),0.0);
 if variant==2u{expected=eval_celestial_planet(r,actual_uv,lit,limb,instr_color_a(i),instr_color_b(i),127.0/255.0*8.0,0.0);}
 let q=eval_celestial(dir,i,1.0);let delta=abs(q.rgb-expected.rgb);
 let input_error=select(0.0,1.0,any(i!=expected_i));
 let night=select(0.0,max(q.rgb.x,max(q.rgb.y,q.rgb.z)),lit==0.0);
 textureStore(result,vec2i(p.xy),vec4f(max(delta.x,max(delta.y,delta.z)),max(abs(q.w-expected.w*(128.0/255.0*2.0)),input_error),night,f32(variant)));
}
