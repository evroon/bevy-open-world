use bevy::prelude::*;
use bevy_flight_sim::FlightSimPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Airbus PFD".to_string(),
                    resolution: (861, 844).into(),
                    canvas: Some("#bevy".to_owned()),
                    prevent_default_event_handling: false,
                    fit_canvas_to_parent: true,
                    ..default()
                }),

                ..default()
            }),
            FlightSimPlugin,
        ))
        .run();
}
