#import bevy_open_world::common

const WORLEY_RESOLUTION = 32;
const WORLEY_RESOLUTION_F32 = 32.0;

#import bevy_sprite::{
    mesh2d_view_bindings::view,
    mesh2d_view_bindings::globals,
    mesh2d_functions::{get_world_from_local, mesh2d_position_local_to_clip},
}
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

// Atmosphere is based on https://www.shadertoy.com/view/wlBXWK


struct Config {
    planet_radius: f32,
    planet_position: vec3f,
    atmosphere_radius: f32,
    atmosphere_rayleigh_beta: vec3f,
    atmosphere_mie_beta: vec3f,
    atmosphere_ambient_beta: vec3f,
    atmosphere_absorption_beta: vec3f,
    atmosphere_height_rayleigh: f32,
    atmosphere_height_mie: f32,
    atmosphere_height_absorption: f32,
    atmosphere_absorption_falloff: f32,
    atmosphere_march_steps: u32,
    atmosphere_light_march_steps: u32,
    clouds_march_steps: u32,
    clouds_self_shadow_steps: u32,
    clouds_bottom: f32,
    clouds_top: f32,
    clouds_coverage: f32,
    clouds_detail_strength: f32,
    clouds_base_edge_softness: f32,
    clouds_bottom_softness: f32,
    clouds_density: f32,
    clouds_shadow_march_step_size: f32,
    clouds_shadow_march_step_multiply: f32,
    clouds_base_scale: f32,
    clouds_details_scale: f32,
    clouds_min_transmittance: f32,
    forward_scattering_g: f32,
    backward_scattering_g: f32,
    scattering_lerp: f32,
    ambient_color_top: vec4f,
    ambient_color_bottom: vec4f,
    sun_dir: vec4f,
    sun_color: vec4f,
    camera_translation: vec3f,
    debug: f32,
    time: f32,
    reprojection_strength: f32,
    render_resolution: vec2f,
    inverse_camera_view: mat4x4f,
    inverse_camera_projection: mat4x4f,
    wind_displacement: vec3f,
};

@group(0) @binding(0) var<uniform> config: Config;

@group(1) @binding(0) var clouds_render_texture: texture_storage_2d<rgba32float, read_write>;
@group(1) @binding(1) var clouds_atlas_texture: texture_storage_2d<rgba32float, read_write>;
@group(1) @binding(2) var clouds_worley_texture: texture_storage_3d<rgba32float, read_write>;
@group(1) @binding(3) var sky_texture: texture_storage_2d<rgba32float, read_write>;

struct RaymarchResult {
    dist: f32,
    color: vec4f,
}


fn calculate_scattering(
    _start: vec3<f32>,
    dir: vec3<f32>,
    max_dist: f32,
    scene_color: vec3<f32>,
    light_dir: vec3<f32>,
    light_intensity: vec3<f32>,
) -> vec3<f32> {
    let g = config.forward_scattering_g;
    let start = _start - config.planet_position;
    var a = dot(dir, dir);
    var b = 2.0 * dot(dir, start);
    var c = dot(start, start) - (config.atmosphere_radius * config.atmosphere_radius);
    var d = (b * b)- 4.0 * a * c;

    // stop early if there is no intersect
    if (d < 0.0) { return scene_color; }

    // calculate the ray length
    var ray_length = vec2(
        max((-b - sqrt(d)) / (2.0 * a), 0.0),
        min((-b + sqrt(d)) / (2.0 * a), max_dist)
    );

    // if the ray did not hit the atmosphere, return a black color
    if (ray_length.x > ray_length.y) { return scene_color; }

    // prevent the mie glow from appearing if there's an object in front of the camera
    let allow_mie = max_dist > ray_length.y;
    // make sure the ray is no longer than allowed
    ray_length.y = min(ray_length.y, max_dist);
    ray_length.x = max(ray_length.x, 0.0);
    // get the step size of the ray
    let step_size_i = (ray_length.y - ray_length.x) / f32(config.atmosphere_march_steps);

    // next, set how far we are along the ray, so we can calculate the position of the sample
    // if the camera is outside the atmosphere, the ray should start at the edge of the atmosphere
    // if it's inside, it should start at the position of the camera
    // the min statement makes sure of that
    var ray_pos_i = ray_length.x + step_size_i * 0.5;

    // these are the values we use to gather all the scattered light
    var total_ray = vec3(0.0); // for rayleigh
    var total_mie = vec3(0.0); // for mie

    // initialize the optical depth. This is used to calculate how much air was in the ray
    var opt_i = vec3(0.0);

    // also init the scale height, avoids some vec2's later on
    let scale_height = vec2(config.atmosphere_height_rayleigh, config.atmosphere_height_mie);

    // Calculate the Rayleigh and Mie phases.
    // This is the color that will be scattered for this ray
    // mu, mumu and gg are used quite a lot in the calculation, so to speed it up, precalculate them
    let mu = dot(dir, light_dir);
    let mumu = mu * mu;
    let gg = g * g;
    let phase_ray = 3.0 / (50.2654824574 /* (16 * pi) */) * (1.0 + mumu);
    var phase_mie = 0.0;
    if allow_mie {
        phase_mie = 3.0
            / (25.1327412287 /* (8 * pi) */)
            * ((1.0 - gg) * (mumu + 1.0))
            / (
                pow(1.0 + gg - 2.0 * mu * g, 1.5)
                * (2.0 + gg)
            );
        }

    // now we need to sample the 'primary' ray. this ray gathers the light that gets scattered onto it
    for (var i: u32 = 0; i < config.atmosphere_march_steps; i++) {
        // calculate where we are along this ray
        let pos_i = start + dir * ray_pos_i;

        // and how high we are above the surface
        let height_i = length(pos_i) - config.planet_radius;

        // now calculate the density of the particles (both for rayleigh and mie)
        var density = vec3(exp(-height_i / scale_height), 0.0);

        // and the absorption density. this is for ozone, which scales together with the rayleigh,
        // but absorbs the most at a specific height, so use the sech function for a nice curve falloff for this height
        // clamp it to avoid it going out of bounds. This prevents weird black spheres on the night side
        let denom = (config.atmosphere_height_absorption - height_i) / config.atmosphere_absorption_falloff;
        density.z = (1.0 / (denom * denom + 1.0)) * density.x;

        // multiply it by the step size here
        // we are going to use the density later on as well
        density *= step_size_i;

        // Add these densities to the optical depth, so that we know how many particles are on this ray.
        opt_i += density;

        // Calculate the step size of the light ray.
        // again with a ray sphere intersect
        // a, b, c and d are already defined
        a = dot(light_dir, light_dir);
        b = 2.0 * dot(light_dir, pos_i);
        c = dot(pos_i, pos_i) - (config.atmosphere_radius * config.atmosphere_radius);
        d = (b * b) - 4.0 * a * c;

        // no early stopping, this one should always be inside the atmosphere
        // calculate the ray length
        let step_size_l = (-b + sqrt(d)) / (2.0 * a * f32(config.atmosphere_light_march_steps));

        // and the position along this ray
        // this time we are sure the ray is in the atmosphere, so set it to 0
        var ray_pos_l = step_size_l * 0.5;

        // and the optical depth of this ray
        var opt_l = vec3(0.0);

        // now sample the light ray
        // this is similar to what we did before
        for (var l: u32 = 0; l < config.atmosphere_light_march_steps; l++) {

            // calculate where we are along this ray
            let pos_l = pos_i + light_dir * ray_pos_l;

            // the heigth of the position
            let height_l = length(pos_l) - config.planet_radius;

            // calculate the particle density, and add it
            // this is a bit verbose
            // first, set the density for ray and mie
            var density_l = vec3(exp(-height_l / scale_height), 0.0);

            // then, the absorption
            let denom = (config.atmosphere_height_absorption - height_l) / config.atmosphere_absorption_falloff;
            density_l.z = (1.0 / (denom * denom + 1.0)) * density_l.x;

            // multiply the density by the step size
            density_l *= step_size_l;

            // and add it to the total optical depth
            opt_l += density_l;

            // and increment where we are along the light ray.
            ray_pos_l += step_size_l;

        }

        // Now we need to calculate the attenuation
        // this is essentially how much light reaches the current sample point due to scattering
        let attn = exp(-config.atmosphere_rayleigh_beta * (opt_i.x + opt_l.x) - config.atmosphere_mie_beta * (opt_i.y + opt_l.y) - config.atmosphere_absorption_beta * (opt_i.z + opt_l.z));

        // accumulate the scattered light (how much will be scattered towards the camera)
        total_ray += density.x * attn;
        total_mie += density.y * attn;

        // and increment the position on this ray
        ray_pos_i += step_size_i;

    }

    // calculate how much light can pass through the atmosphere
    let opacity = exp(-(config.atmosphere_mie_beta * opt_i.y + config.atmosphere_rayleigh_beta * opt_i.x + config.atmosphere_absorption_beta * opt_i.z));

	// calculate and return the final color
    return (
        	phase_ray * config.atmosphere_rayleigh_beta * total_ray // rayleigh color
       		+ phase_mie * config.atmosphere_mie_beta * total_mie // mie
            + opt_i.x * config.atmosphere_ambient_beta // and ambient
    ) * light_intensity + scene_color * opacity; // now make sure the background is rendered correctly


}


/*
A ray-sphere intersect
This was previously used in the atmosphere as well, but it's only used for the planet intersect now, since the atmosphere has this
ray sphere intersect built in
*/
fn ray_sphere_intersect(start: vec3<f32>, dir: vec3<f32>, radius: f32) -> vec2<f32> {
	let a: f32 = dot(dir, dir);
	let b: f32 = 2. * dot(dir, start);
	let c: f32 = dot(start, start) - radius * radius;
	let d: f32 = b * b - 4. * a * c;
	if (d < 0.) {	return vec2<f32>(100000., -100000.);
 }
	return vec2<f32>((-b - sqrt(d)) / (2. * a), (-b + sqrt(d)) / (2. * a));
}


/*
The following function returns the scene color and depth
(the color of the pixel without the atmosphere, and the distance to the surface that is visible on that pixel)

in this case, the function renders a green sphere on the place where the planet should be
color is in .xyz, distance in .w

I won't explain too much about how this works, since that's not the aim of this shader
*/
fn render_scene(pos: vec3f, dir: vec3f) -> vec4f {
    // the color to use, w is the scene depth
    var color = vec4(0.0, 0.0, 0.0, 1e12);

    // add a sun, if the angle between the ray direction and the light direction is small enough, color the pixels white
    if dot(dir, config.sun_dir.xyz) > 0.9998 {
        color = vec4(vec3(3.0), color.w);
    }

    // get where the ray intersects the planet
    let planet_intersect = ray_sphere_intersect(pos - config.planet_position, dir, config.planet_radius);

    // if the ray hit the planet, set the max distance to that ray
    if (0.0 < planet_intersect.y) {
    	color = vec4f(1.0, 1.0, 1.0, max(planet_intersect.x, 0.0));
    }

	return color;
}

/*
next, we need a way to do something with the scattering function

to do something with it we need the camera vector (which is the ray direction) of the current pixel
this function calculates it
*/
fn get_camera_vector(resolution: vec2f, coord: vec2f) -> vec3f {
	var uv    = coord.xy - 0.5;
    return normalize(vec3(uv.x * resolution.x / resolution.y, -uv.y, -1.0));
}

fn henyey_greenstein(ray_dot_sun: f32, g: f32) -> f32 {
    let g_squared = g * g;
    return (1.0 - g_squared) / pow(1.0 + g_squared - 2.0 * g * ray_dot_sun, 1.5);
}

fn intersect_earth_sphere(ray_dir: vec3f, radius: f32) -> f32 {
    let bottom = config.planet_radius * ray_dir.y;
    let d = bottom * bottom + radius * radius + 2.0 * config.planet_radius * radius;
    return sqrt(d) - bottom;
}

fn cloud_map_base(p: vec3f, normalized_height: f32) -> f32 {
	let uv = abs(p * (0.00005 * config.clouds_base_scale) * config.render_resolution.xyy);
    let cloud = textureLoad(
        clouds_atlas_texture,
         vec2u(
            u32(uv.x) % u32(config.render_resolution.x),
            u32(uv.z) % u32(config.render_resolution.y)
        )
    ).rgb;

    var n = normalized_height * normalized_height * cloud.b + pow(1.0 - normalized_height, 16.0);
	return common::remap(cloud.r - n, cloud.g, 1.0);
}

fn cloud_map_detail(position: vec3f) -> f32 {
    let p = abs(position) * (0.0016 * config.clouds_base_scale * config.clouds_details_scale);

    // TODO: add bilinear filtering
    var p1 = p % 32.0;
    let a = textureLoad(clouds_worley_texture, vec3u(u32(p1.x), u32(p1.y), u32(p1.z))).r;

    // TODO: add bilinear filtering
    let p2 = (p + 1.0) % 32.0;
    let b = textureLoad(clouds_worley_texture, vec3u(u32(p2.x), u32(p2.y), u32(p2.z))).r;

    return mix(a, b, fract(p.y));
}

// Erode a bit from the bottom and top of the cloud layer
fn cloud_gradient(normalized_height: f32) -> f32 {
    return (
        common::linearstep(0.0, 0.1, normalized_height) -
        common::linearstep(0.8, 1.2, normalized_height)
    );
}

fn cloud_map(pos: vec3f, normalized_height: f32) -> f32 {
    let ps = pos;

    var m = cloud_map_base(ps, normalized_height) * cloud_gradient(normalized_height);

	let detail_strength = smoothstep(1.0, 0.5, m);

    // Erode with detail
    if detail_strength > 0.0 {
		m -= cloud_map_detail(ps) * detail_strength * config.clouds_detail_strength;
    }

	m = smoothstep(0.0, config.clouds_base_edge_softness, m + config.clouds_coverage - 1.0);
    m *= common::linearstep0(config.clouds_bottom_softness, normalized_height);

    return clamp(m * config.clouds_density * (1.0 + max((ps.x - 7000.0) * 0.005, 0.0)), 0.0, 1.0);
}

fn volumetric_shadow(origin: vec3f, ray_dot_sun: f32) -> f32{
    var ray_step_size = config.clouds_shadow_march_step_size;
    var distance_along_ray = ray_step_size * 0.5;
    var shadow = 1.0;
    let clouds_height = config.clouds_top - config.clouds_bottom;

    for (var s: u32 = 0; s < config.clouds_self_shadow_steps; s++) {
        let pos = origin + config.sun_dir.xyz * distance_along_ray;
        let normalized_height = (length(pos) - (config.planet_radius + config.clouds_bottom)) / clouds_height;

        if (normalized_height > 1.0) { return shadow; };

        let density = cloud_map(pos, normalized_height);
        shadow *= exp(-density * ray_step_size);

        ray_step_size *= config.clouds_shadow_march_step_multiply;
        distance_along_ray += ray_step_size;
    }

    return shadow;
}

fn raymarch(_ray_origin: vec3f, ray_dir: vec3f, _dist: f32) -> RaymarchResult {
    var dist = _dist;

    if (ray_dir.y < 0.0) {
        return RaymarchResult(dist, vec4f(0.0, 0.0, 0.0, 10.0));
    }

    let ro_xz = _ray_origin.xz;

    let ray_origin = vec3f(
        ro_xz.x,
        sqrt(config.planet_radius * config.planet_radius - dot(ro_xz, ro_xz)),
        ro_xz.y
    );

    let start = intersect_earth_sphere(ray_dir, config.clouds_bottom);
    var end = intersect_earth_sphere(ray_dir, config.clouds_top);

    if (start > dist) {
        return RaymarchResult(dist, vec4f(0.0, 0.0, 0.0, 10.0));
    }

    end = min(end, dist);

    let ray_dot_sun = dot(ray_dir, -config.sun_dir.xyz);

    let step_distance = (end - start) / f32(config.clouds_march_steps);
    let hashed_offset = common::hash13(ray_dir + fract(config.time));
    var dir_length = start - step_distance * hashed_offset;

    // Frostbite: dual-lobe phase function
    let scattering = mix(
        henyey_greenstein(ray_dot_sun, config.forward_scattering_g),
        henyey_greenstein(ray_dot_sun, config.backward_scattering_g),
        config.scattering_lerp
    );

    var transmittance = 1.0;
    var scattered_light = vec3f(0.0, 0.0, 0.0);

    dist = config.planet_radius;
    let clouds_height = config.clouds_top - config.clouds_bottom;

    for (var s: u32 = 0; s < config.clouds_march_steps; s++) {
        let p = ray_origin + dir_length * ray_dir;

        let normalized_height = clamp(
            (length(p) - (config.planet_radius + config.clouds_bottom)) / clouds_height,
            0.0,
            1.0
        );

        let density_sampled = cloud_map(p, normalized_height);

        if (density_sampled > 0.0) {
            dist = min(dist, dir_length);
            let ambient_light = mix(config.ambient_color_bottom, config.ambient_color_top, normalized_height).rgb;

            // Frostbite energy-conversing integration
            let S = (ambient_light + config.sun_color.rgb * (scattering * volumetric_shadow(p, ray_dot_sun))) * density_sampled;
            let delta_transmittance = exp(-density_sampled * step_distance);
            let integrated_scattering = (S - S * delta_transmittance) / density_sampled;

            scattered_light += transmittance * integrated_scattering;
            transmittance *= delta_transmittance;
        }

        if transmittance <= config.clouds_min_transmittance { break; }

        dir_length += step_distance;
    }

    return RaymarchResult(dist, vec4f(scattered_light, transmittance));
}

// Fast skycolor function by Íñigo Quílez
// https://www.shadertoy.com/view/MdX3Rr
fn get_sky_color(ray_dir: vec3f) -> vec3f {
    let sundot = clamp(dot(ray_dir,config.sun_dir.xyz),0.0,1.0);
	var col = vec3f(0.2,0.5,0.85)*1.1 - max(ray_dir.y,0.01)*max(ray_dir.y,0.01)*0.5;
    col = mix(col, 0.85*vec3(0.7,0.75,0.85), pow(1.0-max(ray_dir.y,0.0), 6.0) );

    col += 0.25*vec3f(1.0,0.7,0.4)*pow( sundot,5.0 );
    col += 0.25*vec3f(1.0,0.8,0.6)*pow( sundot,64.0 );
    col += 0.20*vec3f(1.0,0.8,0.6)*pow( sundot,512.0 );

    col += clamp((0.1-ray_dir.y)*10., 0., 1.) * vec3f(.0,.1,.2);
    col += 0.2*vec3f(1.0,0.8,0.6)*pow( sundot, 8.0 );
    return col;
}

fn render_clouds_atlas(frag_coord: vec2f) -> vec4f {
    let v_uv = frag_coord / config.render_resolution.xy;
    let coord = vec3f(v_uv, 0.5);

    let mfbm = 0.9;
    let mvor = 0.7;

    return vec4f(
        mix(
            1.0,
            common::tilable_perlin_fbm(coord, 7, 4),
            mfbm
        ) * mix(
            1.0,
            common::tilable_voronoi(coord, 8, 9.0),
            mvor
        ),
        0.625 * common::tilable_voronoi(coord, 3, 15.0) +
            0.250 * common::tilable_voronoi(coord, 3, 19.0) +
            0.125 * common::tilable_voronoi(coord, 3, 23.0) -
            1.0,
        1.0 - common::tilable_voronoi(coord + 0.5, 6, 9.0),
        1.0
    );
}

fn render_clouds_worley(coord: vec3f) -> vec4f {
    let r = common::tilable_voronoi(coord, 16, 3.0);
    let g = common::tilable_voronoi(coord, 4, 8.0);
    let b = common::tilable_voronoi(coord, 4, 16.0);

    let c = max(0.0, 1.0 - (r + g * 0.5 + b * 0.25) / 1.75);

    return vec4f(c);
}

fn main_image(frag_coord: vec2f, camera: mat4x4f, old_cam: mat4x4f, ray_dir: vec3f, ray_origin: vec3f) -> vec4f {
    if (frag_coord.y < 1.0) {
        if frag_coord.x < 1.0 { return vec4f(config.render_resolution.xy, 0.0, 0.0); }
        return common::save_camera(camera, frag_coord, ray_origin);
    }

    var dist = 1e9;
    var col = vec4f(0.0, 0.0, 0.0, 1.0);

    if ray_dir.y > 0.0 {
        let result = raymarch(ray_origin, ray_dir, dist);
        col = result.color;
        dist = result.dist;

        let fog = 1.0 - (0.1 + exp(-dist * 0.0001));

        let scene = render_scene(ray_origin, ray_dir);

        var sky_col = calculate_scattering(
            vec3(0.0, config.planet_radius, 0.0), // the position of the camera
            ray_dir, 					          // the camera vector (ray direction of this pixel)
            scene.w, 						      // max dist, essentially the scene depth
            scene.xyz,						      // scene color, the color of the current pixel being rendered
            config.sun_dir.xyz,					  // light direction
            vec3(40.0),						      // light intensity, 40 looks nice
        );
        col = vec4f(mix(col.rgb, sky_col * (1.0 - col.a), fog), col.a);
        // col = vec4f(mix(col.rgb, get_sky_color(ray_dir) * (1.0 - col.a), fog), col.a);
    }

    if col.w > 1.0 {
        return vec4f(0.0, 0.0, 0.0, 1.0);
    }

    let old_cam_col = textureLoad(clouds_render_texture, vec2u(1, u32(config.render_resolution.y) - 1));
    let new_cam_col = camera[0];

    if abs(old_cam_col[0] - new_cam_col[0]) > 0.0001 {
        return col;
    }

    let original_color = textureLoad(
        clouds_render_texture,
        vec2u(u32(frag_coord.x),
        u32(config.render_resolution.y - 1.0) - u32(frag_coord.y))
    );
    return mix(col, original_color, config.reprojection_strength);
}

fn get_ray_origin(time: f32) -> vec3f {
    return config.camera_translation - config.wind_displacement;
}

fn get_ray_direction(frag_coord: vec2f) -> vec3f {
    // inverse_camera_projection is also called view_from_clip
    // inverse_camera_view is also called world_from_view
    let rect_relative = frag_coord / config.render_resolution;

    // Flip the Y co-ordinate from the top to the bottom to enter NDC.
    let ndc_xy = (rect_relative * 2.0 - vec2f(1.0, 1.0)) * vec2f(1.0, -1.0);

    let ray_clip = vec4f(ndc_xy.xy, -1.0, 1.0);
    let ray_eye = config.inverse_camera_projection * ray_clip;
    let ray_world = config.inverse_camera_view * vec4f(ray_eye.xy, -1.0, 0.0);

    return normalize(ray_world.xyz);
}

@compute @workgroup_size(8, 8, 1)
fn init(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let index = vec2f(f32(invocation_id.x), f32(invocation_id.y));
    let inverted_y_coord = config.render_resolution.y - index.y;

    let worley_coord = vec2f(0.5) + vec2f(index.x, inverted_y_coord);

    let z = floor(worley_coord.x / WORLEY_RESOLUTION_F32) + 8.0 * floor(worley_coord.y / WORLEY_RESOLUTION_F32);
    let xy = vec2f(index.x, inverted_y_coord) % WORLEY_RESOLUTION_F32;
    let xyz = vec3f(xy, z);

    let worley_col = render_clouds_worley(xyz / WORLEY_RESOLUTION_F32);
    let atlas_col = render_clouds_atlas(vec2f(index.x, inverted_y_coord));

    storageBarrier();

    textureStore(clouds_atlas_texture, invocation_id.xy, atlas_col);
    textureStore(clouds_worley_texture, vec3u(u32(xyz.x), u32(xyz.y), u32(xyz.z)), worley_col);
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let index = vec2f(f32(invocation_id.x), f32(invocation_id.y));

    // Load old camera matrix before storageBarrier to prevent race conditions;
    let old_cam = common::load_camera(clouds_render_texture);
    var frag_coord = vec2f(index.x + 0.5, config.render_resolution.y - 0.5 - index.y);

    var ray_origin = get_ray_origin(config.time);
    var ray_dir = get_ray_direction(vec2f(index.x + 0.5, 0.5 + index.y));
    var col = main_image(frag_coord, config.inverse_camera_view, old_cam, ray_dir, ray_origin);

    // Atmosphere
    // let camera_vector = get_camera_vector(iResolution, fragCoord);
    // ray_origin = vec3(0.0, config.atmosphere_radius + (-cos(config.time / 2.0) * (config.atmosphere_radius - config.planet_radius - 1.0)), 0.0);
    ray_origin = vec3(0.0, config.planet_radius, 0.0);

    // get the scene color and depth, color is in xyz, depth in w
    // replace this with something better if you are using this shader for something else
    let scene = render_scene(ray_origin, ray_dir);

    var sky_col = calculate_scattering(
    	ray_origin,				              // the position of the camera
        ray_dir, 					          // the camera vector (ray direction of this pixel)
        scene.w, 						      // max dist, essentially the scene depth
        scene.xyz,						      // scene color, the color of the current pixel being rendered
        config.sun_dir.xyz,					  // light direction
        vec3(40.0),						      // light intensity, 40 looks nice
    );

    storageBarrier();

    textureStore(clouds_render_texture, invocation_id.xy, col);
    // textureStore(sky_texture, invocation_id.xy, vec4f(get_sky_color(ray_dir), 1.0));
    textureStore(sky_texture, invocation_id.xy, vec4(1.0 - exp(-sky_col), 1.0));
}
