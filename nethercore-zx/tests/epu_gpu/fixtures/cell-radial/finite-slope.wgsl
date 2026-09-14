// Diagnostic: physical rays at ordinary owner transitions, not just chart cuts.
fn support_dir(uv:vec2f,axis:vec3f)->vec3f {
    let ref_vec=select(vec3f(0.,1.,0.),vec3f(1.,0.,0.),abs(axis.y)>.9);
    let t=normalize(cross(ref_vec,axis));let b=normalize(cross(axis,t));
    let h=uv.y*2.-1.;let phi=(uv.x-.5)*TAU;
    return normalize(axis*h+sqrt(max(0.,1.-h*h))*(t*cos(phi)+b*sin(phi)));
}
// Locator 0 keeps the frozen noisy geometry; locator 1 follows current geometry.
fn support_info(dir:vec3f,i:vec4u,historical:bool)->vec3f {
 let axis=decode_dir16(instr_dir16(i));
 if historical {return radial_before_cell_radial(radial_before_cell_axis_cylinder_uv(dir,axis),radial_before_cell_density(instr_a(i)),axis,dir);}
 return cell_radial(cell_axis_cylinder_uv(dir,axis),cell_density(instr_a(i)),axis,dir);
}
fn site_max(v:vec3f)->f32 { return max(v.x,max(v.y,v.z)); }
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u){
let selected=array<u32,168>(24576u,30720u,32647u,36864u,38791u,43008u,44905u,49336u,49368u,49576u,50920u,50928u,50984u,51136u,51183u,52319u,52688u,52728u,54128u,55480u,55720u,57064u,57072u,57128u,57280u,57327u,58832u,58872u,60016u,60272u,61624u,61656u,61696u,61864u,63208u,63272u,64607u,64976u,65016u,66160u,66416u,66536u,67768u,67800u,67840u,68008u,69360u,69416u,69568u,69615u,70751u,71120u,71160u,72304u,72560u,72680u,98304u,104448u,106375u,110592u,112519u,116736u,118633u,123064u,123096u,123304u,124648u,124656u,124712u,124864u,124911u,126047u,126416u,126456u,127856u,129208u,129448u,130792u,130800u,130856u,131008u,131055u,132560u,132600u,133744u,134000u,135352u,135384u,135424u,135592u,136936u,137000u,138335u,138704u,138744u,139888u,140144u,140264u,141496u,141528u,141568u,141736u,143088u,143144u,143296u,143343u,144479u,144848u,144888u,146032u,146288u,146408u,172032u,178176u,180103u,184320u,186247u,190464u,192361u,196792u,196824u,197032u,198376u,198384u,198440u,198592u,198639u,199775u,200144u,200184u,201584u,202936u,203176u,204520u,204528u,204584u,204736u,204783u,206288u,206328u,207472u,207728u,209080u,209112u,209152u,209320u,210664u,210728u,212063u,212432u,212472u,213616u,213872u,213992u,215224u,215256u,215296u,215464u,216816u,216872u,217024u,217071u,218207u,218576u,218616u,219760u,220016u,220136u);var measured=false;for(var k=0u;k<168u;k+=1u){measured=measured||(p.y*512u+p.x==selected[k]);}if !measured {textureStore(result,p.xy,vec4f(0.));return;}
    let row=p.y%144u;let variant=3u;let historical=p.y<432u;
    let density_byte=array<u32,3>(0u,127u,255u)[(row/48u)%3u];
    let seed=array<u32,4>(0u,1u,127u,255u)[(row/12u)%4u];
    let v=array<f32,4>(.13,.37,.67,.89)[(row/3u)%4u];
    let gap=array<u32,3>(0u,64u,255u)[row%3u];
    let alpha=array<u32,3>(0u,7u,15u)[(p.y/144u)%3u];
    let i=vec4u((seed<<24u)|(0xff80u<<8u)|(alpha<<4u)|15u,
        (96u<<24u)|(density_byte<<16u)|(128u<<8u)|gap,
        (0x58u<<24u)|0xc09050u,(OP_CELL<<27u)|(7u<<24u)|(3u<<21u)|(variant<<16u)|0x1828u);
    let axis=decode_dir16(instr_dir16(i));let n=round(mix(4.,64.,u8_to_01(instr_a(i))));
    var lo=(f32(p.x)+.013)/512.;var hi=lo+1./512.;
    let original=support_info(support_dir(vec2f(lo,v),axis),i,historical);
    let end=support_info(support_dir(vec2f(hi,v),axis),i,historical);
    if all(original.xy==end.xy) {textureStore(result,p.xy,vec4f(0.));return;}
    for(var k=0u;k<24u;k+=1u){
        let mid=(lo+hi)*.5;let info=support_info(support_dir(vec2f(mid,v),axis),i,historical);
        if all(info.xy==original.xy){lo=mid;}else{hi=mid;}
    }
    let root=(lo+hi)*.5;
    var previous=vec2f(1e20);var final_error=vec2f(0.);var invalid=0.;var classification=1.;var oracle_error=0.;
    for(var scale=0u;scale<3u;scale+=1u){
        let eps=array<f32,3>(.001,.0001,.00001)[scale]/n;
        var paint:array<vec4f,5>;var regions:array<vec4f,5>;
        var dirs:array<vec3f,5>;
        for(var s=0u;s<5u;s+=1u){
            let dir=support_dir(vec2f(root+f32(i32(s)-2)*eps,v),axis);dirs[s]=dir;
            let q=eval_cell(dir,i,RegionWeights(.2,.3,.5));
            let oracle=radial_ref_paint(radial_ref_fields(dir,i),i);
            oracle_error=max(oracle_error,max(site_max(abs(q.sample.rgb*q.sample.w-oracle.sample.rgb*oracle.sample.w)),site_max(abs(vec3f(q.regions.sky,q.regions.wall,q.regions.floor)-vec3f(oracle.regions.sky,oracle.regions.wall,oracle.regions.floor)))));
            let dispatch=evaluate_bounds_layer(dir,i,OP_CELL,axis,RegionWeights(.2,.3,.5));
            var layers:array<vec4u,8>;layers[0]=i;
            let composed=evaluate_epu_layers(dir,layers);
            let blend=apply_blend(vec3f(0.),q.sample,BLEND_LERP);
            if site_max(abs(composed-blend))>.002 || site_max(abs(q.sample.rgb-dispatch.sample.rgb))>.002
                || abs(q.sample.w-dispatch.sample.w)>.002 {invalid+=1.;}
            paint[s]=vec4f(blend,q.sample.w);
            regions[s]=vec4f(q.regions.sky,q.regions.wall,q.regions.floor,0.);
            if abs(q.regions.sky+q.regions.wall+q.regions.floor-1.)>.002 {invalid+=1.;}
        }
        // INJECT_ON_CUT_JUMP
        if all(dirs[1]==dirs[3]) {invalid+=1.;}
        var across=vec2f(0.);var same=vec2f(0.);
        for(var a=1u;a<4u;a+=1u){for(var b=a+1u;b<4u;b+=1u){
            let rgb=abs(paint[a]-paint[b]);
            across=max(across,vec2f(max(site_max(rgb.xyz),rgb.w),site_max(abs(regions[a].xyz-regions[b].xyz))));
        }}
        for(var side=0u;side<2u;side+=1u){let a=side*3u;let b=a+1u;
            let rgb=abs(paint[a]-paint[b]);
            same=max(same,vec2f(max(site_max(rgb.xyz),rgb.w),site_max(abs(regions[a].xyz-regions[b].xyz))));
        }
        if scale>0u && any(across>previous*.35+vec2f(.001)) {invalid+=1.;}
        previous=across;
        if scale==2u {
            final_error=max(across,same);
            // Keep and measure ambiguous final owner pairs; never exclude them.
            let a=support_info(dirs[1],i,historical);let b=support_info(dirs[3],i,historical);
            classification=select(1.,2.,(cell_hash2(a.xy,f32(seed))<128./255.)!=(cell_hash2(b.xy,f32(seed))<128./255.));
            if all(a.xy==b.xy) {classification= -classification;}
        }
    }
    textureStore(result,p.xy,vec4f(final_error,oracle_error,classification));
}
