use bevy::shader::ShaderRef;
use bevy::sprite_render::{Material2d, Material2dPlugin};
use bevy::window::WindowResized;
use bevy::{prelude::*, reflect::TypePath, render::render_resource::AsBindGroup};

const SHADER_ASSET_PATH: &str = "shaders/atmosphere.wgsl";

pub fn on_resize_system(
    mut mesh: Single<(&Mesh2d, &mut Transform)>,
    mut resize_reader: MessageReader<WindowResized>,
) {
    for e in resize_reader.read() {
        mesh.1.scale = Vec3::new(e.width, e.height, 1.0);
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            Material2dPlugin::<CustomMaterial>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, on_resize_system)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
) {
    commands.spawn(Camera2d);
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(materials.add(CustomMaterial {})),
        Transform::default().with_scale(Vec3::splat(1.0)),
    ));
}

impl Material2d for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct CustomMaterial {}
