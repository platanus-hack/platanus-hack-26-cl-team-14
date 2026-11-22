// Print the opaque scene in categorical inks. Scenario edits draw over the print.
@group(0) @binding(0) var scene: texture_2d<f32>;
@group(0) @binding(1) var depth: texture_depth_2d;
struct PrintView {
    up_and_focal: vec4<f32>,
    sun_and_title: vec4<f32>,
    world_from_clip: mat4x4<f32>,
    land: vec4<f32>,
    sky_and_reach: vec4<f32>,
}
@group(0) @binding(2) var<uniform> view: PrintView;

// Ultramarine pen ink on warm paper, from full ink to bare paper.
const INKS = array<u32, 14>(
    0x10158au, 0x1c25a9u, 0x3342d4u, 0x4e5edeu, 0x6575e7u, 0x7f8ef0u,
    0x9da9f4u, 0xb3bdf7u, 0xc5ccfau, 0xd4d9fau, 0xe1e4f9u,
    0xedeef7u, 0xf4f4f6u, 0xfbfaf5u
);
const BAYER = array<u32,16>(0u,8u,2u,10u,12u,4u,14u,6u,3u,11u,1u,9u,15u,7u,13u,5u);

// At night the saturated ultramarine reads as neon, so the dark print keeps a muted navy ramp.
const NIGHT_INKS = array<u32, 14>(
    0x242942u, 0x343957u, 0x50557au, 0x666d98u, 0x7a81adu, 0x9099c2u,
    0xa9b2d6u, 0xbcc4e2u, 0xccd2e9u, 0xdadeefu, 0xe5e8f4u,
    0xeff1f8u, 0xf5f6fbu, 0xfdfcf9u
);

fn ink(index: u32, dark: bool) -> vec3<f32> {
    var c = INKS[index];
    if dark { c = NIGHT_INKS[index]; }
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

fn print_city(pos: vec4<f32>, dark: bool, walls: bool) -> vec4<f32> {
    let p = vec2<i32>(pos.xy);
    let z = z_at(p);
    if z == 0. { return textureLoad(scene, p, 0); }
    let dx = vec2<i32>(1,0);
    let dy = vec2<i32>(0,1);
    // One-sided curvature traces the edge once. Reuse the same samples for planar faces.
    let horizontal = vec2<f32>(z_at(p-dx), z_at(p+dx));
    let tolerance = max(.00022*z, 1e-9);
    let bend_x = 2.*z - horizontal.x - horizontal.y;
    var crease = bend_x > tolerance;
    var vertical = vec2<f32>(z);
    var bend_y = 0.;
    if !crease {
        vertical = vec2<f32>(z_at(p-dy), z_at(p+dy));
        bend_y = 2.*z - vertical.x - vertical.y;
        crease = bend_y > tolerance;
    }
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
        if walls && violet && l < .32 && !boundary && max(abs(bend_x), abs(bend_y)) <= tolerance {
            // Reverse depth on a perspective-projected plane is affine in pixel coordinates.
            // Its two slopes give the geometric normal, including for the coarse building boxes.
            let slope = .5 * vec2<f32>(horizontal.y-horizontal.x, vertical.y-vertical.x);
            let center = .5 * vec2<f32>(textureDimensions(depth));
            let n = normalize(vec3<f32>(view.up_and_focal.w * vec2<f32>(-slope.x, slope.y),
                                         z - dot(pos.xy-center, slope)));
            let sun = dot(n, view.sun_and_title.xyz);
            if abs(dot(n, view.up_and_focal.xyz)) < .25 && sun < .15 {
                // A wall turned away from the sun prints as one solid ink. The darkest walls take a deeper one.
                level = select(2u, 1u, sun < -.35 && l < .12);
                outline = min(outline, level);
            }
        }
        // The shadow that a building casts on the ground prints as one solid ink too.
        if !violet && level >= 6u && level <= 9u {
            level = 5u;
        }
    }
    let shade = select(level, outline, crease || boundary);
    let base = ink(shade, dark);
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
        // Day flora is a bright matcha green.
        color = select(vec3<f32>(.46,.66,.15), vec3<f32>(.65,1.25,.80), dark) * v;
    } else if rgb.g > 1.9*rgb.r && rgb.b > 1.25*rgb.g {
        color = vec3<f32>(.32, .67, .95) * v;
    } else if neutral {
        let gray = vec3<f32>(.90, .92, .94) * v;
        color = select(mix(gray, base, .35), gray, dark);
    }
    // Past the surveyed rectangle the mirrored land dissolves into the sky.
    let ndc = vec2<f32>(pos.x, -pos.y) / vec2<f32>(textureDimensions(depth)) * 2. + vec2<f32>(-1., 1.);
    let world = view.world_from_clip * vec4<f32>(ndc, z, 1.);
    let ground = world.xz / world.w;
    let beyond = length(max(max(view.land.xy - ground, ground - view.land.zw), vec2<f32>(0.)));
    color = mix(color, view.sky_and_reach.rgb, smoothstep(0., view.sky_and_reach.w, beyond));
    return vec4<f32>(color, 1.);
}

@fragment fn day(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    return print_city(pos, false, true);
}

@fragment fn night(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    return print_city(pos, true, false);
}

@fragment fn title(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    return mix(print_city(pos, false, false), print_city(pos, true, false), view.sun_and_title.w);
}
