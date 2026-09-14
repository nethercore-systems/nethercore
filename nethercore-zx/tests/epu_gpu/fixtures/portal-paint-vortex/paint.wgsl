@group(0) @binding(0) var result: texture_storage_2d<rgba16float,write>;
@group(0) @binding(1) var<storage,read> words: array<vec4u>;
const USE_LEGACY:bool=__LEGACY__;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let i=words[p.y];let variant=instr_variant_id(i);let center=decode_dir16(instr_dir16(i));
 let up=select(vec3f(0,1,0),vec3f(1,0,0),abs(center.y)>0.9);let t=normalize(cross(up,center));let b=normalize(cross(center,t));
 let size=mix(0.05f,0.8f,f32(instr_a(i))/255.0f);let offset=array<f32,8>(-0.025,-0.005,0,0.005,0.02,0.06,0.1,0.2)[p.x%8u];
 let angle=f32(p.x/8u)*TAU/8.0;var uv=vec2f(cos(angle),sin(angle))*(size+offset);
 if variant==1u {uv=vec2f(size+offset,size*0.6*(f32(p.x/8u)/3.5-1.0));}
 if variant==4u {uv=vec2f(size*0.1+offset,(f32(p.x/8u)-3.5)*0.15);}
 if variant==5u {uv=vec2f((f32(p.x/8u)-3.5)*0.15,size*0.2+offset);}
 let dir=normalize(center+t*uv.x+b*uv.y);let mask=f32(p.y%4u)/3.0;
 var a=i;a.z=0xff000000u;a.w=(a.w&0xffff0000u)|0xffffu;
 var b_i=i;b_i.z=0x00ffffffu;b_i.w&=0xffff0000u;b_i.y|=255u<<24u;
 // Basis paints expose the unchanged geometric components, not the candidate mix.
 let interior=legacy_portal(dir,a,1.0).rgb.x;let edge=legacy_portal(dir,b_i,1.0).rgb.x*0.5;
 let expected=(instr_color_a(i)*interior*instr_alpha_a_f32(i)+instr_color_b(i)*edge*instr_alpha_b_f32(i)*(f32(instr_intensity(i))/255.0*2.0))*mask;
 let old=legacy_portal(dir,i,mask);var actual=eval_portal(dir,i,mask);if USE_LEGACY{actual=old;}
 let delta=abs(actual.rgb*actual.w-expected);let error=max(delta.x,max(delta.y,delta.z));
 let invalid=instr_opcode(i)!=17u||dot(dir,center)<0.3;
 textureStore(result,vec2i(p.xy),vec4f(error,abs(actual.w-old.w),select(0.0,1.0,invalid),1.0));
}
