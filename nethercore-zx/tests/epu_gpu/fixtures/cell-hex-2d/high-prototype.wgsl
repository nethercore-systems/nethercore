// QA ONLY: algebraic chart intersection/continuation, not Euclidean distance.
fn high_hex(uv:vec2f,density:f32)->vec3f {
 let old=cell_hex(uv,density);
 if abs(density-round(density))<=.00001 {return old;}
 let x=uv.x*density;let y=uv.y*density;
 let shift=select(0.,.5,fract(floor(y)*.5)<.25);
 var best=vec3f(old.xy,min(old.z,min(x,density-x)));
 // Only the two endpoint owner charts; never clip row borders.
 for(var side=0u;side<2u;side+=1u) {
  let endpoint=select(0.,density,side==1u);
  let id=floor(endpoint+shift-select(0.,.00001,side==1u));
  let image_x=select(x-density,x+density,side==1u);
  let cx=image_x+shift-id-.5;
  let field=.5-max(abs(cx),.866*abs(fract(y)-.5)+.5*abs(cx));
  let clipped=min(field,select(x-density,-x,side==1u));
  if clipped>best.z {best=vec3f(id,floor(y),clipped);}
 }
 return best;
}
