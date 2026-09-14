@group(0) @binding(0) var result: texture_storage_2d<rgba16float,write>;
@group(0) @binding(1) var<storage,read> words: array<vec4u>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let i=words[p.y];let center=decode_dir16(instr_dir16(i));let up=select(vec3f(0,1,0),vec3f(1,0,0),abs(center.y)>0.9);let t=normalize(cross(up,center));let b=normalize(cross(center,t));
 let size=mix(0.05f,0.8f,f32(instr_a(i))/255.0f);let angle=f32(p.x%16u)*TAU/16.0;
 let r=max(0.0,size-0.2+f32(p.x/16u)*0.04);var uv=vec2f(cos(angle),sin(angle))*r;if p.x==0u{uv=vec2f(0.0);}
 let dir=normalize(center+t*uv.x+b*uv.y);let phase=f32(instr_d(i))/256.0;
 let warped=portal_apply_vortex_warp(uv,phase);let turned=normalize(center+t*warped.x+b*warped.y);
 var tear=i;tear.w=(tear.w&~(7u<<16u))|(2u<<16u);
 // Composition oracle: existing TEAR seen in the existing spiral-warped chart.
 let expected=eval_portal(turned,tear,1.0);let actual=eval_portal(dir,i,1.0);
 let delta=abs(actual.rgb*actual.w-expected.rgb*expected.w);let error=max(abs(actual.w-expected.w),max(delta.x,max(delta.y,delta.z)));
 var p0=i;p0.x&=0x00ffffffu;var p64=p0;p64.x|=64u<<24u;
 let change=abs(eval_portal(dir,p0,1.0).w-eval_portal(dir,p64,1.0).w);
 let start=portal_apply_vortex_warp(uv,0.0);let end=portal_apply_vortex_warp(uv,1.0);
 let invalid=instr_opcode(i)!=17u||instr_variant_id(i)!=3u||any(start!=start)||any(end!=end)||any(abs(start-end)>vec2f(0.000002))||(instr_c(i)==0u&&change>0.002);
 textureStore(result,vec2i(p.xy),vec4f(error,select(0.0,1.0,instr_c(i)>0u&&change>0.01),select(0.0,1.0,invalid),1.0));
}
