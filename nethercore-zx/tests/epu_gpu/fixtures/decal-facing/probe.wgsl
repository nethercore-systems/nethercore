@group(0) @binding(0) var result: texture_storage_2d<rgba16float,write>;
@group(0) @binding(1) var<storage,read> data: array<vec4u>;
@compute @workgroup_size(8,8)
fn probe(@builtin(global_invocation_id) p: vec3u) {
 let dims=textureDimensions(result);if p.x>=dims.x||p.y>=dims.y {return;}
 let i=data[p.y];let axis=decode_dir16(instr_dir16(i));
 let hint=select(vec3f(0,1,0),vec3f(1,0,0),abs(axis.y)>.9);
 let t=normalize(cross(hint,axis));let b=cross(axis,t);
 let mu=select(2.0*f32(p.x)/f32(dims.x-1u)-1.0,0.0,p.x==32u);
 let az=TAU*f32(p.y%17u)/17.0;
 let dir=normalize(axis*mu+(t*cos(az)+b*sin(az))*sqrt(max(0.0,1.0-mu*mu)));
 let mask=f32(p.y%4u)/3.0;
 let old=legacy_facing_decal(dir,i,mask);let actual=ACTUAL_DECAL_FACING(dir,i,mask);
 var wanted=old;if dot(dir,axis)<=0.0 {wanted=LayerSample(vec3f(0),0.0);}
 let rgb=abs(actual.rgb-wanted.rgb);let err=max(abs(actual.w-wanted.w),max(rgb.x,max(rgb.y,rgb.z)));
 textureStore(result,vec2i(p.xy),vec4f(err,f32(dot(dir,axis)>0.0),f32(instr_opcode(i)!=8u),1));
}
