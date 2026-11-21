// Four samples vote for one swatch. Depth outlines belong to the front surface.
@group(0) @binding(0) var scene: texture_2d<f32>;
@group(0) @binding(1) var depth: texture_depth_2d;

// Santiago roof clay, plaster, glass, vegetation, roads, water and transit.
const PALETTE = array<u32, 20>(
    0x272c36u,0x465265u,0x687581u,0x8c959au,0xb6b6adu,0xe7e0cbu,
    0x774c44u,0xa5674fu,0xcd8d69u,0xe4b28bu,0xa79a79u,0xc8bfa1u,
    0x374f47u,0x56705au,0x748a67u,0xa2ad7cu,0x456e80u,0x78a1a9u,
    0xb74636u,0xe5bc53u
);
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
fn nearest(c:vec3<f32>)->u32{
    var best=0u; var score=1e8;
    for(var i=0u;i<20u;i++){
        let d=swatch(i)-c;
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
    var votes:array<u32,4>;
    for(var i=0u;i<4u;i++){votes[i]=nearest(srgb(textureLoad(scene,p+offsets[i],0).rgb));}
    var winner=0u;var count=0u;
    for(var i=0u;i<4u;i++){
        var n=0u;
        for(var j=0u;j<4u;j++){if votes[j]==votes[i]{n++;}}
        if n>count {count=n;winner=i;}
    }
    var edge=false;
    for(var i=0u;i<4u;i++){
        if votes[i]==votes[winner]{
            let q=p+offsets[i];
            edge=edge||front_edge(q,vec2(2,0))||front_edge(q,vec2(0,2));
        }
    }
    let color=swatch(votes[winner])*select(1.,.72,edge);
    return vec4<f32>(decode_srgb(color),1.);
}
