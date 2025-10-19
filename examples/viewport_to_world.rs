use bevy::{camera::ViewportConversionError, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, draw_cursor)
        .run();
}

fn viewport_to_world(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    viewport_position: Vec2,
) -> Result<Ray3d, ViewportConversionError> {
    let target_rect = camera
        .logical_viewport_rect()
        .ok_or(ViewportConversionError::NoViewportSize)?;

    let view_from_clip = camera.computed.clip_from_view.inverse(); // inverse projection matrix
    let world_from_view = camera_transform.to_matrix(); // inverse view matrix

    let rect_relative = viewport_position / target_rect.size();

    // Flip the Y co-ordinate from the top to the bottom to enter NDC.
    let ndc_xy = (rect_relative * 2.0 - Vec2::ONE) * Vec2::new(1.0, -1.0);

    let ray_clip = Vec4::new(ndc_xy.x, ndc_xy.y, -1.0, 1.0);
    let ray_eye = view_from_clip * ray_clip;
    let ray_world = world_from_view * Vec4::new(ray_eye.x, ray_eye.y, -1.0, 0.0);

    // The fallible direction constructor ensures that direction isn't NaN.
    Dir3::new(ray_world.xyz())
        .map_err(|_| ViewportConversionError::InvalidData)
        .map(|direction| Ray3d {
            origin: camera_transform.affine().translation.to_vec3(),
            direction,
        })
}

fn draw_cursor(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    ground: Single<&GlobalTransform, With<Ground>>,
    window: Single<&Window>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = *camera_query;

    if let Some(cursor_position) = window.cursor_position()
        // Calculate a ray pointing from the camera into the world based on the cursor's position.
        && let Ok(ray) = viewport_to_world(camera, camera_transform, cursor_position)
        // Calculate if and at what distance the ray is hitting the ground plane.
        && let Some(distance) =
            ray.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up()))
    {
        let point = ray.get_point(distance);

        // Draw a circle just above the ground plane at that position.
        gizmos.circle(
            Isometry3d::new(
                point + ground.up() * 0.01,
                Quat::from_rotation_arc(Vec3::Z, ground.up().as_vec3()),
            ),
            0.2,
            Color::WHITE,
        );
    }
}

#[derive(Component)]
struct Ground;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20., 20.))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Ground,
    ));

    // light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(15.0, 5.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
