use bevy::prelude::*;

use crate::{clickhouse::DataFetch, config::ADSBConfig, state::Aircraft};

pub fn spawn_aircraft(
    mut commands: Commands,
    mut adsb: ResMut<ADSBConfig>,
    mut data: ResMut<DataFetch>,
) {
    while let Some(aircraft) = data.create.pop() {
        if adsb.planes >= 5_000 {
            return;
        }
        adsb.planes += 1;
        commands.spawn((
            Mesh3d(adsb.mesh.clone()),
            aircraft.clone(),
            MeshMaterial3d(adsb.material.clone()),
            Transform::from_translation(aircraft.last_state.get_position_in_world()),
        ));
    }
}

pub fn move_aircraft(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Aircraft)>,
    mut config: ResMut<ADSBConfig>,
    mut data: ResMut<DataFetch>,
) {
    config.ticks += 1;
    config.time = config.time + config.target_delta;

    for (entity, mut transform, mut aircraft) in query.iter_mut() {
        if let Some(ac_update) = data.update.get_mut(&aircraft.icao) {
            aircraft.buffer.append(ac_update);
            aircraft.seek_buffer(config.time);
        }

        if let Some(state) = aircraft.buffer.peek() {
            let time_left = state.timestamp - config.time;
            let time_lerp = config.target_delta.as_seconds_f32() / time_left.as_seconds_f32();

            if time_lerp < 1.0 && time_lerp > 0.0 {
                let current_pos = transform.translation;
                let target_pos = state.get_position_in_world();
                let direction = target_pos - current_pos;

                transform.translation = current_pos + time_lerp * direction;
                transform.rotation = Quat::from_rotation_arc(Vec3::Y, direction.normalize());
                aircraft.last_update = config.time;
            }
        }

        if config.time - aircraft.last_update > config.remove_aircraft_after_last_signal {
            commands.get_entity(entity).unwrap().clear();
            commands.get_entity(entity).unwrap().despawn();
            config.planes -= 1;
            data.icaos.remove(&aircraft.icao);
        }
    }
}
