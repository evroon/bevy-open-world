use bevy::{
    color::LinearRgba,
    prelude::{
        Assets, Color, Cuboid, FromWorld, Handle, Mesh, Resource, StandardMaterial, Vec3, World,
        default,
    },
};
use std::collections::HashMap;
use strum::IntoEnumIterator;

use crate::{mesh::LightColor, osm_types::BuildingClass};

type Reflectance = f32;
type Roughness = f32;
pub trait MaterialForBuilding {
    fn to_material_params(&self) -> (Reflectance, Roughness);
}

const DEFAULT_MATERIAL_PROPS: (Reflectance, Roughness) = (0.6, 0.5);
impl MaterialForBuilding for BuildingClass {
    fn to_material_params(&self) -> (Reflectance, Roughness) {
        match self {
            BuildingClass::Commercial => (0.3, 0.8),
            _ => DEFAULT_MATERIAL_PROPS,
        }
    }
}

#[derive(Resource)]
pub struct MapMaterialHandle {
    pub roof: Handle<StandardMaterial>,
    pub roofs: HashMap<BuildingClass, Handle<StandardMaterial>>,
    pub walls: HashMap<BuildingClass, Handle<StandardMaterial>>,
    pub lights: HashMap<LightColor, Handle<StandardMaterial>>,
    pub unknown_building: Handle<StandardMaterial>,
    pub unknown_building_roof: Handle<StandardMaterial>,
}
impl FromWorld for MapMaterialHandle {
    fn from_world(world: &mut World) -> Self {
        let mut standard_materials = world.resource_mut::<Assets<StandardMaterial>>();

        let roof_color = Color::linear_rgb(0.3, 0.3, 0.2);
        let roof = standard_materials.add(StandardMaterial {
            base_color: roof_color,
            depth_bias: 0.,
            reflectance: 0.5,
            perceptual_roughness: 0.75,
            ..default()
        });

        let mut roofs: HashMap<BuildingClass, Handle<StandardMaterial>> = HashMap::new();
        for building_class in BuildingClass::iter() {
            let color = Color::from(&building_class);
            let (reflectance, roughness) = building_class.to_material_params();
            let roof_color_handle = standard_materials.add(StandardMaterial {
                base_color: color,
                depth_bias: 0.,
                reflectance,
                perceptual_roughness: roughness,
                ..default()
            });
            roofs
                .entry(building_class)
                .or_insert_with_key(|_key| roof_color_handle);
        }

        let mut walls: HashMap<BuildingClass, Handle<StandardMaterial>> = HashMap::new();
        for building_class in BuildingClass::iter() {
            let color = Color::from(&building_class);
            let (reflectance, roughness) = building_class.to_material_params();
            let wall_color_handle = standard_materials.add(StandardMaterial {
                base_color: color,
                depth_bias: 0.,
                reflectance,
                perceptual_roughness: roughness,
                ..default()
            });
            walls
                .entry(building_class)
                .or_insert_with_key(|_key| wall_color_handle);
        }

        let unknown_building_color = Color::linear_rgb(0.3, 0.3, 0.3);
        let unknown_building = standard_materials.add(StandardMaterial {
            base_color: unknown_building_color,
            depth_bias: 0.,
            reflectance: 0.5,
            perceptual_roughness: 0.7,
            ..default()
        });

        let unknown_building_roof_color = Color::WHITE;
        let unknown_building_roof = standard_materials.add(StandardMaterial {
            base_color: unknown_building_roof_color,
            depth_bias: 0.,
            reflectance: 0.5,
            perceptual_roughness: 0.7,
            ..default()
        });

        Self {
            roof,
            roofs,
            walls,
            unknown_building,
            unknown_building_roof,
            lights: (vec![
                (
                    LightColor::White,
                    standard_materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        depth_bias: 0.,
                        reflectance: 0.5,
                        perceptual_roughness: 0.7,
                        emissive: LinearRgba::rgb(100.0, 100.0, 255.0),
                        ..default()
                    }),
                ),
                (
                    LightColor::Yellow,
                    standard_materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        depth_bias: 0.,
                        reflectance: 0.5,
                        perceptual_roughness: 0.7,
                        emissive: LinearRgba::rgb(100.0, 100.0, 10.0),
                        ..default()
                    }),
                ),
                (
                    LightColor::Orange,
                    standard_materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        depth_bias: 0.,
                        reflectance: 0.5,
                        perceptual_roughness: 0.7,
                        emissive: LinearRgba::rgb(223., 113., 48.),
                        ..default()
                    }),
                ),
                (
                    LightColor::Red,
                    standard_materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        depth_bias: 0.,
                        reflectance: 0.5,
                        perceptual_roughness: 0.7,
                        emissive: LinearRgba::rgb(255., 0., 0.),
                        ..default()
                    }),
                ),
                (
                    LightColor::Blue,
                    standard_materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        depth_bias: 0.,
                        reflectance: 0.5,
                        perceptual_roughness: 0.7,
                        emissive: LinearRgba::rgb(10.0, 10.0, 100.0),
                        ..default()
                    }),
                ),
                (
                    LightColor::Green,
                    standard_materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        depth_bias: 0.,
                        reflectance: 0.5,
                        perceptual_roughness: 0.7,
                        emissive: LinearRgba::rgb(0., 255., 0.),
                        ..default()
                    }),
                ),
            ])
            .into_iter()
            .collect(),
        }
    }
}

/// Shared mesh handles for map rendering.  Analogous to [`MapMaterialHandle`]
/// but for [`Mesh`] assets that are reused across chunks.
#[derive(Resource)]
pub struct MapMeshHandle {
    /// A single thin cuboid used for every street-light instance.
    pub light: Handle<Mesh>,
}

impl FromWorld for MapMeshHandle {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let light = meshes.add(Cuboid::from_size(Vec3::new(0.003, 5.0, 0.003)));
        Self { light }
    }
}
