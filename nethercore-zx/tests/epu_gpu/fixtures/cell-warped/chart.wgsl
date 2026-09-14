// Frozen chart-cut gate: buffer-fed instructions; all 256 densities.
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@group(0) @binding(1) var<storage,read> case_words:array<vec4u,864>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let base=case_words[p.y%768u];let axis=decode_dir16(instr_dir16(base));
 let latitude=array<f32,4>(.13,.47,.67,.89)[p.y/768u];
 let seed=array<u32,4>(0u,1u,127u,255u)[p.x%4u];
 let fill=array<u32,3>(0u,128u,255u)[(p.x/4u)%3u];
 let gap=array<u32,3>(0u,64u,255u)[(p.x/12u)%3u];
 let alpha=array<u32,3>(0u,7u,15u)[p.x/36u];
 let i=vec4u((seed<<24u)|(base.x&0x00ffff00u)|(alpha<<4u)|15u,(base.y&0xffff0000u)|(fill<<8u)|gap,base.z,base.w);
 let ref_vec=select(vec3f(0.,1.,0.),vec3f(1.,0.,0.),abs(axis.y)>.9);
 let t=normalize(cross(ref_vec,axis));let b=normalize(cross(axis,t));
 let h=latitude*2.-1.;let r=sqrt(1.-h*h);let center=normalize(axis*h-t*r);let tangent=cross(axis,center);
 let steps=array<f32,5>(.004,.0004,.00004,.000004,.0000004);
 let offsets=array<f32,5>(-2.,-1.,0.,1.,2.);
 var previous=vec2f(1e9);var prior_span=1e9;var last=vec2f(0.);var flags=0.;var final_span=0.;
 for(var scale=0u;scale<5u;scale+=1u) {
  var paint:array<vec4f,5>;var regions:array<vec4f,5>;var dirs:array<vec3f,5>;var geometry:array<vec3f,5>;
  for(var s=0u;s<5u;s+=1u) {
   let delta=offsets[s]*steps[scale];var dir=center;
   if s!=2u {
    if scale<3u {dir=normalize(axis*h+r*(-t*cos(delta)-b*sin(delta)));}
    else {dir=center+tangent*delta;}
   }
   dirs[s]=dir;
   let q=evaluate_bounds_layer(dir,i,OP_CELL,axis,RegionWeights(.2,.3,.5));
   paint[s]=vec4f(q.sample.rgb*q.sample.w,q.sample.w);regions[s]=vec4f(q.regions.sky,q.regions.wall,q.regions.floor,1.);
   geometry[s]=cell_warped_radial_fields(cell_density(instr_a(i)),axis,dir,0.,1.).xyz;
   if abs(length(dir)-1.)>.00000025 {flags+=1.;}
   // INJECT_CHART_JUMP
  }
  var across=vec2f(0.);var same=vec2f(0.);
  for(var c=0u;c<4u;c+=1u) {
   across.x=max(across.x,max(abs(paint[1][c]-paint[3][c]),max(abs(paint[1][c]-paint[2][c]),abs(paint[3][c]-paint[2][c]))));
   across.y=max(across.y,max(abs(regions[1][c]-regions[3][c]),max(abs(regions[1][c]-regions[2][c]),abs(regions[3][c]-regions[2][c]))));
   same.x=max(same.x,max(abs(paint[0][c]-paint[1][c]),abs(paint[3][c]-paint[4][c])));
   same.y=max(same.y,max(abs(regions[0][c]-regions[1][c]),abs(regions[3][c]-regions[4][c])));
  }
  let span=length(dirs[1]-dirs[3]);
  if all(dirs[1]==dirs[3]) || (scale>0u && span>=prior_span*.3) {flags+=2.;}
  if scale>=3u && any(across>max(previous*.35,same*2.)+vec2f(.001)) {flags+=4.;}
  if scale==4u && any(geometry[1].xy!=geometry[3].xy) && min(geometry[1].z,geometry[3].z)>.005 {flags+=8.;}
  last=across;previous=across;prior_span=span;final_span=span;
 }
 if any(last>vec2f(.01,.005)) {flags+=16.;}
 textureStore(result,p.xy,vec4f(last,flags,final_span*1000000.));
}
