use bevy::{
    color::palettes::css::BLACK,
    image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
    math::Affine2,
    pbr::OpaqueRendererMethod,
    prelude::*,
};

pub fn spawn_runway(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    _asset_server: Res<AssetServer>,
) {
    let texture_repeat = 10.0;
    let _repeat_loader = |s: &mut _| {
        *s = ImageLoaderSettings {
            sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::Repeat,
                ..default()
            }),
            ..default()
        }
    };
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(1.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            // metallic_roughness_texture: Some(asset_server.load_with_settings(
            //     "textures/asphalt-clean/clean_asphalt_rough_2k.jpg",
            //     repeat_loader,
            // )),
            // normal_map_texture: Some(asset_server.load_with_settings(
            //     "textures/asphalt-clean/clean_asphalt_nor_gl_2k.jpg",
            //     repeat_loader,
            // )),
            // base_color_texture: Some(asset_server.load_with_settings(
            //     "textures/asphalt-clean/clean_asphalt_diff_2k.jpg",
            //     repeat_loader,
            // )),
            // occlusion_texture: Some(asset_server.load_with_settings(
            //     "textures/asphalt-clean/clean_asphalt_ao_2k.jpg",
            //     repeat_loader,
            // )),
            base_color: BLACK.into(),
            reflectance: 0.025,
            opaque_render_method: OpaqueRendererMethod::Auto,
            uv_transform: Affine2::from_scale(Vec2::new(
                1.0 * texture_repeat,
                20.0 * texture_repeat,
            )),
            ..Default::default()
        })),
        Transform::from_xyz(0.0, 0.2, 0.0).with_scale(Vec3::new(0.1, 1.0, 2.0)),
    ));
}

pub fn spawn_aircraft(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        SceneRoot(asset_server.load("models/low_poly_spaceship/scene.gltf#Scene0")),
        Transform::from_xyz(0.0, 0.3, 0.0).with_scale(Vec3::splat(0.01)),
    ));
}
