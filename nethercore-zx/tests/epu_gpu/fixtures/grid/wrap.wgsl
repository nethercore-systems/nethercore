// Original rays, steps and hard-edge controls retained; margins follow the approved new count/phase contract.
// Actual production dispatch; diagnose the chart cut away from authored line edges.
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
fn witness_value(dir:vec3f, instr:vec4u)->f32 {
 let s=evaluate_layer(dir,instr,vec3f(0.,1.,0.),RegionWeights(1.,0.,0.));
 return s.rgb.x*s.w @INJECT@;
}
fn edge_margin(dir:vec3f,instr:vec4u)->f32 {
 let pattern=instr_c(instr)>>4u;
 let scale=f32(select((382u+63u*instr_a(instr))/255u,2u*((510u+63u*instr_a(instr))/510u),pattern==2u));
 let thick=mix(.001,.1,f32(instr_b(instr))/255.);
 let scroll=(f32(instr_d(instr))/256.)*f32(instr_c(instr)&15u)*select(1.,2.,pattern==2u);
 let uv=vec2f(atan2(dir.x,dir.z)/TAU*scale+scroll,(dir.y*.5+.5)*scale);
 let frac=fract(uv);
 if instr_c(instr)>>4u==2u {return min(min(frac.x,1.-frac.x),min(frac.y,1.-frac.y));}
 let dx=abs(abs(frac.x-.5)-thick);
 if instr_c(instr)>>4u==1u {return min(dx,abs(abs(frac.y-.5)-thick));}
 return dx;
}
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let levels=array<f32,4>(.001,.0001,.00001,.000001);
 let e=levels[p.x/256u];
 if p.y==256u {
  // A real authored sharp stripe edge must stay sharp; this is not a wrap seam.
  let instr=vec4u(255u,(255u<<24u)|(255u<<16u)|(128u<<8u),0xff000000u,(9u<<27u)|(7u<<24u)|(3u<<21u)|0xffffu);
  let thick=mix(0.001f,0.1f,128.0f/255.0f);let angle=(.5+thick)/64.*TAU;
  let l=vec3f(sin(angle-e),0.,cos(angle-e));let r=vec3f(sin(angle+e),0.,cos(angle+e));
  textureStore(result,p.xy,vec4f(abs(witness_value(l,instr)-witness_value(r,instr)),length(l-r)*1e6,0.,1.));return;
 }
 let patterns=array<u32,4>(0u,1u,2u,15u);
 let speeds=array<u32,8>(0u,1u,3u,5u,7u,11u,14u,15u);
 let phases=array<u32,8>(0u,1u,17u,63u,127u,191u,254u,255u);
 let pattern=patterns[p.y/64u];let speed=speeds[(p.y/8u)%8u];let phase=phases[p.y%8u];
 let instr=vec4u((phase<<24u)|255u,(255u<<24u)|((p.x%256u)<<16u)|(128u<<8u)|(pattern<<4u)|speed,0xff000000u,(9u<<27u)|(7u<<24u)|(3u<<21u)|0xffffu);
 let l=normalize(vec3f(-e,.275,-1.));let r=normalize(vec3f(e,.275,-1.));let c=normalize(vec3f(0.,.275,-1.));
 let ll=normalize(vec3f(-2.*e,.275,-1.));let rr=normalize(vec3f(2.*e,.275,-1.));
 let a=witness_value(l,instr);let b=witness_value(r,instr);let zero=witness_value(c,instr);
 let jump=max(abs(a-b),max(abs(a-zero),abs(b-zero)));
 let same_side=max(abs(a-witness_value(ll,instr)),abs(b-witness_value(rr,instr)));
 let margin=min(edge_margin(l,instr),min(edge_margin(r,instr),edge_margin(c,instr)));
 textureStore(result,p.xy,vec4f(jump,same_side,margin,length(l-r)*1e6));
}
