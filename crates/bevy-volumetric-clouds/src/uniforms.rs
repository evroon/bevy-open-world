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
    pub march_steps: u32,
    pub self_shadow_steps: u32,
    pub earth_radius: f32,
    pub bottom: f32,
    pub top: f32,
    pub coverage: f32,
    pub detail_strength: f32,
    pub base_edge_softness: f32,
    pub bottom_softness: f32,
    pub density: f32,
    pub shadow_march_step_size: f32,
    pub shadow_march_step_multiply: f32,
    pub forward_scattering_g: f32,
    pub backward_scattering_g: f32,
    pub scattering_lerp: f32,
    pub ambient_color_top: Vec4,
    pub ambient_color_bottom: Vec4,
    pub min_transmittance: f32,
    pub base_scale: f32,
    pub detail_scale: f32,
    pub sun_dir: Vec4,
    pub sun_color: Vec4,
    pub camera_ro: Vec4,
    pub camera_fl: f32,
    pub debug: f32,
    pub time: f32,
    pub reprojection_strength: f32,
    pub render_resolution: Vec2,
    pub camera: Mat3,
}

impl Default for CloudsUniform {
    fn default() -> Self {
        Self {
            march_steps: 0,
            self_shadow_steps: 0,
            earth_radius: 0.0,
            bottom: 0.,
            top: 0.,
            coverage: 0.0,
            detail_strength: 0.0,
            base_edge_softness: 0.0,
            bottom_softness: 0.0,
            density: 0.0,
            shadow_march_step_size: 0.0,
            shadow_march_step_multiply: 0.0,
            forward_scattering_g: 0.0,
            backward_scattering_g: 0.0,
            scattering_lerp: 0.0,
            ambient_color_top: Vec4::ZERO,
            ambient_color_bottom: Vec4::ZERO,
            min_transmittance: 0.0,
            base_scale: 0.0,
            detail_scale: 0.0,
            sun_dir: Vec4::ZERO,
            sun_color: Vec4::ZERO,
            camera_ro: Vec4::ZERO,
            camera_fl: 0.0,
            debug: 0.0,
            time: 0.0,
            reprojection_strength: 0.95,
            render_resolution: Vec2::new(1920.0, 1080.0),
            camera: Mat3::IDENTITY,
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
