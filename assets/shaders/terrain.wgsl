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
const EPSILON = 0.0001;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
};

struct TerrainMaterial {
}

@group(3) @binding(100)
var<uniform> terrain_material: TerrainMaterial;


@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let model = get_world_from_local(vertex.instance_index);
    let position = vertex.position;

    var out: VertexOutput;
    out.instance_index = vertex.instance_index;
    out.world_position = mesh_position_local_to_world(model, vec4<f32>(position, 1.0));

    out.position = position_world_to_clip(out.world_position.xyz);
    out.world_normal = vertex.world_normal;
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