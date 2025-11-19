pub mod material;
pub mod mesh;
pub mod quadtree;
pub mod system;

use bevy::{pbr::ExtendedMaterial, prelude::*};

use crate::{material::PlanetMaterial, system::update_quadtree};

pub struct PlanetsPlugin;

impl Plugin for PlanetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, PlanetMaterial>,
        >::default())
            .add_systems(Update, update_quadtree);
    }
}
