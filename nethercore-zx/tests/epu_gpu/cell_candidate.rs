//! Integral candidate oracle: enumerate every canonical column, not a moving window.
use super::*;

#[test]
fn cell_candidate_topology_oracle() {
    let pixels = probe_image(
        r#"
fn candidate_oracle(uv:vec2f,n:i32,seed:f32,shatter:bool,local:bool)->vec4f {
    let scaled=uv*f32(n);let base=vec2i(floor(scaled));
    var first=100.;var second=100.;var owner=vec2f(0.);
    for(var y=base.y-2;y<=base.y+2;y+=1) {
        for(var x=0;x<n;x+=1) {
            let delta_id=(x-base.x+n)%n;
            if local && (abs(y-base.y)>1 || (delta_id>1 && delta_id<n-1)) {continue;}
            let id=vec2f(f32(x),f32(y));
            let j=cell_hash2_vec2(id,seed+select(0.,42.,shatter));
            let point=id+j*select(.8,1.,shatter)+vec2f(select(.1,0.,shatter));
            let d=length(vec2f(shortest_periodic_delta(scaled.x,point.x,f32(n)),scaled.y-point.y));
            if d<first {second=first;first=d;owner=id;} else if d<second {second=d;}
        }
    }
    return vec4f(owner,(second-first)*.5,second);
}
@group(0) @binding(0) var result:texture_storage_2d<rgba16float,write>;
@compute @workgroup_size(1) fn probe(@builtin(global_invocation_id) p:vec3u) {
    let n=i32(4u+4u*(p.y%16u));let shatter=(p.y/16u)%2u==1u;
    let seed=array<f32,4>(0.,1.,127.,255.)[p.y/32u];
    let index=p.x/4u;
    let x=f32(index%16u)/16.;let y=f32(index/16u)*.25+.13;
    let uv=vec2f(x,y);
    let oracle=candidate_oracle(uv,n,seed,shatter,false);
    let local=candidate_oracle(uv,n,seed,shatter,true);
    var actual=cell_voronoi(uv,f32(n),seed);
    if shatter {actual=cell_shatter(uv,f32(n),seed);}
    let owner_ok=all(actual.xy==oracle.xy) || oracle.z<.00001;
    let hash_jump=length(cell_hash2_vec2(vec2f(-1.,0.),seed)-cell_hash2_vec2(vec2f(f32(n-1),0.),seed));
    var output=vec4f(select(0.,1.,owner_ok && abs(actual.z-oracle.z)<.00001),abs(local.z-oracle.z),hash_jump,oracle.z);
    if p.x%4u==1u {output=vec4f(actual,0.);}
    if p.x%4u==2u {output=oracle;}
    if p.x%4u==3u {output=local;}
    textureStore(result,p.xy,output);
}
"#,
        1024,
        128,
    );
    let mut wrong = Vec::new();
    let mut missed_second = 0;
    let mut hash_jumps = 0;
    let mut positive = 0;
    let mut edges = 0;
    for (i, group) in pixels.chunks_exact(4).enumerate() {
        let p = group[0];
        assert!(p.iter().all(|x| x.is_finite()));
        if p[0] != 1. {
            if wrong.len() < 16 {
                println!("ORACLE_DIAG {i}: {group:?}");
            }
            wrong.push(i);
        }
        if p[1] > 0.00001 {
            missed_second += 1;
        }
        if p[2] > 0.01 {
            hash_jumps += 1;
        }
        if p[3] > 0.02 {
            positive += 1;
        }
        if p[3] < 0.005 {
            edges += 1;
        }
    }
    println!(
        "CELL_CANDIDATE_ORACLE wrong={} canonical_3x3_misses={missed_second} unwrapped_hash_jumps={hash_jumps} positive={positive} near_edges={edges} examples={:?}",
        wrong.len(),
        &wrong[..wrong.len().min(16)]
    );
    assert!(missed_second > 0 && hash_jumps > 0 && positive > 0 && edges > 0);
    assert!(
        wrong.is_empty(),
        "production differs from unique-column nearest/second-nearest oracle"
    );
}
