@group(0) @binding(0) var result: texture_storage_2d<rgba16float,write>;
@group(0) @binding(1) var<storage,read> inputs:array<vec4u>;
fn bytes(v:u32)->vec4f{return vec4f(f32(v&255u),f32((v>>8u)&255u),f32((v>>16u)&255u),f32(v>>24u));}
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u){
 let instr=inputs[(p.y*58u)*2u+1u]; let mode=p.x/30u; let path=p.x%30u;
 let nc=decode_dir16(0x8080u); let nr=decode_dir16(instr_dir16(instr));
 let bc=split_build_basis(nc); let br=split_build_basis(nr);
 let sign=select(-1.,1.,p.y>=36u);
 var dir=nc*sign; if mode>=1u{dir=nr*sign;} if mode==2u{dir=normalize(dir);}
 let count=mix(2.,16.,u8_to_01(instr_c(instr))); let rot=u8_to_01(instr_d(instr)); let bw=max(u8_to_01(instr_a(instr))*0.2,0.001);
 let wc=split_prism(dir,nc,bc,count,rot,bw); let wr=split_prism(dir,nr,br,count,rot,bw);
 let v=eval_split(dir,instr,RegionWeights(1.,0.,0.));var layers:array<vec4u,8>;layers[0]=instr;
 var out=vec4f(0.);
 if path<3u{out=bytes(bitcast<u32>(nc[path]));}
 if path>=3u && path<6u{out=bytes(bitcast<u32>(nr[path-3u]));}
 if path>=6u && path<9u{out=bytes(bitcast<u32>(bc[0][path-6u]));}
 if path>=9u && path<12u{out=bytes(bitcast<u32>(br[0][path-9u]));}
 if path>=12u && path<15u{out=bytes(bitcast<u32>(bc[1][path-12u]));}
 if path>=15u && path<18u{out=bytes(bitcast<u32>(br[1][path-15u]));}
 if path==18u{out=bytes(bitcast<u32>(dot(dir,bc[0])));}
 if path==19u{out=bytes(bitcast<u32>(dot(dir,bc[1])));}
 if path==20u{out=bytes(bitcast<u32>(dot(dir,br[0])));}
 if path==21u{out=bytes(bitcast<u32>(dot(dir,br[1])));}
 if path==22u{out=vec4f(wc,1.);}
 if path==23u{out=vec4f(wr,1.);}
 if path==24u{out=vec4f(v.regions.sky,v.regions.wall,v.regions.floor,1.);}
 if path==25u{out=vec4f(v.sample.rgb,v.sample.w);}
 if path==26u{out=vec4f(evaluate_epu_layers(dir,layers),1.);}
 if path>=27u{out=bytes(bitcast<u32>(dir[path-27u]));}
 textureStore(result,p.xy,out);
}
