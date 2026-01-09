pub mod mesh;
pub mod osm_types;
mod theme;

use std::fs::File;
extern crate osm_xml as osm;
use crate::mesh::{spawn_fill_mesh, spawn_stroke_mesh};
use crate::{
    building::{Building, polygon_building},
    material::{MapMaterialHandle, OSMConfig},
    task_pool::{handle_tasks, spawn_task},
};
use bevy::prelude::*;
use lyon::math::point;
use mesh::BuildInstruction;
use theme::get_way_build_instruction;
mod building;
mod material;
mod task_pool;

pub struct OSMPlugin;

impl Plugin for OSMPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapMaterialHandle>()
            .init_resource::<OSMConfig>()
            .add_systems(Startup, spawn_task)
            .add_systems(Update, handle_tasks);
    }
}

pub fn build_tile() -> Vec<Building> {
    let mut result = Vec::new();
    let mut rng = rand::rng();
    let f = File::open("assets/osm/manhattan.osm").unwrap();
    let doc = &osm::OSM::parse(f).unwrap();
    let mut position_sum = (0.0, 0.0);
    let mut node_count = 0.0;

    for way in doc.ways.values() {
        for n in &way.nodes {
            if let osm::Reference::Node(node) = doc.resolve_reference(n) {
                let (x, y) = (node.lat as f32, node.lon as f32);
                position_sum.0 += x;
                position_sum.1 += y;
                node_count += 1.0;
            }
        }
    }
    let position_avg = (position_sum.0 / node_count, position_sum.1 / node_count);

    for way in doc.ways.values() {
        let mut points = Vec::new();
        for n in &way.nodes {
            if let osm::Reference::Node(node) = doc.resolve_reference(n) {
                points.push(point(
                    (node.lat as f32 - position_avg.0) * 1000.0,
                    (node.lon as f32 - position_avg.1) * 1000.0,
                ));
            }
        }
        match get_way_build_instruction(&way.tags) {
            BuildInstruction::Fill(fill) => {
                // spawn_fill_mesh(&mut commands, &mut meshes, &mut materials, points, fill);
            }
            BuildInstruction::Stroke(stroke) => {
                // spawn_stroke_mesh(&mut commands, &mut meshes, &mut materials, points, stroke);
            }
            BuildInstruction::Building(building) => {
                result.push(polygon_building(&building, points, &mut rng));
            }
            BuildInstruction::None => {}
        }
    }

    println!("Finished building {} buildings", result.len());
    result
    // for rel in doc.relations.values() {
    //     let mut points = Vec::new();
    //     for m in &rel.members {
    //         if let osm::Member::Way(way, t) = m {
    //             if t != "outer" {
    //                 continue;
    //             }
    //             if let osm::Reference::Way(way) = doc.resolve_reference(way) {
    //                 for n in &way.nodes {
    //                     if let osm::Reference::Node(node) = doc.resolve_reference(n) {
    //                         points.push(point(
    //                             (node.lat as f32 - position_avg.0) * 1000.0,
    //                             (node.lon as f32 - position_avg.1) * 1000.0,
    //                         ));
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     if points.is_empty() {
    //         continue;
    //     }
    //     match get_rel_build_instruction(&rel.tags) {
    //         BuildInstruction::Fill(fill) => {
    //             spawn_fill_mesh(&mut commands, &mut meshes, &mut materials, points, fill);
    //         }
    //         BuildInstruction::Stroke(stroke) => {
    //             spawn_stroke_mesh(&mut commands, &mut meshes, &mut materials, points, stroke);
    //         }
    //         BuildInstruction::Building(_) => {}
    //         BuildInstruction::None => {}
    //     }
    // }
}
