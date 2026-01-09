use core::f32::consts::PI;

use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension, OpaqueRendererMethod},
    prelude::*,
    render::render_resource::*,
    shader::ShaderRef,
};

use crate::{EARTH_RADIUS, EarthConfig};

/// This example uses a shader source file from the assets subdirectory
const SHADER_ASSET_PATH: &str = "shaders/earth.wgsl";

pub fn setup_earth(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, EarthMaterialExtension>>>,
    asset_server: Res<AssetServer>,
    earth_config: Res<EarthConfig>,
) {
    commands.spawn((
        // We can't use an icosphere because of UV seams
        // https://github.com/bevyengine/bevy/issues/4987
        Mesh3d(meshes.add(Sphere::new(EARTH_RADIUS).mesh().uv(32, 32))),
        Transform::from_rotation(
            Quat::from_rotation_y(0.5 * PI) * Quat::from_rotation_x(-PI * 0.5),
        ),
        MeshMaterial3d(materials.add(ExtendedMaterial {
            base: StandardMaterial {
                metallic_roughness_texture: Some(
                    asset_server.load("textures/earth/earth_bump_roughness_clouds_4096.jpg"),
                ),
                // specular_texture: Some(asset_server.load("textures/earth/earth_specular_2048.jpg")),
                normal_map_texture: Some(asset_server.load("textures/earth/earth_normal_2048.jpg")),
                metallic: 0.9,
                opaque_render_method: OpaqueRendererMethod::Auto,
                ..Default::default()
            },
            extension: EarthMaterialExtension {
                base_color_day: Some(asset_server.load("textures/earth/earth_day_4096.jpg")),
                base_color_night: Some(asset_server.load("textures/earth/earth_night_4096.jpg")),
                sun_dir: earth_config.sun_direction * Vec3::Z,
                emission_strength: earth_config.emission_strength,
                emission_threshold: earth_config.transition_fraction,
                transition_fraction: earth_config.emission_threshold,
            },
        })),
    ));
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct EarthMaterialExtension {
    #[texture(100)]
    #[sampler(101)]
    base_color_day: Option<Handle<Image>>,
    #[texture(102)]
    #[sampler(103)]
    base_color_night: Option<Handle<Image>>,
    #[uniform(104)]
    sun_dir: Vec3,
    #[uniform(105)]
    emission_strength: f32,
    #[uniform(106)]
    emission_threshold: f32,
    #[uniform(107)]
    transition_fraction: f32,
}

impl MaterialExtension for EarthMaterialExtension {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

pub fn animate_materials(
    earth_config: Res<EarthConfig>,
    material_handles: Query<
        &MeshMaterial3d<ExtendedMaterial<StandardMaterial, EarthMaterialExtension>>,
    >,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, EarthMaterialExtension>>>,
) {
    for material_handle in material_handles.iter() {
        if let Some(material) = materials.get_mut(material_handle) {
            material.extension.sun_dir = earth_config.sun_direction * Vec3::Z;
            material.extension.emission_strength = earth_config.emission_strength;
            material.extension.transition_fraction = earth_config.transition_fraction;
            material.extension.emission_threshold = earth_config.emission_threshold;
        }
    }
}
