#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}

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

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var base_color_day_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var base_color_day_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(102) var base_color_night_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(103) var base_color_night_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(104) var<uniform> sun_dir: vec3f;
@group(#{MATERIAL_BIND_GROUP}) @binding(105) var<uniform> emission_strength: f32;
@group(#{MATERIAL_BIND_GROUP}) @binding(106) var<uniform> emission_threshold: f32;
@group(#{MATERIAL_BIND_GROUP}) @binding(107) var<uniform> transition_fraction: f32;

const PI: f32 = 3.1415;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    let uv = vec2f(in.uv.x, in.uv.y);

    let sun_dot = dot(normalize(in.world_position.xyz), sun_dir);

    // Transition between day and night
    if sun_dot > 0.0 && sun_dot < transition_fraction {
        let day_color = textureSampleLevel(base_color_day_texture, base_color_day_sampler, uv, 0.0);
        let night_color = textureSampleLevel(base_color_night_texture, base_color_night_sampler, uv, 0.0);

        // transition of 0 means night, 1 means day
        let transition = sun_dot / transition_fraction;

        pbr_input.material.base_color = (1.0 - transition) * night_color + transition * day_color;

        if night_color.r > emission_threshold {
            pbr_input.material.emissive = vec4f(emission_strength) * night_color * (1.0 - transition);
            pbr_input.material.emissive.a = 1.0;
        }
    // Night
    } else if sun_dot < 0.0 {
        let night_color = textureSampleLevel(base_color_night_texture, base_color_night_sampler, uv, 0.0);
        pbr_input.material.base_color = night_color;

        if night_color.r > emission_threshold {
            pbr_input.material.emissive = vec4f(emission_strength) * night_color;
            pbr_input.material.emissive.a = 1.0;
        }
    // Day
    } else  {
        pbr_input.material.base_color = textureSampleLevel(base_color_day_texture, base_color_day_sampler, uv, 0.0);
    }

#ifdef PREPASS_PIPELINE
    // in deferred mode we can't modify anything after that, as lighting is run in a separate fullscreen shader.
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    // apply lighting
    out.color = apply_pbr_lighting(pbr_input);

    // we can optionally modify the lit color before post-processing is applied
    // out.color = vec4<f32>(vec4<u32>(out.color * f32(my_extended_material.quantize_steps))) / f32(my_extended_material.quantize_steps);

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    // we can optionally modify the final result here
    out.color = out.color * 2.0;
#endif

    return out;
}