#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use std::{fs::File, io::BufWriter};

use bevy::prelude::*;
use bevy::window::WindowClosing;
use serialization::{deserialize_transform, serialize_transform};

mod serialization;

/// A component that saves a [`Transform`] to disk and restores it when you reopn the application.
///
/// It requires a [`Transform`]. You can omit it and Bevy will create a [`Transform::IDENTITY`] for
/// you.
///
/// ```rust
/// use bevy_where_was_i::WhereWasI;
///
/// WhereWasI::from_name("my_entity");
/// ```
#[derive(Component)]
#[require(Transform)]
pub struct WhereWasI {
    name: String,
}

impl WhereWasI {
    /// Construct a [`WhereWasI`] plugin with a name
    pub fn from_name(name: &str) -> Self {
        Self { name: name.into() }
    }

    /// A shorthand used for cameras
    ///
    /// Equivalent to:
    /// ```rust
    /// use bevy_where_was_i::WhereWasI;
    ///
    /// WhereWasI::from_name("camera");
    /// ```
    pub fn camera() -> Self {
        WhereWasI::from_name("camera")
    }
}

/// A [`Resource`] to store the `directory` in so we can access in the systems of this plugin.
#[derive(Resource)]
struct WhereWasIConfig {
    directory: String,
}

/// Plugin that saves the [`Transform`] state after closing a Bevy application, and restores it
/// when launching the application again.
pub struct WhereWasIPlugin {
    /// The directory where savefiles will be stored and loaded from
    pub directory: String,
}

impl Default for WhereWasIPlugin {
    fn default() -> Self {
        Self {
            directory: "./assets/saves".into(),
        }
    }
}

impl Plugin for WhereWasIPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WhereWasIConfig {
            directory: self.directory.clone(),
        })
        .add_systems(Update, save_state)
        .add_systems(PostStartup, load_state);
    }
}

/// Read file `filename` line-by-line
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

/// Load the state of all [`Transform`]s belonging to [`WhereWasI`] components
fn load_state(mut to_save: Query<(&WhereWasI, &mut Transform)>, config: Res<WhereWasIConfig>) {
    let mut initialized = 0;

    for (where_was_i, mut transform) in to_save.iter_mut() {
        let (directory, filename) = (&config.directory, &where_was_i.name);
        let filepath = format!("{directory}/{filename}.state");

        if let Ok(contents) = read_lines(filepath) {
            match deserialize_transform(contents) {
                Ok(new) => {
                    *transform = new;
                    initialized += 1;
                }
                Err(err) => {
                    error!("Could not deserialize transform: {}", err.message);
                }
            }
        }
    }

    info!("Initialized {} transform(s)", initialized);
}

/// Saves the state of all [`Transform`]s belonging to [`WhereWasI`] components when closing a
/// window
///
/// Note: this doesn't work for WASM.
fn save_state(
    mut events: MessageReader<WindowClosing>,
    to_save: Query<(&WhereWasI, &Transform)>,
    config: Res<WhereWasIConfig>,
) {
    let directory = &config.directory;
    let mut saved_files = 0;

    if events.read().next().is_some() {
        for (where_was_i, transform) in to_save.iter() {
            let filename = where_was_i.name.clone();

            if let Ok(false) = fs::exists(directory) {
                fs::create_dir_all(directory).expect("Could not create directory");
            }

            let mut writer = BufWriter::new(
                File::create(format!("{directory}/{filename}.state"))
                    .expect("Error occurred while opening file"),
            );

            #[cfg(not(target_arch = "wasm32"))]
            serialize_transform(&mut writer, transform)
                .expect("Error occurred while writing to disk");

            saved_files += 1;
        }
        info!("Saved {} transforms to: {}", saved_files, directory);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRANSFORM: Transform = Transform {
        translation: Vec3::new(4.0, 3.5, -2.0),
        rotation: Quat::from_xyzw(-0.1, 0.7, 0.4, 0.6),
        scale: Vec3::new(12.6, -1.0, 2.4),
    };
    const SAVE_STATE_FILE: &str = "assets/tests/system_save_test.state";

    fn setup_camera_with_transform(mut commands: Commands<'_, '_>) {
        commands.spawn((WhereWasI::from_name("system_save_test"), TRANSFORM));
    }

    fn setup_camera_without_transform(mut commands: Commands<'_, '_>) {
        commands.spawn((Camera::default(), WhereWasI::camera()));
    }

    #[test]
    fn test_save() {
        let mut app = App::new();

        if let Ok(true) = fs::exists(SAVE_STATE_FILE) {
            fs::remove_file("assets/tests/system_save_test.state").unwrap();
        }
        assert!(!fs::exists(SAVE_STATE_FILE).unwrap());

        app.insert_resource(WhereWasIConfig {
            directory: "assets/tests".into(),
        });
        app.add_message::<WindowClosing>();
        app.add_systems(Startup, setup_camera_with_transform);
        app.add_systems(Update, save_state);

        // Send an `WindowClosing` event
        app.world_mut()
            .resource_mut::<Messages<WindowClosing>>()
            .write(WindowClosing {
                window: Entity::from_raw_u32(322).unwrap(),
            });

        app.update();

        let lines = read_lines("assets/tests/system_save_test.state").unwrap();
        assert_eq!(deserialize_transform(lines).unwrap(), TRANSFORM);

        fs::remove_file("assets/tests/system_save_test.state").unwrap();
    }

    #[test]
    fn test_load() {
        let mut app = App::new();

        app.insert_resource(WhereWasIConfig {
            directory: "assets/tests".into(),
        });
        app.add_systems(Startup, setup_camera_without_transform);
        app.add_systems(Update, load_state);

        app.update();

        let result = app
            .world_mut()
            .query::<&Transform>()
            .single(app.world())
            .expect("`load_state` should have added a Transform to the world");

        const TRANSFORM: Transform = Transform {
            translation: Vec3::new(10.000002, 10.0, 10.0),
            rotation: Quat::from_xyzw(-0.27984813, 0.36470526, 0.11591691, 0.88047624),
            scale: Vec3::new(1.0, 1.0, 1.0),
        };
        assert_eq!(*result, TRANSFORM);
    }
}
