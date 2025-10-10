#define_import_path bevy_open_world::common

const INV_SCENE_SCALE = 0.1;

fn linearstep(s: f32, e: f32, v: f32) -> f32 {
    return clamp((v - s) * (1.0 / (e - s)), 0.0, 1.0);
}

fn linearstep0(e: f32, v: f32) -> f32 {
    return min(v * (1.0 / e), 1.);
}

fn remap(v: f32, s: f32, e: f32) -> f32 {
    return (v - s) / (e - s);
}

fn get_ray(camera: mat3x3f, frag_coord: vec2f, resolution: vec2f, camera_fl: f32) -> vec3f {
    let p = -(2.0 * frag_coord - resolution) / resolution.y;
    return camera * normalize(vec3f(p, camera_fl));
}

// To reduce noise I use temporal reprojection.
// The temporal repojection code is based on code from the shader
// Rain Forest (by Íñigo Quílez):
//
// https://www.shadertoy.com/view/4ttSWf
fn save_camera(camera: mat3x3f, frag_coord: vec2f, ro: vec3f) -> vec4f {
    if abs(frag_coord.x - 4.5) < 0.5 { return vec4f(camera[2], -dot(camera[2], ro)); }
    if abs(frag_coord.x - 3.5) < 0.5 { return vec4f(camera[1], -dot(camera[1], ro)); }
    if abs(frag_coord.x - 2.5) < 0.5 { return vec4f(camera[0], -dot(camera[0], ro)); }

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

fn reproject_pos(camera: mat3x3f, pos: vec3f, resolution: vec2f, old_cam: mat4x4f, camera_fl: f32, camera_translation: vec3f) -> vec2f {
    let oldCam = mat4x4f(
        vec4f(camera[0], -dot(camera[0], camera_translation)),
        vec4f(camera[1], -dot(camera[1], camera_translation)),
        vec4f(camera[2], -dot(camera[2], camera_translation)),
        vec4f(0.0, 0.0, 0.0, 1.0),
    );
    let wpos = vec4f(pos, 1.0);
    let cpos = wpos * oldCam;
    let npos = camera_fl * -cpos.xy / cpos.z;
    return 0.5 + 0.5 * npos * vec2f(resolution.y / resolution.x, 1.0);
}

// Noise functions
//
// Hash without Sine by DaveHoskins
//
// https://www.shadertoy.com/view/4djSRW
//
fn hash12(_p: vec2f) -> f32 {
    let p = 50.0 * fract(_p * 0.3183099);
    return fract(p.x * p.y * (p.x + p.y));
}

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

fn modi(x: f32, y: f32) -> i32 {
    return i32(x - y * floor(x / y));
}

// Noise functions used for cloud shapes
//
// Based on Frostbite
// https://github.com/sebh/TileableVolumeNoise/blob/master/TileableVolumeNoise.cpp
fn value_noise(x: vec3f, tile: f32) -> f32 {
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

fn voronoi(x: vec3f, tile: f32) -> f32 {
    let p = floor(x);
    let f = fract(x);

    var res = 100.0;
    for (var k = -1.0; k < 1.1; k += 1.0) {
        for (var j = -1.0; j < 1.1; j += 1.0) {
            for (var i = -1.0; i < 1.1; i += 1.0) {
                let b = vec3f(i, j, k);
                var c = p + b;

                if tile > 0.0 {
                    c = c % vec3f(tile);
                }

                let r = vec3f(b) - f + hash13(c);
                let d = dot(r, r);

                if d < res {
                    res = d;
                }
            }
        }
    }

    return 1.0 - res;
}

fn tilable_voronoi(p: vec3f, octaves: i32, tile: f32) -> f32 {
    var f = 1.;
    var a = 1.;
    var c = 0.;
    var w = 0.;

    if tile > 0. { f = tile; }

    for (var i = 0; i < octaves; i++) {
        c += a * voronoi(p * f, f);
        f *= 2.0;
        w += a;
        a *= 0.5;
    }

    return c / w;
}

fn tilable_fbm(p: vec3f, octaves: i32, tile: f32) -> f32 {
    var f = 1.;
    var a = 1.;
    var c = 0.;
    var w = 0.;

    if tile > 0. { f = tile; }

    for (var i = 0; i < octaves; i++) {
        c += a * value_noise(p * f, f);
        f *= 2.0;
        w += a;
        a *= 0.5;
    }

    return c / w;
}
