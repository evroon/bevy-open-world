#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
    view_transformations::position_world_to_clip,
}
#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_world, mesh_normal_local_to_world}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif
#import bevy_open_world::common
const EPSILON = 0.0001;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
};

const VERTICES_PER_NODE = 8.0;
const CELL_VERTEX_SPACING = 1.0 / VERTICES_PER_NODE;

struct PlanetMaterial {
    planet_radius: f32,
}

@group(2) @binding(100)
var<uniform> planet_material: PlanetMaterial;

// https://github.com/bevy-interstellar/wgsl_noise
fn noise_fade_vec2f(x: vec2<f32>) -> vec2<f32> {
    return x * x * x * (x * (x * 6.0 - 15.0) + 10.0);
}

fn noise_permute_vec4f(x: vec4<f32>) -> vec4<f32> {
    return (((x * 34.0) + 10.0) * x) % 289.0;
}

fn noise_perlin_vec2f(p: vec2<f32>) -> f32 {
    var pi = floor(p.xyxy) + vec4(0.0, 0.0, 1.0, 1.0);
    pi = pi % 289.0;    // to avoid trauncation effects in permutation

    let pf = fract(p.xyxy) - vec4(0.0, 0.0, 1.0, 1.0);

    let ix = pi.xzxz;
    let iy = pi.yyww;
    let fx = pf.xzxz;
    let fy = pf.yyww;

    let i = noise_permute_vec4f(noise_permute_vec4f(ix) + iy);

    var gx = fract(i * (1.0 / 41.0)) * 2.0 - 1.0;
    let gy = abs(gx) - 0.5 ;
    let tx = floor(gx + 0.5);
    gx = gx - tx;

    var g00 = vec2(gx.x, gy.x);
    var g10 = vec2(gx.y, gy.y);
    var g01 = vec2(gx.z, gy.z);
    var g11 = vec2(gx.w, gy.w);

    let norm = inverseSqrt(vec4(
        dot(g00, g00),
        dot(g01, g01),
        dot(g10, g10),
        dot(g11, g11)
    ));
    g00 *= norm.x;
    g01 *= norm.y;
    g10 *= norm.z;
    g11 *= norm.w;

    let n00 = dot(g00, vec2(fx.x, fy.x));
    let n10 = dot(g10, vec2(fx.y, fy.y));
    let n01 = dot(g01, vec2(fx.z, fy.z));
    let n11 = dot(g11, vec2(fx.w, fy.w));

    let fade_xy = noise_fade_vec2f(pf.xy);
    let n_x = mix(vec2(n00, n01), vec2(n10, n11), fade_xy.x);
    let n_xy = mix(n_x.x, n_x.y, fade_xy.y);
    return 2.3 * n_xy;
}

fn get_height(pos: vec2f) -> f32 {
    return noise_perlin_vec2f(pos) * 0.1;
}


@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let model = get_world_from_local(vertex.instance_index);
    let position = vertex.position;

    var out: VertexOutput;
    out.instance_index = vertex.instance_index;
    out.world_position = mesh_position_local_to_world(model, vec4<f32>(position, 1.0));// * planet_material.planet_radius;
    out.world_position.y = get_height(out.world_position.xz);

    out.position = position_world_to_clip(out.world_position.xyz);
    out.world_normal = vec3(0.0, 1.0, 0.0);
    out.uv = vertex.tex_coords;
    return out;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // we can optionally modify the input before lighting and alpha_discard is applied
    pbr_input.material.base_color.b = pbr_input.material.base_color.r;

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
    // in deferred mode we can't modify anything after that, as lighting is run in a separate fullscreen shader.
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;

    // apply lighting
    out.color = apply_pbr_lighting(pbr_input);

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    // we can optionally modify the final result here
    out.color = out.color * 2.0;
#endif

    // out.color = textureSampleLevel(position_texture, position_sampler, vec2<f32>(vec2<i32>(in.uv * 128.0)) / 128.0, 0.0) / 128.0;
    // out.color.a = 1.0;
    if abs(in.uv.x) < 0.01 || abs(in.uv.y) < 0.01 {
        out.color = vec4(1.0, 1.0, 1.0, 1.0);
    }

    return out;
}