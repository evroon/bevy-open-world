#define_import_path bevy_open_world::common

const UI0 = u32(1597334673);
const UI1 = u32(3812015801);
const UI3 = vec3u(UI0, UI1, u32(2798796415));
const UIF = (1.0 / f32(0xffffffff));

fn linearstep(s: f32, e: f32, v: f32) -> f32 {
    return clamp((v - s) * (1.0 / (e - s)), 0.0, 1.0);
}

fn linearstep0(e: f32, v: f32) -> f32 {
    return min(v * (1.0 / e), 1.0);
}

fn remap(v: f32, s: f32, e: f32) -> f32 {
    return (v - s) / (e - s);
}

// Temporal reprojection is used to reduce noise.
// The temporal repojection code is based on code from the shader
// Rain Forest (by Íñigo Quílez):
//
// https://www.shadertoy.com/view/4ttSWf
fn save_camera(camera: mat4x4f, frag_coord: vec2f, ray_origin: vec3f) -> vec4f {
    let camera_col_0 = vec3f(camera[0][0], camera[0][1], camera[0][2]);
    let camera_col_1 = vec3f(camera[1][0], camera[1][1], camera[1][2]);
    let camera_col_2 = vec3f(camera[2][0], camera[2][1], camera[2][2]);

    if abs(frag_coord.x - 4.5) < 0.5 { return vec4f(camera_col_2, -dot(camera_col_2, ray_origin)); }
    if abs(frag_coord.x - 3.5) < 0.5 { return vec4f(camera_col_1, -dot(camera_col_1, ray_origin)); }
    if abs(frag_coord.x - 2.5) < 0.5 { return vec4f(camera_col_0, -dot(camera_col_0, ray_origin)); }

    return vec4f(0.0);
}

fn load_camera(texture: texture_storage_2d<rgba32float, read_write>) -> mat4x4f {
    return mat4x4f(
        textureLoad(texture, vec2u(2, 0)),
        textureLoad(texture, vec2u(3, 0)),
        textureLoad(texture, vec2u(4, 0)),
        vec4f(0.0, 0.0, 0.0, 1.0)
    );
}

fn reproject_pos(camera: mat4x4f, pos: vec3f, resolution: vec2f, old_cam: mat4x4f, camera_translation: vec3f) -> vec2f {
    let camera_col_0 = vec3f(camera[0][0], camera[0][1], camera[0][2]);
    let camera_col_1 = vec3f(camera[1][0], camera[1][1], camera[1][2]);
    let camera_col_2 = vec3f(camera[2][0], camera[2][1], camera[2][2]);

    let old_cam_reconstructed = mat4x4f(
        vec4f(camera_col_0, -dot(camera_col_0, camera_translation)),
        vec4f(camera_col_1, -dot(camera_col_1, camera_translation)),
        vec4f(camera_col_2, -dot(camera_col_2, camera_translation)),
        vec4f(0.0, 0.0, 0.0, 1.0),
    );
    let wpos = vec4f(pos, 1.0);
    let cpos = wpos * old_cam_reconstructed;
    let npos = -cpos.xy / cpos.z;
    return 0.5 + 0.5 * npos * vec2f(resolution.y / resolution.x, 1.0);
}

// Noise functions
//
// Hash without Sine by DaveHoskins
//
// https://www.shadertoy.com/view/4djSRW
fn hash13(_p3: vec3f) -> f32 {
    var p3 = fract(_p3 * 1031.1031);
    p3 += dot(p3, p3.yzx + 19.19);
    return fract((p3.x + p3.y) * p3.z);
}

fn value_hash(_p3: vec3f) -> f32 {
    var p3 = fract(_p3 * 0.1031);
    p3 += dot(p3, p3.yzx + 19.19);
    return fract((p3.x + p3.y) * p3.z);
}

// Noise functions used for cloud shapes
//
// Based on Frostbite
// https://github.com/sebh/TileableVolumeNoise/blob/master/TileableVolumeNoise.cpp
fn hash_based_noise(x: vec3f, tile: f32) -> f32 {
    let p = floor(x);
    var f = fract(x);
    f = f * f * (3.0 - 2.0 * f);

    return mix(
        mix(
            mix(
                value_hash(p % tile),
                value_hash((p + vec3f(1.0, 0.0, 0)) % tile),
                f.x
            ),
            mix(
                value_hash((p + vec3f(0.0, 1.0, 0.0)) % tile),
                value_hash((p + vec3f(1.0, 1.0, 0.0)) % tile),
                f.x
            ),
            f.y
        ),
        mix(
            mix(
                value_hash((p + vec3f(0.0, 0.0, 1.0)) % tile),
                value_hash((p + vec3f(1.0, 0.0, 1.0)) % tile),
                f.x
            ),
            mix(
                value_hash((p + vec3f(0.0, 1.0, 1.0)) % tile),
                value_hash((p + vec3f(1.0, 1.0, 1.0)) % tile),
                f.x
            ),
            f.y
        ),
        f.z
    );
}

// Source: https://www.shadertoy.com/view/3dVXDc
fn hash33(p: vec3f) -> vec3f
{
	var q = vec3u(vec3i(p)) * UI3;
	q = (q.x ^ q.y ^ q.z) * UI3;
	return -1.0 + 2.0 * vec3f(q) * UIF;
}

// Gradient noise by iq (modified to be tileable)
// Alternative to hash_based_noise
// Source: https://www.shadertoy.com/view/3dVXDc
fn gradient_noise(x: vec3f, freq: f32) -> f32{
    // grid
    let p = floor(x);
    let w = fract(x);

    // quintic interpolant
    let u = w * w * w * (w * (w * 6.0 - 15.0) + 10.0);

    // gradients
    let ga = hash33((p + vec3f(0.0, 0.0, 0.0)) % freq);
    let gb = hash33((p + vec3f(1.0, 0.0, 0.0)) % freq);
    let gc = hash33((p + vec3f(0.0, 1.0, 0.0)) % freq);
    let gd = hash33((p + vec3f(1.0, 1.0, 0.0)) % freq);
    let ge = hash33((p + vec3f(0.0, 0.0, 1.0)) % freq);
    let gf = hash33((p + vec3f(1.0, 0.0, 1.0)) % freq);
    let gg = hash33((p + vec3f(0.0, 1.0, 1.0)) % freq);
    let gh = hash33((p + vec3f(1.0, 1.0, 1.0)) % freq);

    // projections
    let va = dot(ga, w - vec3f(0.0, 0.0, 0.0));
    let vb = dot(gb, w - vec3f(1.0, 0.0, 0.0));
    let vc = dot(gc, w - vec3f(0.0, 1.0, 0.0));
    let vd = dot(gd, w - vec3f(1.0, 1.0, 0.0));
    let ve = dot(ge, w - vec3f(0.0, 0.0, 1.0));
    let vf = dot(gf, w - vec3f(1.0, 0.0, 1.0));
    let vg = dot(gg, w - vec3f(0.0, 1.0, 1.0));
    let vh = dot(gh, w - vec3f(1.0, 1.0, 1.0));

    // interpolation
    return va +
           u.x * (vb - va) +
           u.y * (vc - va) +
           u.z * (ve - va) +
           u.x * u.y * (va - vb - vc + vd) +
           u.y * u.z * (va - vc - ve + vg) +
           u.z * u.x * (va - vb - ve + vf) +
           u.x * u.y * u.z * (-va + vb + vc - vd + ve - vf - vg + vh);
}

fn voronoi(x: vec3f, tile: f32) -> f32 {
    let p = floor(x);
    let f = fract(x);

    var res = 100.0;

    for (var k = -1.0; k < 1.1; k += 1.0) {
        for (var j = -1.0; j < 1.1; j += 1.0) {
            for (var i = -1.0; i < 1.1; i += 1.0) {
                let b = vec3f(i, j, k);
                var c = (p + b) % vec3f(tile);

                let r = vec3f(b) - f + hash13(c);
                let d = dot(r, r);

                res = min(res, d);
            }
        }
    }

    return 1.0 - res;
}

fn tilable_voronoi(p: vec3f, octaves: i32, _freq: f32) -> f32 {
    var freq = _freq;
    var amplitude = 1.0;
    var noise = 0.0;
    var w = 0.0;

    for (var i = 0; i < octaves; i++) {
        noise += amplitude * voronoi(p * freq, freq);
        freq *= 2.0;
        w += amplitude;
        amplitude *= 0.5;
    }

    return noise / w;
}

fn tilable_perlin_fbm(p: vec3f, octaves: i32, _freq: f32) -> f32 {
    var freq = _freq;
    var amplitude = 1.0;
    var noise = 0.0;
    var w = 0.0;

    for (var i = 0; i < octaves; i++) {
        // Alternative noise method can be used here
        // noise += amplitude * gradient_noise(p * freq, freq);
        noise += amplitude * hash_based_noise(p * freq, freq);
        freq *= 2.0;
        w += amplitude;
        amplitude *= 0.5;
    }

    // return noise;
    return noise / w;
}
