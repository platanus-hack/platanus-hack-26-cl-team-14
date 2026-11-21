import bevy_pbr::render::{
    forward_io::VertexOutput,
    mesh_view_bindings as view_bindings,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::main_pass_post_lighting_processing,
    view_transformations::position_world_to_ndc,
};

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var dem: texture_2d<i32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var<uniform> grid: vec4f;
@group(constants::MATERIAL_BIND_GROUP) @binding(102) var<uniform> aboard: u32;

// Height and gradient on the same two triangles as city::terrain::terrain_from_xz.
fn surface(xz: vec2f) -> vec3f {
    let last = vec2i(textureDimensions(dem)) - vec2i(1);
    let uv = clamp((xz - grid.xy) / grid.zw, vec2f(0.), vec2f(last));
    let p = min(vec2i(uv), last - vec2i(1));
    let f = uv - vec2f(p);
    let b = f32(textureLoad(dem, p + vec2i(1,0), 0).r);
    let c = f32(textureLoad(dem, p + vec2i(0,1), 0).r);
    if f.x + f.y <= 1. {
        let a = f32(textureLoad(dem, p, 0).r);
        let slope = vec2f(b-a, c-a);
        return vec3f(a + dot(f, slope), slope / grid.zw);
    }
    let d = f32(textureLoad(dem, p + vec2i(1,1), 0).r);
    let slope = vec2f(d-c,d-b);
    return vec3f(d + dot(f - vec2f(1.), slope), slope / grid.zw);
}

struct FerryOutput {
    @location(0) color: vec4f,
    @builtin(frag_depth) depth: f32,
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) front: bool) -> FerryOutput {
    let pbr = pbr_input_from_standard_material(in, front);
    let color = main_pass_post_lighting_processing(pbr, pbr.material.base_color);
    if aboard == 0u {
        return FerryOutput(color, in.position.z);
    }

    // Keep the actor silhouette. Give its fragments the water-layer depth along
    // their camera ray, between the drawn river (+0.08 m) and road decks (+0.30 m).
    // The opaque deck triangles then occlude both hull and riders at the same pixels.
    let ray = normalize(in.world_position.xyz - view_bindings::view.world_position.xyz);
    var p = in.world_position.xyz;
    for (var i = 0; i < 3; i += 1) {
        let h = surface(p.xz);
        p += ray * ((h.x + 0.15 - p.y) / (ray.y - dot(h.yz, ray.xz)));
    }
    return FerryOutput(color, position_world_to_ndc(p).z);
}
