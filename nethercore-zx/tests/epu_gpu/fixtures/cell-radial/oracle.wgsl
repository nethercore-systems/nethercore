fn radial_max(v:vec3f)->f32 {return max(v.x,max(v.y,v.z));}
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
 let byte=p.y%256u;let hemisphere=(p.y/256u)%2u;let fill=array<u32,3>(0u,128u,255u)[p.y/512u];
 let seed=array<u32,4>(0u,1u,127u,255u)[(p.x/8u)%4u];
 let columns=round(cell_density(byte));let rings=f32((u32(columns)+1u)/2u);
 let radius_cell=array<f32,8>(0.,.25,.999999,1.,1.000001,1.5,2.,rings)[p.x%8u];
 let angle_cell=array<f32,8>(-.00001,0.,.00001,.37,1.,1.37,columns-1.,columns-.37)[p.x/8u];
 let radius=min(radius_cell/rings,1.);let theta=(angle_cell/columns-.5)*TAU;
 let axis_bits=array<u32,3>(0xff80u,0x80ffu,0xe0a0u)[byte%3u];let axis=decode_dir16(axis_bits);
 let reference=select(vec3f(0.,1.,0.),vec3f(1.,0.,0.),abs(axis.y)>.9);
 let t=normalize(cross(reference,axis));let b=normalize(cross(axis,t));
 let dir=normalize(axis*select(1.,-1.,hemisphere==1u)*sqrt(max(0.,1.-radius*radius))+radius*(t*cos(theta)+b*sin(theta)));
 let uv=cell_axis_cylinder_uv(dir,axis);let x=uv.x*columns;let y=min(1.,length(cross(dir,axis)))*rings;
 var expected_owner=vec2f(f32(u32(floor(x))%u32(columns)),min(floor(y),rings-1.));
 if y<1. {expected_owner=vec2f(0.);}
 // Independent full annular-sector enumeration, not the production local stencil.
 var geometric=1.-y;var painted=select(-100.,geometric,cell_hash2(vec2f(0.),f32(seed))<u8_to_01(fill));
 for(var ring=1u;ring<u32(rings);ring+=1u) {
  for(var spoke=0u;spoke<u32(columns);spoke+=1u) {
   let center=f32(spoke)+.5;let delta=x-center-round((x-center)/columns)*columns;
   let edge=min(.5-abs(delta),min(y-f32(ring),f32(ring)+1.-y));
   geometric=max(geometric,edge);
   if cell_hash2(vec2f(f32(spoke),f32(ring)),f32(seed))<u8_to_01(fill) {painted=max(painted,edge);}
  }
 }
 let i=vec4u((seed<<24u)|(axis_bits<<8u)|255u,(byte<<16u)|(fill<<8u),
  0x58c09050u,(OP_CELL<<27u)|(7u<<24u)|(3u<<21u)|(3u<<16u)|0x1828u);
 let geometry=cell_radial(uv,cell_density(byte),axis,dir);
 let q=evaluate_bounds_layer(dir,i,OP_CELL,axis,RegionWeights(.2,.3,.5));
 let expected=regions_from_signed_distance(painted,.005);
 let rgb=instr_color_a(i)*expected.sky+instr_color_b(i)*(expected.wall+.5*expected.floor);
 var layers:array<vec4u,8>;layers[0]=i;
 let actual_rgb=evaluate_epu_layers(dir,layers);
 textureStore(result,p.xy,vec4f(radial_max(abs(geometry-vec3f(expected_owner,geometric))),
  radial_max(abs(vec3f(q.regions.sky,q.regions.wall,q.regions.floor)-vec3f(expected.sky,expected.wall,expected.floor))),
  max(radial_max(abs(q.sample.rgb*q.sample.w-rgb)),radial_max(abs(actual_rgb-rgb))),expected.floor));
}
