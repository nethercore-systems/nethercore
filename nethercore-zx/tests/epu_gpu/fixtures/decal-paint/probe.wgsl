@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
@group(0) @binding(1) var<storage,read> input_words:array<vec4u>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 var i=input_words[0];let shape=p.y/9u;let soft_q=array<u32,3>(0u,7u,15u)[(p.y/3u)%3u];let mode=p.y%3u;
 let alpha=p.x%16u;let aa=select(alpha,0u,mode==1u);let ab=select(select(alpha,15u-alpha,mode==2u),0u,mode==0u);
 i.x=(i.x&0xffffff00u)|(aa<<4u)|ab;i.y=(i.y&0xff00ffffu)|(((shape<<4u)|soft_q)<<16u);
 let center=decode_dir16(instr_dir16(i));let t=normalize(cross(vec3f(0.,1.,0.),center));
 let size=f32(instr_b(i))/255.*0.5;let soft=mix(0.001f,0.05f,f32(soft_q)/15.);
 var edge_angle=size;if shape==1u {edge_angle=size*1.2;}else if shape==2u {edge_angle=asin(size);}else if shape==3u {edge_angle=asin(size*0.1);}
 let angle=edge_angle+(f32(p.x/16u)/15.*6.-3.)*soft;let ray=normalize(center*cos(angle)+t*sin(angle));
 let old=legacy_decal(ray,i,1.);let v=ACTUAL_DECAL(ray,i,1.);
 // Legacy RGB already contains the correctly coverage-weighted paint. Its final
 // weight applies that coverage again. Preserve weight, but don't square paint.
 let error=abs(v.rgb*v.w-old.rgb);let e=max(error.x,max(error.y,error.z));
 textureStore(result,vec2i(p.xy),vec4f(e,abs(v.w-old.w),v.w,f32(instr_opcode(i))));
}
