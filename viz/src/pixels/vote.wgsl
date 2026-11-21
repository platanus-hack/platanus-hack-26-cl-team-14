// Four samples vote for one swatch. Depth outlines belong to the front surface.
@group(0) @binding(0) var scene: texture_2d<f32>;
@group(0) @binding(1) var depth: texture_depth_2d;

// Paper, graphite, violet, water, vegetation, lit windows and intervention marks.
const PALETTE = array<u32, 28>(
    0x22273du,0x343c58u,0x4d577au,0x687399u,0x9296bau,0xb2b2d3u,
    0xcbc9e4u,0xe1e0eeu,0xefedf6u,0xfaf9f5u,
    0x534e88u,0x7772a5u,0xa5b7dcu,0x5885bdu,0x315b99u,
    0x4a6369u,0x6d9090u,0xa4bfb3u,0xc4d7c4u,
    0xb09689u,0xd3beb1u,0xf0e4ceu,0xeac281u,0xffe4a4u,
    0xaa4848u,0xe56d5eu,0xeb9c87u,0xe4a942u
);
const BAYER = array<u32,16>(0u,8u,2u,10u,12u,4u,14u,6u,3u,11u,1u,9u,15u,7u,13u,5u);
fn swatch(i:u32)->vec3<f32>{
    let c=PALETTE[i];
    return vec3<f32>(f32((c>>16u)&255u), f32((c>>8u)&255u), f32(c&255u))/255.;
}
fn srgb(rgb_linear:vec3<f32>)->vec3<f32>{
    let x=clamp(rgb_linear,vec3<f32>(0.),vec3<f32>(1.));
    return select(1.055*pow(x,vec3<f32>(1./2.4))-vec3<f32>(.055),12.92*x,x<=vec3<f32>(.0031308));
}
fn decode_srgb(c:vec3<f32>)->vec3<f32>{
    return select(pow((c+vec3<f32>(.055))/1.055,vec3<f32>(2.4)),c/12.92,c<=vec3<f32>(.04045));
}
fn nearest(color:vec3<f32>, bias:f32)->u32{
    let value = dot(color,vec3<f32>(.30,.55,.15));
    let chroma = max(color.r,max(color.g,color.b)) - min(color.r,min(color.g,color.b));
    let paper = .55 * smoothstep(.30,.78,value) * (1.-smoothstep(.28,.65,chroma));
    let washed = mix(color,vec3<f32>(.98,.975,.99),paper) + bias;
    var best=0u; var score=1e8;
    for(var i=0u;i<28u;i++){
        let d=swatch(i)-washed;
        let dist=dot(d*d,vec3<f32>(.30,.55,.15));
        if dist<score { best=i; score=dist; }
    }
    return best;
}
fn z_at(p:vec2<i32>)->f32{
    return textureLoad(depth,clamp(p,vec2<i32>(0),vec2<i32>(textureDimensions(depth))-vec2<i32>(1)),0);
}
fn front_edge(p:vec2<i32>, offset:vec2<i32>)->bool{
    // Reverse-Z: a positive difference marks the near side. Symmetric slopes cancel.
    let c=z_at(p); let a=z_at(p+offset); let b=z_at(p-offset);
    let back=min(a,b);
    return c-back>max(c*.005,1e-8) && c-back>1.8*abs(max(a,b)-c);
}
@fragment fn fragment(@builtin(position) pos:vec4<f32>)->@location(0) vec4<f32>{
    let p=(vec2<i32>(pos.xy)/vec2<i32>(2))*2;
    let offsets=array<vec2<i32>,4>(vec2(0,0),vec2(1,0),vec2(0,1),vec2(1,1));
    var z: array<f32,4>;
    var nearest_z = 0.;
    var covered = 0u;
    var sky = 0u;
    for (var i = 0u; i < 4u; i++) {
        z[i] = z_at(p + offsets[i]);
        nearest_z = max(nearest_z, z[i]);
        if z[i] > 0. { covered++; } else { sky = i; }
    }
    if covered < 2u { return textureLoad(scene, p + offsets[sky], 0); }

    // Reverse-Z separates the near side of a silhouette from its backdrop.
    let front_depth = nearest_z / 1.007;
    var front_count = 0u;
    for (var i = 0u; i < 4u; i++) {
        front_count += select(0u, 1u, z[i] >= front_depth);
    }
    let grid = vec2<u32>(p/2);
    let rank = f32(BAYER[(grid.x & 3u) + 4u * (grid.y & 3u)]);
    let bias = ((rank + .5)/16. - .5) * .35 * .3 * .25;
    var votes: array<u32,4>;
    var keep: array<bool,4>;
    for (var i = 0u; i < 4u; i++) {
        keep[i] = z[i] > 0. && ((z[i] >= front_depth) == (front_count >= 2u));
        if keep[i] { votes[i] = nearest(srgb(textureLoad(scene, p + offsets[i], 0).rgb),bias); }
    }

    var winner = 0u;
    var count = 0u;
    for (var i = 0u; i < 4u; i++) {
        if !keep[i] { continue; }
        var n = 0u;
        for (var j = 0u; j < 4u; j++) {
            if keep[j] && votes[j] == votes[i] { n++; }
        }
        if n > count { count = n; winner = i; }
    }
    var edge = false;
    for (var i = 0u; i < 4u; i++) {
        if keep[i] && votes[i] == votes[winner] {
            let q = p + offsets[i];
            edge = edge || front_edge(q, vec2(2,0)) || front_edge(q, vec2(0,2));
        }
    }
    let fill = swatch(votes[winner]);
    let color = select(fill, mix(fill, swatch(10u), .68), edge);
    return vec4<f32>(decode_srgb(color), 1.);
}
