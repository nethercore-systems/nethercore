@group(0) @binding(0) var result: texture_storage_2d<rgba16float, write>;
fn bytes(v:u32)->vec4f {return vec4f(f32(v&255u),f32((v>>8u)&255u),f32((v>>16u)&255u),f32(v>>24u));}
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let row=p.y%72u; let context=p.y/72u; let s=p.x%58u; let path=p.x/58u;
 let pc=array<u32,4>(0u,128u,129u,255u)[row%4u];
 let pa=array<u32,3>(0u,64u,255u)[(row/4u)%3u];
 let pd=array<u32,3>(0u,64u,255u)[(row/12u)%3u];
 let sign=select(-1.,1.,row>=36u);
 let n=decode_dir16(0x8080u); let basis=split_build_basis(n);
 var dir=n*sign;
 if s==1u {dir=normalize(dir);}
 if s>=2u {
   let r=array<f32,7>(0.3,0.1,0.01,0.001,0.0001,0.00001,0.000001)[(s-2u)/8u];
   let meridian=(f32((s-2u)%8u)+0.37)*TAU/8.;
   dir=normalize(n*sign*cos(r)+(basis[0]*cos(meridian)+basis[1]*sin(meridian))*sin(r));
 }
 let alpha=array<u32,3>(15u,7u,0u)[context%3u];
 let blend=select(3u,0u,context>=3u);
 let instr=vec4u((pd<<24u)|(0x8080u<<8u)|(alpha<<4u),(pa<<16u)|pc,0x4060c080u,(4u<<27u)|(7u<<24u)|(blend<<21u)|(5u<<16u)|0xe080u);
 let weights=split_prism(dir,n,basis,mix(2.,16.,u8_to_01(pc)),u8_to_01(pd),max(u8_to_01(pa)*0.2,0.001));
 let v=eval_split(dir,instr,RegionWeights(1.,0.,0.));
 let bound=evaluate_bounds_layer(dir,instr,instr_opcode(instr),n,RegionWeights(1.,0.,0.));
 var layers:array<vec4u,8>; layers[0]=instr;
 var out=vec4f(weights,1.);
 if path==1u {out=vec4f(v.regions.sky,v.regions.wall,v.regions.floor,v.region_mix);}
 if path==2u {out=vec4f(bound.regions.sky,bound.regions.wall,bound.regions.floor,bound.region_mix);}
 if path==3u {out=vec4f(v.sample.rgb,v.sample.w);}
 if path==4u {out=vec4f(bound.sample.rgb,bound.sample.w);}
 if path==5u {out=vec4f(apply_blend(vec3f(0.),v.sample,blend),1.);}
 if path==6u {out=vec4f(evaluate_epu_layers(dir,layers),1.);}
 if path>=7u && path<=16u {
   let mask=array<u32,5>(0u,1u,2u,4u,7u)[(path-7u)/2u];
   layers[0].x=layers[0].x&0xffffff0fu;
   let axes=array<u32,6>(0x80ffu,0x8000u,0xff80u,0x0080u,0x8080u,0xffffu);
   var manual=vec3f(0.);
   for(var j=0u;j<6u;j++) {
     let feature=vec4u((axes[j]<<8u)|240u,32u<<24u,0x80808080u,(18u<<27u)|(mask<<24u)|0x8080u);
     layers[j+1u]=feature;
     manual=apply_blend(manual,evaluate_layer(dir,feature,n,v.regions),instr_blend(feature));
   }
   out=vec4f(manual,1.);
   if (path-7u)%2u==1u {out=vec4f(evaluate_epu_layers(dir,layers),1.);}
 }
 if path>=17u && path<=19u {out=bytes(bitcast<u32>(dir[path-17u]));}
 if path==20u {out=bytes(bitcast<u32>(dot(dir,basis[0])));}
 if path==21u {out=bytes(bitcast<u32>(dot(dir,basis[1])));}
 if path==22u {out=vec4f(dot(dir,n),atan2(dot(dir,basis[1]),dot(dir,basis[0])),length(dir),1.);}
 if path==23u {out=vec4f(f32(instr_c(instr)),f32(instr_a(instr)),f32(instr_d(instr)),f32(instr_blend(instr)));}
 if path==24u {out=vec4f(instr_color_a(instr),instr_alpha_a_f32(instr));}
 if path==25u {out=vec4f(instr_color_b(instr),f32(instr_variant_id(instr)));}
 textureStore(result,p.xy,out);
}
