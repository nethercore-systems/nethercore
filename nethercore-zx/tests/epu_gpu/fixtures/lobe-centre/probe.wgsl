@group(0) @binding(0) var result: texture_storage_2d<rgba16float,write>;
@group(0) @binding(1) var<storage,read> data: array<vec4u>;
@compute @workgroup_size(8,8)
fn probe(@builtin(global_invocation_id) p: vec3u) {
 let dims=textureDimensions(result); if p.x>=dims.x || p.y>=dims.y { return; }
 let i=data[p.y];let d=f32(p.x)/f32(dims.x-1u);let axis=decode_dir16(instr_dir16(i));
 let hint=select(vec3f(0,1,0),vec3f(1,0,0),abs(axis.y)>.9);let tangent=normalize(cross(hint,axis));
 let dir=normalize(axis*d+tangent*sqrt(max(0.0,1.0-d*d)));
 let actual=ACTUAL_LOBE(dir,i,0.75);let prior=legacy_lobe(dir,i,0.75);
 let cosine=epu_saturate(dot(dir,axis));let exponent=mix(1.0,64.0,u8_to_01(instr_a(i)));
 let phase=f32(instr_d(i))/256.0;
 var wave=1.0;
 if instr_c(i)==1u {wave=.5+.5*sin(TAU*phase);} else if instr_c(i)==2u {wave=1.0-abs(2.0*phase-1.0);} else if instr_c(i)==3u {wave=step(0.5,fract(phase*4.0));} else if instr_c(i)>3u {wave=.5+.5*sin(TAU*phase);}
 let wanted=pow(cosine,exponent)*u8_to_01(instr_intensity(i))*2.0*instr_alpha_a_f32(i)*wave*.75;
 let paint=abs(actual.rgb-prior.rgb);
 textureStore(result,vec2i(p.xy),vec4f(abs(actual.w-wanted),max(paint.x,max(paint.y,paint.z)),f32(instr_opcode(i)!=18u),1));
}
