// Full production FLOW at floor-lane ownership boundaries; turbulence disabled.
@group(0) @binding(0) var result: texture_storage_2d<rgba16float,write>;
fn lane_direction(across:f32,along:f32,lane_freq:f32)->vec3f {
    let up=vec3f(0.0,1.0,0.0);
    let flow=decode_dir16(0x80ffu);
    let along_axis=normalize(flow-up*dot(flow,up));
    let across_axis=cross(up,along_axis);
    return normalize(along_axis*along-up+across_axis*(across/lane_freq));
}
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let level=p.y%4u;
    let along_id=(p.y/4u)%16u;
    let lane=i32((p.y/64u)%32u)-16;
    let scale_id=p.y/(64u*32u);
    let pa=array<u32,3>(0u,128u,255u)[scale_id];
    let scale_i=1u+(pa*15u)/255u;
    let lane_freq=6.0+f32(scale_i)*1.5;
    let eps=array<f32,4>(0.001,0.0003,0.0001,0.00003)[level];
    let along=(f32(along_id)-7.5)*0.13;
    if p.x==10u {
        let left=lane_direction(f32(lane)-eps,along,lane_freq);
        let right=lane_direction(f32(lane)+eps,along,lane_freq);
        let instr=vec4u((0x80ffu<<8u)|0xfbu,(128u<<24u)|(pa<<16u)|0x21u,
                        0xaa336699u,(11u<<27u)|(7u<<24u)|(3u<<21u)|0x9966u);
        let xl=flow_lane_coordinate(left,instr,vec3f(0.0,1.0,0.0),1.0).rgb.x;
        let xr=flow_lane_coordinate(right,instr,vec3f(0.0,1.0,0.0),1.0).rgb.x;
        textureStore(result,p.xy,vec4f((xl-f32(lane))*1000000.0,(xr-f32(lane))*1000000.0,floor(xl),floor(xr)));
        return;
    }
    let point=p.x%5u;
    let offset=array<f32,5>(-1.0,0.0,1.0,-3.0,3.0)[point]*eps;
    let dir=lane_direction(f32(lane)+offset,along,lane_freq);
    let instr=vec4u((0x80ffu<<8u)|0xfbu,(128u<<24u)|(pa<<16u)|0x21u,
                    0xaa336699u,(11u<<27u)|(7u<<24u)|(3u<<21u)|0x9966u);
    let sample=eval_flow(dir,instr,vec3f(0.0,1.0,0.0),1.0);
    var value=vec4f(sample.rgb*sample.w,sample.w);
    if p.x>=5u && (point==1u || point==2u || point==4u) { value.x+=2.0; }
    textureStore(result,p.xy,value);
}
