// Independent exhaustive annular-cell oracle: never call the production stencil.
fn radial_ref_fields(dir:vec3f,i:vec4u)->vec4f {
 let axis=decode_dir16(instr_dir16(i));let columns=round(cell_density(instr_a(i)));let rings=f32((u32(columns)+1u)/2u);
 let uv=cell_axis_cylinder_uv(dir,axis);let x=uv.x*columns;let y=min(1.,length(cross(dir,axis)))*rings;
 var owner=vec2f(f32(u32(floor(x))%u32(columns)),min(floor(y),rings-1.));if y<1. {owner=vec2f(0.);}
 let seed=f32(instr_d(i));let fill=u8_to_01(instr_b(i));
 var geometric=1.-y;var painted=select(-100.,geometric,cell_hash2(vec2f(0.),seed)<fill);
 for(var ring=1u;ring<u32(rings);ring+=1u){for(var spoke=0u;spoke<u32(columns);spoke+=1u){
  let center=f32(spoke)+.5;let delta=x-center-round((x-center)/columns)*columns;
  let edge=min(.5-abs(delta),min(y-f32(ring),f32(ring)+1.-y));geometric=max(geometric,edge);
  if cell_hash2(vec2f(f32(spoke),f32(ring)),seed)<fill {painted=max(painted,edge);}
 }}
 return vec4f(owner,geometric,painted);
}
fn radial_ref_paint(fields:vec4f,i:vec4u)->BoundsResult {
 let gap=u8_to_01(instr_c(i))*.2;let d=fields.w-gap;
 let regions=regions_from_signed_distance(d,max(.005,gap*.5));
 let sky=regions.sky*instr_alpha_a_f32(i);let weight=sky+regions.wall+regions.floor;
 let color_a=instr_color_a(i);let color_b=instr_color_b(i);
 var premultiplied=color_a*sky+color_b*(regions.wall+.5*regions.floor);
 if gap>0. && fields.w> -99. {
  let t=clamp(1.-abs(d)/(gap*.3),0.,1.);
  premultiplied+=max(color_a,color_b)*(t*t*(3.-2.*t))*instr_alpha_b_f32(i)*u8_to_01(instr_intensity(i));
 }
 var rgb=vec3f(0.);if weight>0. {rgb=premultiplied/weight;}
 return BoundsResult(LayerSample(rgb,clamp(weight,0.,1.)),regions,1.);
}
