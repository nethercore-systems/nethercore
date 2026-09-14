// Experimental exact Euclidean boundary distance of floor-owned HEX polygons.
// ponytail: exhaustive columns and three rows/images; optimize only after contract approval.
struct HexPoly { v: array<vec2f, 12>, n: u32 }
fn hex_clip_x(poly: HexPoly, cut: f32, side: f32) -> HexPoly {
 var out: HexPoly;
 for(var i=0u;i<poly.n;i+=1u) {
  let a=poly.v[i]; let b=poly.v[(i+1u)%poly.n];
  let da=(a.x-cut)*side; let db=(b.x-cut)*side;
  if da>=0. { out.v[out.n]=a; out.n+=1u; }
  if (da>=0.) != (db>=0.) {
   out.v[out.n]=mix(a,b,da/(da-db)); out.n+=1u;
  }
 }
 return out;
}
fn hex_polygon(k: f32, row: f32, density: f32) -> HexPoly {
 let offset=select(0.,.5,fract(row*.5)<.25);
 let c=vec2f(k+.5-offset,row+.5);
 let a=1.-.866; let b=.25/.866;
 var p:HexPoly; p.n=8u;
 p.v[0]=c+vec2f(-a,-.5);p.v[1]=c+vec2f(a,-.5);
 p.v[2]=c+vec2f(.5,-b);p.v[3]=c+vec2f(.5,b);
 p.v[4]=c+vec2f(a,.5);p.v[5]=c+vec2f(-a,.5);
 p.v[6]=c+vec2f(-.5,b);p.v[7]=c+vec2f(-.5,-b);
 return hex_clip_x(hex_clip_x(p,0.,1.),density,-1.);
}
fn hex_poly_sd(p:vec2f,poly:HexPoly)->f32 {
 var distance=1000.;var inside=true;
 for(var i=0u;i<poly.n;i+=1u) {
  let a=poly.v[i];let b=poly.v[(i+1u)%poly.n];let e=b-a;
  let q=p-a;let len2=dot(e,e);
  if len2>1e-14 {
   distance=min(distance,length(q-e*clamp(dot(q,e)/len2,0.,1.)));
   inside=inside && (e.x*q.y-e.y*q.x>=0.);
  }
 }
 return select(-distance,distance,inside);
}
fn cell_hex_2d(uv:vec2f,density:f32)->vec3f {
 if abs(density-round(density))<=.00001 {return cell_hex(uv,density);}
 let p=uv*density;var best=-1000.;var owner=vec2f(0.);
 for(var row=i32(floor(p.y))-1;row<=i32(floor(p.y))+1;row+=1) {
  for(var k=0;k<=i32(ceil(density));k+=1) {
   let poly=hex_polygon(f32(k),f32(row),density);
   if poly.n<3u {continue;}
   for(var image=-1;image<=1;image+=1) {
    let sd=hex_poly_sd(p-vec2f(f32(image)*density,0.),poly);
    if sd>best {best=sd;owner=vec2f(f32(k),f32(row));}
   }
  }
 }
 return vec3f(owner,best);
}
