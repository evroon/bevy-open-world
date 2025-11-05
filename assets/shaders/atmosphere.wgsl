#import bevy_sprite::{
    mesh2d_view_bindings::view,
    mesh2d_view_bindings::globals,
    mesh2d_functions::{get_world_from_local, mesh2d_position_local_to_clip},
}
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

// Based on https://www.shadertoy.com/view/wlBXWK


// first, lets define some constants to use (planet radius, position, and scattering coefficients)
const PLANET_POS = vec3f(0.0); /* the position of the planet */
const PLANET_RADIUS = 6371e3; /* radius of the planet */
const ATMOS_RADIUS = 6471e3; /* radius of the atmosphere */
// scattering coeffs
const RAY_BETA = vec3f(5.5e-6, 13.0e-6, 22.4e-6); /* rayleigh, affects the color of the sky */
const MIE_BETA = vec3(21e-6); /* mie, affects the color of the blob around the sun */
const AMBIENT_BETA = vec3(0.0); /* ambient, affects the scattering color when there is no lighting from the sun */
const ABSORPTION_BETA = vec3(2.04e-5, 4.97e-5, 1.95e-6); /* what color gets absorbed by the atmosphere (Due to things like ozone) */
const G = 0.7; /* mie scattering direction, or how big the blob around the sun is */
// and the heights (how far to go up before the scattering has no effect)
const HEIGHT_RAY = 8e3; /* rayleigh height */
const HEIGHT_MIE = 1.2e3; /* and mie */
const HEIGHT_ABSORPTION = 30e3; /* at what height the absorption is at it's maximum */
const ABSORPTION_FALLOFF = 4e3; /* how much the absorption decreases the further away it gets from the maximum height */
// and the steps (more looks better, but is slower)
// the primary step has the most effect on looks
// #if HW_PERFORMANCE==0
// edit these if you are on mobile
// const PRIMARY_STEPS = 12;
// const LIGHT_STEPS = 4;
// # else
// and these on desktop
const PRIMARY_STEPS = 32; /* primary steps, affects quality the most */
const LIGHT_STEPS = 8; /* light steps, how much steps in the light direction are taken */
// #endif


fn calculate_scattering(
    _start: vec3<f32>,
    dir: vec3<f32>,
    max_dist: f32,
    scene_color: vec3<f32>,
    light_dir: vec3<f32>,
    light_intensity: vec3<f32>,
    planet_position: vec3<f32>,
    planet_radius: f32,
    atmo_radius: f32,
    beta_ray: vec3<f32>,
    beta_mie: vec3<f32>,
    beta_absorption: vec3<f32>,
    beta_ambient: vec3<f32>,
    g: f32,
    height_ray: f32,
    height_mie: f32,
    height_absorption: f32,
    absorption_falloff: f32,
    steps_i: i32,
    steps_l: i32,
) -> vec3<f32> {
    let start = _start - planet_position;
    var a = dot(dir, dir);
    var b = 2.0 * dot(dir, start);
    var c = dot(start, start) - (atmo_radius * atmo_radius);
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
    let step_size_i = (ray_length.y - ray_length.x) / f32(steps_i);

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
    let scale_height = vec2(height_ray, height_mie);

    // Calculate the Rayleigh and Mie phases.
    // This is the color that will be scattered for this ray
    // mu, mumu and gg are used quite a lot in the calculation, so to speed it up, precalculate them
    let mu = dot(dir, light_dir);
    let mumu = mu * mu;
    let gg = g * g;
    let phase_ray = 3.0 / (50.2654824574 /* (16 * pi) */) * (1.0 + mumu);
    var phase_mie = 0.0;
    if allow_mie { phase_mie = 3.0 / (25.1327412287 /* (8 * pi) */) * ((1.0 - gg) * (mumu + 1.0)) / (pow(1.0 + gg - 2.0 * mu * g, 1.5) * (2.0 + gg)); }

    // now we need to sample the 'primary' ray. this ray gathers the light that gets scattered onto it
    for (var i = 0; i < steps_i; i++) {
        // calculate where we are along this ray
        let pos_i = start + dir * ray_pos_i;

        // and how high we are above the surface
        let height_i = length(pos_i) - planet_radius;

        // now calculate the density of the particles (both for rayleigh and mie)
        var density = vec3(exp(-height_i / scale_height), 0.0);

        // and the absorption density. this is for ozone, which scales together with the rayleigh,
        // but absorbs the most at a specific height, so use the sech function for a nice curve falloff for this height
        // clamp it to avoid it going out of bounds. This prevents weird black spheres on the night side
        let denom = (height_absorption - height_i) / absorption_falloff;
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
        c = dot(pos_i, pos_i) - (atmo_radius * atmo_radius);
        d = (b * b) - 4.0 * a * c;

        // no early stopping, this one should always be inside the atmosphere
        // calculate the ray length
        let step_size_l = (-b + sqrt(d)) / (2.0 * a * f32(steps_l));

        // and the position along this ray
        // this time we are sure the ray is in the atmosphere, so set it to 0
        var ray_pos_l = step_size_l * 0.5;

        // and the optical depth of this ray
        var opt_l = vec3(0.0);

        // now sample the light ray
        // this is similar to what we did before
        for (var l = 0; l < steps_l; l++) {

            // calculate where we are along this ray
            let pos_l = pos_i + light_dir * ray_pos_l;

            // the heigth of the position
            let height_l = length(pos_l) - planet_radius;

            // calculate the particle density, and add it
            // this is a bit verbose
            // first, set the density for ray and mie
            var density_l = vec3(exp(-height_l / scale_height), 0.0);

            // then, the absorption
            let denom = (height_absorption - height_l) / absorption_falloff;
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
        let attn = exp(-beta_ray * (opt_i.x + opt_l.x) - beta_mie * (opt_i.y + opt_l.y) - beta_absorption * (opt_i.z + opt_l.z));

        // accumulate the scattered light (how much will be scattered towards the camera)
        total_ray += density.x * attn;
        total_mie += density.y * attn;

        // and increment the position on this ray
        ray_pos_i += step_size_i;

    }

    // calculate how much light can pass through the atmosphere
    let opacity = exp(-(beta_mie * opt_i.y + beta_ray * opt_i.x + beta_absorption * opt_i.z));

	// calculate and return the final color
    return (
        	phase_ray * beta_ray * total_ray // rayleigh color
       		+ phase_mie * beta_mie * total_mie // mie
            + opt_i.x * beta_ambient // and ambient
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
To make the planet we're rendering look nicer, we implemented a skylight function here

Essentially it just takes a sample of the atmosphere in the direction of the surface normal
*/
fn skylight(sample_pos: vec3f, _surface_normal: vec3f, light_dir: vec3f, background_col: vec3f) -> vec3f {

    // slightly bend the surface normal towards the light direction
    let surface_normal = normalize(mix(_surface_normal, light_dir, 0.6));

    // and sample the atmosphere
    return calculate_scattering(
    	sample_pos,						// the position of the camera
        surface_normal, 				// the camera vector (ray direction of this pixel)
        3.0 * ATMOS_RADIUS, 			// max dist, since nothing will stop the ray here, just use some arbitrary value
        background_col,					// scene color, just the background color here
        light_dir,						// light direction
        vec3(40.0),						// light intensity, 40 looks nice
        PLANET_POS,						// position of the planet
        PLANET_RADIUS,                  // radius of the planet in meters
        ATMOS_RADIUS,                   // radius of the atmosphere in meters
        RAY_BETA,						// Rayleigh scattering coefficient
        MIE_BETA,                       // Mie scattering coefficient
        ABSORPTION_BETA,                // Absorbtion coefficient
        AMBIENT_BETA,					// ambient scattering, turned off for now. This causes the air to glow a bit when no light reaches it
        G,                          	// Mie preferred scattering direction
        HEIGHT_RAY,                     // Rayleigh scale height
        HEIGHT_MIE,                     // Mie scale height
        HEIGHT_ABSORPTION,				// the height at which the most absorption happens
        ABSORPTION_FALLOFF,				// how fast the absorption falls off from the absorption height
        LIGHT_STEPS, 					// steps in the ray direction
        LIGHT_STEPS 					// steps in the light direction
    );
}


/*
The following function returns the scene color and depth
(the color of the pixel without the atmosphere, and the distance to the surface that is visible on that pixel)

in this case, the function renders a green sphere on the place where the planet should be
color is in .xyz, distance in .w

I won't explain too much about how this works, since that's not the aim of this shader
*/
fn render_scene(pos: vec3f, dir: vec3f, light_dir: vec3f) -> vec4f {
    // the color to use, w is the scene depth
    var color = vec4(0.0, 0.0, 0.0, 1e12);

    // add a sun, if the angle between the ray direction and the light direction is small enough, color the pixels white
    if dot(dir, light_dir) > 0.9998 {
        color = vec4(vec3(3.0), color.w);
    }

    // get where the ray intersects the planet
    let planet_intersect = ray_sphere_intersect(pos - PLANET_POS, dir, PLANET_RADIUS);

    // if the ray hit the planet, set the max distance to that ray
    if (0.0 < planet_intersect.y) {
    	color.w = max(planet_intersect.x, 0.0);

        // sample position, where the pixel is
        let sample_pos = pos + (dir * planet_intersect.x) - PLANET_POS;

        // and the surface normal
        let surface_normal = normalize(sample_pos);

        // get the color of the sphere
        color = vec4(0.0, 0.25, 0.05, color.w);

        // get wether this point is shadowed, + how much light scatters towards the camera according to the lommel-seelinger law
        let N = surface_normal;
        let V = -dir;
        let L = light_dir;
        let dotNV = max(1e-6, dot(N, V));
        let dotNL = max(1e-6, dot(N, L));
        let shadow = dotNL / (dotNL + dotNV);

        // apply the shadow
        color = vec4(color.xyz * shadow, color.w);

        // apply skylight
        color = vec4(color.xyz + clamp(skylight(sample_pos, surface_normal, light_dir, vec3(0.0)) * vec3(0.0, 0.25, 0.05), vec3(0.0), vec3(1.0)), color.w);
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


@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let fragCoord = mesh.uv;
    let iResolution = view.viewport.zw;
    let iTime = globals.time;
    let iMouse = vec2(0.0);
    var uv = fragCoord;

    // get the camera vector
    let camera_vector = get_camera_vector(iResolution, fragCoord);

    // get the camera position, switch based on the defines
    let camera_position = vec3(0.0, ATMOS_RADIUS + (-cos(iTime / 2.0) * (ATMOS_RADIUS - PLANET_RADIUS - 1.0)), 0.0);

    // get the light direction
    // also base this on the mouse position, that way the time of day can be changed with the mouse
    var light_dir = vec3(0.0);
    if iMouse.y == 0.0 {
        light_dir = normalize(vec3(0.0, cos(-iTime/8.0), sin(-iTime/8.0)));
    } else {
    	light_dir = normalize(vec3(0.0, cos(iMouse.y * -5.0 / iResolution.y), sin(iMouse.y * -5.0 / iResolution.y)));
    }

    // get the scene color and depth, color is in xyz, depth in w
    // replace this with something better if you are using this shader for something else
    let scene = render_scene(camera_position, camera_vector, light_dir);

    // the color of this pixel
    var col = vec3(0.0);//scene.xyz;

    // get the atmosphere color
    col += calculate_scattering(
    	camera_position,				// the position of the camera
        camera_vector, 					// the camera vector (ray direction of this pixel)
        scene.w, 						// max dist, essentially the scene depth
        scene.xyz,						// scene color, the color of the current pixel being rendered
        light_dir,						// light direction
        vec3(40.0),						// light intensity, 40 looks nice
        PLANET_POS,						// position of the planet
        PLANET_RADIUS,                  // radius of the planet in meters
        ATMOS_RADIUS,                   // radius of the atmosphere in meters
        RAY_BETA,						// Rayleigh scattering coefficient
        MIE_BETA,                       // Mie scattering coefficient
        ABSORPTION_BETA,                // Absorbtion coefficient
        AMBIENT_BETA,					// ambient scattering, turned off for now. This causes the air to glow a bit when no light reaches it
        G,                          	// Mie preferred scattering direction
        HEIGHT_RAY,                     // Rayleigh scale height
        HEIGHT_MIE,                     // Mie scale height
        HEIGHT_ABSORPTION,				// the height at which the most absorption happens
        ABSORPTION_FALLOFF,				// how fast the absorption falls off from the absorption height
        PRIMARY_STEPS, 					// steps in the ray direction
        LIGHT_STEPS 					// steps in the light direction
    );

    // apply exposure, removing this makes the brighter colors look ugly
    // you can play around with removing this
    col = 1.0 - exp(-col);

    return vec4(col, 1.0);
}
