@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@group(0) @binding(1) var<storage,read> instructions:array<vec4u>;
@compute @workgroup_size(32) fn probe(@builtin(global_invocation_id) p:vec3u){
 if p.x>=64u||p.y>=1024u{return;}let i=instructions[p.y/4u];let tilt=instr_d(i);let radius=array<f32,4>(1.6,1.9,2.2,2.4)[p.y%4u];
 let size=mix(0.5f,45.0f,64.0f/255.0f)*PI/180.0;let axis=decode_dir16(0x8080u);let t=normalize(cross(vec3f(0.,1.,0.),axis));let b=cross(axis,t);
 let phi=f32(p.x)*TAU/64.0;let d=normalize(axis*cos(radius*size)+(t*cos(phi)+b*sin(phi))*sin(radius*size));

 let q=eval_celestial(d,i,1.0);let ring=q.w/(128.0/255.0*2.0);
 var expected=0.0;
 if tilt>0u {
  let aspect=sin(f32(tilt)/255.0*PI*0.5);
  let rr=radius*length(vec2f(cos(phi),sin(phi)/aspect));
  expected=smoothstep(1.4,1.6,rr)*smoothstep(2.6,2.4,rr);
 }
 let input_error=select(0.0,1.0,tilt!=p.y/4u||instr_intensity(i)!=128u||instr_a(i)!=64u||instr_b(i)!=127u||instr_c(i)!=0u||instr_dir16(i)!=0x8080u);
 textureStore(result,vec2i(p.xy),vec4f(ring,radius,f32(instr_opcode(i)*8u+instr_variant_id(i)),max(abs(ring-expected),input_error)));
}