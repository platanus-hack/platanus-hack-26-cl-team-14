// Print the opaque scene in categorical inks. Scenario edits draw over the print.
@group(0) @binding(0) var scene: texture_2d<f32>;
@group(0) @binding(1) var depth: texture_depth_2d;
@group(0) @binding(2) var<uniform> title_light: vec4<f32>;

const INKS = array<u32, 14>(
    0x242942u, 0x343957u, 0x50557au, 0x666d98u, 0x7a81adu, 0x9099c2u,
    0xa9b2d6u, 0xbcc4e2u, 0xccd2e9u, 0xdadeefu, 0xe5e8f4u,
    0xeff1f8u, 0xf5f6fbu, 0xfdfcf9u
);
const BAYER = array<u32,16>(0u,8u,2u,10u,12u,4u,14u,6u,3u,11u,1u,9u,15u,7u,13u,5u);

fn ink(index: u32) -> vec3<f32> {
    let c = INKS[index];
    let s = vec3<f32>(f32((c >> 16u) & 255u), f32((c >> 8u) & 255u), f32(c & 255u)) / 255.;
    return pow((s + .055) / 1.055, vec3<f32>(2.4));
}

fn pixel(p: vec2<i32>) -> vec2<i32> {
    return clamp(p, vec2<i32>(0), vec2<i32>(textureDimensions(depth)) - 1);
}

fn z_at(p: vec2<i32>) -> f32 {
    return textureLoad(depth, pixel(p), 0);
}

fn light(p: vec2<i32>) -> f32 {
    let rgb = max(textureLoad(scene, pixel(p), 0).rgb, vec3<f32>(0.));
    return dot(rgb, vec3<f32>(.30,.55,.15));
}

// The second depth difference vanishes on a planar roof or street. Its sign
// selects one side of the silhouette or fold, keeping the line one sample wide.
fn depth_line(c: f32, a: f32, b: f32) -> bool {
    return 2.*c - a - b > max(.00022*c, 1e-9);
}

fn print_city(pos: vec4<f32>, dark: bool) -> vec4<f32> {
    let p = vec2<i32>(pos.xy);
    let z = z_at(p);
    if z == 0. { return textureLoad(scene, p, 0); }
    let dx = vec2<i32>(1,0);
    let dy = vec2<i32>(0,1);
    let crease = depth_line(z, z_at(p-dx), z_at(p+dx)) || depth_line(z, z_at(p-dy), z_at(p+dy));
    let rgb = textureLoad(scene, p, 0).rgb;
    let l = dot(rgb, vec3<f32>(.30,.55,.15));
    let neutral = max(max(rgb.r,rgb.g),rgb.b) - min(min(rgb.r,rgb.g),rgb.b) < .20*max(l,.001);
    // One-sided contrast also traces coplanar material boundaries such as curbs.
    let boundary = max(light(p+dx), light(p+dy)) - l > .09;
    let grid = vec2<u32>(p) / 2u;
    let rank = (f32(BAYER[(grid.x & 3u) + 4u*(grid.y & 3u)]) + .5) / 16.;
    var level: u32;
    var outline: u32;
    if dark {
        level = u32(clamp(pow(l,.75)*13. + .30*rank, 0., 13.));
        outline = min(level, 2u);
    } else {
        let value = smoothstep(.024, .56, l);
        // Give shaded violet walls and neutral paving more ink while lit paper still reaches 13.
        let violet = rgb.b > 1.4*max(rgb.r,rgb.g) && rgb.g < 1.5*rgb.r;
        let structure = violet || neutral;
        let shadow = select(7., 4.5, structure);
        level = u32(clamp(mix(shadow, 13.2, value) + rank, 0., 13.));
        outline = select(5u, 3u, crease);
        if structure { outline -= 1u; }
    }
    let shade = select(level, outline, crease || boundary);
    let base = ink(shade);
    let v = dot(base, vec3<f32>(.30,.55,.15));
    // Preserve material hue through the lightness quantization. The source water
    // is cyan-blue, distinct from the violet building ink under both lights.
    var color = base;
    // The unlit actor marks carry their own palette through the print pass.
    if distance(rgb, vec3<f32>(.85, .13, .09)) < .008 || distance(rgb, vec3<f32>(.1651, .0595, .1779)) < .008 {
        color = rgb;
    } else if distance(rgb, vec3<f32>(.0080, .0704, .0931)) < .008 {
        color = rgb;
    } else if rgb.g > 1.18*rgb.r && rgb.g > 1.05*rgb.b {
        color = select(vec3<f32>(.48,.74,.53), vec3<f32>(.65,1.25,.80), dark) * v;
    } else if rgb.g > 1.9*rgb.r && rgb.b > 1.25*rgb.g {
        color = vec3<f32>(.32, .67, .95) * v;
    } else if neutral {
        let gray = vec3<f32>(.90, .92, .94) * v;
        color = select(mix(gray, base, .35), gray, dark);
    }
    return vec4<f32>(color, 1.);
}

@fragment fn day(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    return print_city(pos, false);
}

@fragment fn night(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    return print_city(pos, true);
}

@fragment fn title(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    return mix(print_city(pos, false), print_city(pos, true), title_light.x);
}
