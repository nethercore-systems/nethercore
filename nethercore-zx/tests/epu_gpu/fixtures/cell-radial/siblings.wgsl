@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u){
    let variant=p.y/144u;let row=p.y%144u;
    let fill_byte=array<u32,3>(0u,128u,255u)[row%3u];
    let alpha=array<u32,3>(0u,7u,15u)[(row/3u)%3u];
    let gap=array<u32,4>(0u,1u,64u,255u)[(row/9u)%4u];
    let seed=array<u32,4>(0u,1u,127u,255u)[row/36u];
    let density_byte=p.x;
    let i=vec4u((seed<<24u)|(0xff80u<<8u)|(alpha<<4u)|15u,
        (96u<<24u)|(density_byte<<16u)|(fill_byte<<8u)|gap,
        (0x58u<<24u)|0xc09050u,(OP_CELL<<27u)|(7u<<24u)|(3u<<21u)|(variant<<16u)|0x1828u);
    let axis=decode_dir16(instr_dir16(i));
    let dir=support_dir(vec2f((f32(p.x)+.371)/256.,fract(f32(row)*.381966+.137)),axis);
    let uv=cell_axis_cylinder_uv(dir,axis);let density=mix(4.,64.,u8_to_01(density_byte));
    let fill=u8_to_01(fill_byte);let gap_width=u8_to_01(gap)*.2;
    let q=eval_cell(dir,i,RegionWeights(.2,.3,.5));
    var old=radial_before_eval_cell(dir,i,RegionWeights(.2,.3,.5));
    var radial_reference=vec4f(0.);
    if variant==3u {radial_reference=radial_ref_fields(dir,i);old=radial_ref_paint(radial_reference,i);}
    // Selector 6 is deliberately defined now; keep all original rays and limits.
    if variant==6u {
        var expected_i=i;expected_i.w=(i.w&0xffe0ffffu)|(3u<<16u);
        old=warpref_eval_cell(dir,expected_i,RegionWeights(.2,.3,.5));
    }
    // OLD_SELECTOR_SIX_FALLBACK_CONTROL
    let is_site=false; // Exact preservation for 0..5/7; frozen organic reference for 6.
    var info=vec3f(0.);var old_info=vec3f(0.);
    if variant==2u {info=cell_voronoi(uv,density,f32(seed));old_info=radial_before_cell_voronoi(uv,density,f32(seed));}
    if variant==4u {info=cell_shatter(uv,density,f32(seed));old_info=radial_before_cell_shatter(uv,density,f32(seed));}
    if variant==3u {info=cell_radial(uv,cell_density(density_byte),axis,dir);old_info=radial_reference.xyz;}
    if variant==6u {
        info=cell_warped_radial_fields(cell_density(density_byte),axis,dir,f32(seed),fill).xyz;
        old_info=warpref_cell_radial_fields(uv,cell_density(density_byte),axis,dir,f32(seed),fill).xyz;
    }
    let geometry_bad=any(info.xy!=old_info.xy)||abs(info.z-old_info.z)>.00001;
    let solid=cell_hash2(old_info.xy,f32(seed))<fill;
    // Outside the existing signed edge band, empty-owner output is unchanged.
    // F2 bounds the nearest filled site's distance when the nearest site is empty.
    let preserve= !is_site || solid || old_info.z>max(.005,gap_width*.5)-gap_width || fill_byte==0u;
    let rgb=site_max(abs(q.sample.rgb*q.sample.w-old.sample.rgb*old.sample.w));
    let regions=site_max(abs(vec3f(q.regions.sky,q.regions.wall,q.regions.floor)-vec3f(old.regions.sky,old.regions.wall,old.regions.floor)));
    let delta=max(rgb,max(regions,abs(q.sample.w-old.sample.w)));
    textureStore(result,p.xy,vec4f(select(0.,1.,geometry_bad),select(0.,delta,preserve),
        // Compare actual components: subtracting equal products produced a tiny
        // residual even in the frozen old-versus-old no-op control.
        select(0.,1.,!is_site && variant!=3u && (any(q.sample.rgb!=old.sample.rgb) || q.sample.w!=old.sample.w
            || q.regions.sky!=old.regions.sky || q.regions.wall!=old.regions.wall || q.regions.floor!=old.regions.floor)),
        select(0.,1.,q.regions.floor>.1)+select(0.,2.,is_site && !preserve && delta>.00001)));
}
