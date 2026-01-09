use core::f32::consts::PI;

use bevy::prelude::*;

#[derive(Component)]
pub struct SkyboxPlane {
    pub orig_translation: Vec3,
}

pub fn init_skybox(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    let box_size = 1.0;

    let mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(box_size)));
    let nx_material = StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/milkyway/nx.png")),
        unlit: true,
        ..default()
    };
    let ny_material = StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/milkyway/ny.png")),
        unlit: true,
        ..default()
    };
    let nz_material = StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/milkyway/nz.png")),
        unlit: true,
        ..default()
    };
    let px_material = StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/milkyway/px.png")),
        unlit: true,
        ..default()
    };
    let py_material = StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/milkyway/py.png")),
        unlit: true,
        ..default()
    };
    let pz_material = StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/milkyway/pz.png")),
        unlit: true,
        ..default()
    };

    // negative x
    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(standard_materials.add(nx_material)),
        Transform::from_translation(Vec3::new(-box_size, 0.0, 0.0))
            .with_rotation(Quat::from_rotation_z(-PI * 0.5) * Quat::from_rotation_y(PI * 0.5)),
        SkyboxPlane {
            orig_translation: Vec3::new(-box_size, 0.0, 0.0),
        },
    ));

    // negative y
    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(standard_materials.add(ny_material)),
        Transform::from_translation(Vec3::new(0.0, -box_size, 0.0)),
        SkyboxPlane {
            orig_translation: Vec3::new(0.0, -box_size, 0.0),
        },
    ));

    // negative z
    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(standard_materials.add(pz_material)),
        Transform::from_translation(Vec3::new(0.0, 0.0, -box_size))
            .with_rotation(Quat::from_rotation_x(PI * 0.5)),
        SkyboxPlane {
            orig_translation: Vec3::new(0.0, 0.0, -box_size),
        },
    ));

    // positive x
    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(standard_materials.add(px_material)),
        Transform::from_translation(Vec3::new(box_size, 0.0, 0.0))
            .with_rotation(Quat::from_rotation_z(PI * 0.5) * Quat::from_rotation_y(-PI * 0.5)),
        SkyboxPlane {
            orig_translation: Vec3::new(box_size, 0.0, 0.0),
        },
    ));

    // positive y
    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(standard_materials.add(py_material)),
        Transform::from_translation(Vec3::new(0.0, box_size, 0.0))
            .with_rotation(Quat::from_rotation_z(PI) * Quat::from_rotation_y(PI)),
        SkyboxPlane {
            orig_translation: Vec3::new(0.0, box_size, 0.0),
        },
    ));

    // positive z
    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(standard_materials.add(nz_material)),
        Transform::from_translation(Vec3::new(0.0, 0.0, box_size))
            .with_rotation(Quat::from_rotation_x(-PI * 0.5) * Quat::from_rotation_y(PI)),
        SkyboxPlane {
            orig_translation: Vec3::new(0.0, 0.0, box_size),
        },
    ));
}

pub fn update_skybox(
    camera: Single<(&Transform, &Camera, &Projection), Without<SkyboxPlane>>,
    mut skybox: Query<(&mut Transform, &SkyboxPlane)>,
) {
    let far = match camera.2 {
        Projection::Perspective(pers) => pers.far,
        _ => {
            panic!("unexpected projection")
        }
    };
    let scale = far * 1000000.0;

    for (mut transform, plane) in skybox.iter_mut() {
        transform.scale = Vec3::splat(scale);
        transform.translation = camera.0.translation + plane.orig_translation * scale;
    }
}
