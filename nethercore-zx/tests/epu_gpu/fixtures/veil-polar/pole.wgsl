@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    // 16 azimuths plus the exact pole, at four shrinking transverse radii.
    let az = p.x % 17u; let level = p.x / 17u;
    let variant = p.y % 5u; let pole = (p.y / 5u) % 2u; let axis_case = p.y / 10u;
    let axis_bits = select(0u, 0x8080u, axis_case == 1u);
    let axis = select(vec3f(0.0,1.0,0.0), decode_dir16(axis_bits), axis_bits != 0u);
    let hint = select(vec3f(0.0,1.0,0.0), vec3f(1.0,0.0,0.0), abs(axis.y) > 0.9);
    let t = normalize(cross(hint,axis)); let b = normalize(cross(axis,t));
    let eps = array<f32,4>(0.01,0.001,0.0001,0.00001)[level];
    let theta = f32(az) * TAU / 16.0;
    let radial = select(eps, 0.0, az == 16u);
    let ray = normalize(axis*select(1.0,-1.0,pole==1u) + radial*(t*cos(theta)+b*sin(theta)));
    let instr = vec4u((37u<<24u)|(axis_bits<<8u)|0xf7u,
        (192u<<24u)|(64u<<16u)|(20u<<8u)|128u,0x22u<<24u|0x2244ccu,
        (13u<<27u)|(7u<<24u)|((16u|variant)<<16u)|0xeebb);
    let value = eval_veil(ray,instr,1.0);
    textureStore(result,p.xy,vec4f(value.rgb*value.w,value.w));
}
