//! Demonstrates screen space reflections in deferred rendering.

use std::ops::Range;

use bevy::{
    anti_alias::fxaa::Fxaa,
    color::palettes::css::{BLACK, WHITE},
    core_pipeline::Skybox,
    image::{
        ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
        ImageSamplerDescriptor,
    },
    input::mouse::MouseWheel,
    math::{vec3, vec4},
    pbr::{
        DefaultOpaqueRendererMethod, ExtendedMaterial, MaterialExtension, ScreenSpaceReflections,
    },
    prelude::*,
    render::{
        render_resource::{AsBindGroup, ShaderType},
        view::Hdr,
    },
    shader::ShaderRef,
};

/// This example uses a shader source file from the assets subdirectory
const SHADER_ASSET_PATH: &str = "shaders/water_material.wgsl";

// The speed of camera movement.
const CAMERA_KEYBOARD_ZOOM_SPEED: f32 = 0.1;
const CAMERA_KEYBOARD_ORBIT_SPEED: f32 = 0.02;
const CAMERA_MOUSE_WHEEL_ZOOM_SPEED: f32 = 0.25;

// We clamp camera distances to this range.
const CAMERA_ZOOM_RANGE: Range<f32> = 2.0..12.0;

static TURN_SSR_OFF_HELP_TEXT: &str = "Press Space to turn screen-space reflections off";
static TURN_SSR_ON_HELP_TEXT: &str = "Press Space to turn screen-space reflections on";
static MOVE_CAMERA_HELP_TEXT: &str =
    "Press WASD or use the mouse wheel to pan and orbit the camera";
static SWITCH_TO_FLIGHT_HELMET_HELP_TEXT: &str = "Press Enter to switch to the flight helmet model";
static SWITCH_TO_CUBE_HELP_TEXT: &str = "Press Enter to switch to the cube model";

/// A custom [`ExtendedMaterial`] that creates animated water ripples.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct Water {
    /// The normal map image.
    ///
    /// Note that, like all normal maps, this must not be loaded as sRGB.
    #[texture(100)]
    #[sampler(101)]
    normals: Handle<Image>,

    // Parameters to the water shader.
    #[uniform(102)]
    settings: WaterSettings,
}

/// Parameters to the water shader.
#[derive(ShaderType, Debug, Clone)]
pub struct WaterSettings {
    /// How much to displace each octave each frame, in the u and v directions.
    /// Two octaves are packed into each `vec4`.
    octave_vectors: [Vec4; 2],
    /// How wide the waves are in each octave.
    octave_scales: Vec4,
    /// How high the waves are in each octave.
    octave_strengths: Vec4,
}

/// The current settings that the user has chosen.
#[derive(Resource)]
pub struct AppSettings {
    /// Whether screen space reflections are on.
    ssr_on: bool,
    /// Which model is being displayed.
    displayed_model: DisplayedModel,
}

/// Which model is being displayed.
#[derive(Default)]
enum DisplayedModel {
    /// The cube is being displayed.
    #[default]
    Cube,
    /// The flight helmet is being displayed.
    FlightHelmet,
}

fn main() {
    // Enable deferred rendering, which is necessary for screen-space
    // reflections at this time. Disable multisampled antialiasing, as deferred
    // rendering doesn't support that.
    App::new()
        .insert_resource(DefaultOpaqueRendererMethod::deferred())
        .init_resource::<AppSettings>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Screen Space Reflections Example".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, Water>>::default())
        .add_systems(Update, adjust_app_settings)
        .run();
}

// Spawns the water plane.
pub fn spawn_water(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    water_materials: &mut Assets<ExtendedMaterial<StandardMaterial, Water>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(1.0)))),
        MeshMaterial3d(water_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: BLACK.into(),
                perceptual_roughness: 0.0,
                ..default()
            },
            extension: Water {
                normals: asset_server.load_with_settings::<Image, ImageLoaderSettings>(
                    "textures/water_normals.png",
                    |settings| {
                        settings.is_srgb = false;
                        settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                            address_mode_u: ImageAddressMode::Repeat,
                            address_mode_v: ImageAddressMode::Repeat,
                            mag_filter: ImageFilterMode::Linear,
                            min_filter: ImageFilterMode::Linear,
                            ..default()
                        });
                    },
                ),
                // These water settings are just random values to create some
                // variety.
                settings: WaterSettings {
                    octave_vectors: [
                        vec4(0.080, 0.059, 0.073, -0.062),
                        vec4(0.153, 0.138, -0.149, -0.195),
                    ],
                    octave_scales: vec4(1.0, 2.1, 7.9, 14.9) * 5.0,
                    octave_strengths: vec4(0.16, 0.18, 0.093, 0.044),
                },
            },
        })),
        Transform::from_scale(Vec3::splat(100.0)),
    ));
}

// Spawns the camera.
pub fn spawn_camera(commands: &mut Commands, asset_server: &AssetServer) {
    // Create the camera. Add an environment map and skybox so the water has
    // something interesting to reflect, other than the cube. Enable deferred
    // rendering by adding depth and deferred prepasses. Turn on FXAA to make
    // the scene look a little nicer. Finally, add screen space reflections.
    commands
        .spawn((
            Camera3d::default(),
            Transform::from_translation(vec3(-1.25, 2.25, 4.5)).looking_at(Vec3::ZERO, Vec3::Y),
            Hdr,
            Msaa::Off,
        ))
        .insert(EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 5000.0,
            ..default()
        })
        .insert(Skybox {
            image: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            brightness: 5000.0,
            ..default()
        })
        .insert(ScreenSpaceReflections::default())
        .insert(Fxaa::default());
}

impl MaterialExtension for Water {
    fn deferred_fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

// Adjusts app settings per user input.
pub fn adjust_app_settings(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_settings: ResMut<AppSettings>,
    mut cameras: Query<Entity, With<Camera>>,
) {
    // If there are no changes, we're going to bail for efficiency. Record that
    // here.
    let mut any_changes = false;

    // If the user pressed Space, toggle SSR.
    if keyboard_input.just_pressed(KeyCode::Space) {
        app_settings.ssr_on = !app_settings.ssr_on;
        any_changes = true;
    }

    // If the user pressed Enter, switch models.
    if keyboard_input.just_pressed(KeyCode::Enter) {
        app_settings.displayed_model = match app_settings.displayed_model {
            DisplayedModel::Cube => DisplayedModel::FlightHelmet,
            DisplayedModel::FlightHelmet => DisplayedModel::Cube,
        };
        any_changes = true;
    }

    // If there were no changes, bail.
    if !any_changes {
        return;
    }

    // Update SSR settings.
    for camera in cameras.iter_mut() {
        if app_settings.ssr_on {
            commands
                .entity(camera)
                .insert(ScreenSpaceReflections::default());
        } else {
            commands.entity(camera).remove::<ScreenSpaceReflections>();
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            ssr_on: true,
            displayed_model: default(),
        }
    }
}
