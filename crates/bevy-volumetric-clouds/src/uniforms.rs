use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        render_resource::{AsBindGroup, ShaderType, UniformBuffer},
    },
};

#[derive(Clone, Resource, ExtractResource, Reflect, ShaderType)]
#[reflect(Resource, Default)]
pub struct CloudsUniform {
    pub planet_radius: f32,
    pub planet_position: Vec3,
    pub atmosphere_radius: f32,
    pub atmosphere_rayleigh_beta: Vec3,
    pub atmosphere_mie_beta: Vec3,
    pub atmosphere_ambient_beta: Vec3,
    pub atmosphere_absorption_beta: Vec3,
    pub atmosphere_height_rayleigh: f32,
    pub atmosphere_height_mie: f32,
    pub atmosphere_height_absorption: f32,
    pub atmosphere_absorption_falloff: f32,
    pub atmosphere_march_steps: u32,
    pub atmosphere_light_march_steps: u32,
    pub clouds_march_steps: u32,
    pub clouds_self_shadow_steps: u32,
    pub clouds_bottom: f32,
    pub clouds_top: f32,
    pub clouds_coverage: f32,
    pub clouds_detail_strength: f32,
    pub clouds_base_edge_softness: f32,
    pub clouds_bottom_softness: f32,
    pub clouds_density: f32,
    pub clouds_shadow_march_step_size: f32,
    pub clouds_shadow_march_step_multiply: f32,
    pub clouds_base_scale: f32,
    pub clouds_details_scale: f32,
    pub clouds_min_transmittance: f32,
    pub forward_scattering_g: f32,
    pub backward_scattering_g: f32,
    pub scattering_lerp: f32,
    pub ambient_color_top: Vec4,
    pub ambient_color_bottom: Vec4,
    pub sun_dir: Vec4,
    pub sun_color: Vec4,
    pub camera_translation: Vec3,
    pub debug: f32,
    pub time: f32,
    pub reprojection_strength: f32,
    pub render_resolution: Vec2,
    pub inverse_camera_view: Mat4,
    pub inverse_camera_projection: Mat4,
    pub wind_displacement: Vec3,
}

impl Default for CloudsUniform {
    fn default() -> Self {
        Self {
            atmosphere_radius: 6471e3,
            atmosphere_rayleigh_beta: Vec3::new(5.5e-6, 13.0e-6, 22.4e-6),
            atmosphere_mie_beta: Vec3::splat(21e-6),
            atmosphere_ambient_beta: Vec3::ZERO,
            atmosphere_absorption_beta: Vec3::new(2.04e-5, 4.97e-5, 1.95e-6),
            atmosphere_absorption_falloff: 4e3,
            atmosphere_height_rayleigh: 8e3,
            atmosphere_height_mie: 1.2e3,
            atmosphere_height_absorption: 3e4,
            atmosphere_march_steps: 32,
            atmosphere_light_march_steps: 8,
            planet_radius: 0.0,
            planet_position: Vec3::ZERO,
            clouds_march_steps: 0,
            clouds_self_shadow_steps: 0,
            clouds_bottom: 0.,
            clouds_top: 0.,
            clouds_coverage: 0.0,
            clouds_detail_strength: 0.0,
            clouds_base_edge_softness: 0.0,
            clouds_bottom_softness: 0.0,
            clouds_density: 0.0,
            clouds_shadow_march_step_size: 0.0,
            clouds_shadow_march_step_multiply: 0.0,
            forward_scattering_g: 0.0,
            backward_scattering_g: 0.0,
            scattering_lerp: 0.0,
            ambient_color_top: Vec4::ZERO,
            ambient_color_bottom: Vec4::ZERO,
            clouds_min_transmittance: 0.0,
            clouds_base_scale: 0.0,
            clouds_details_scale: 0.0,
            sun_dir: Vec4::ZERO,
            sun_color: Vec4::ZERO,
            camera_translation: Vec3::ZERO,
            debug: 0.0,
            time: 0.0,
            reprojection_strength: 0.95,
            render_resolution: Vec2::new(1920.0, 1080.0),
            inverse_camera_view: Mat4::IDENTITY,
            inverse_camera_projection: Mat4::IDENTITY,
            wind_displacement: Vec3::new(-11.0, 0.0, 23.0),
        }
    }
}

#[derive(Resource, Default)]
pub struct CloudsUniformBuffer {
    pub buffer: UniformBuffer<CloudsUniform>,
}

#[derive(Resource, Clone, ExtractResource, AsBindGroup)]
pub struct CloudsImage {
    #[storage_texture(0, image_format = Rgba32Float, access = ReadWrite)]
    pub cloud_render_image: Handle<Image>,

    #[storage_texture(1, image_format = Rgba32Float, access = ReadWrite)]
    pub cloud_atlas_image: Handle<Image>,

    #[storage_texture(2, image_format = Rgba32Float, access = ReadWrite, dimension = "3d")]
    pub cloud_worley_image: Handle<Image>,

    #[storage_texture(3, image_format = Rgba32Float, access = ReadWrite)]
    pub sky_image: Handle<Image>,
}
