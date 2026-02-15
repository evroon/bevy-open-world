extern crate osm_xml as osm;
use crate::{
    building::{Building, polygon_building},
    location::{Location, get_osm_for_location},
    mesh::{BuildInstruction, spawn_fill_mesh, spawn_stroke_mesh},
    theme::get_way_build_instruction,
};
use bevy::prelude::*;
use lyon::math::point;

pub fn build_tile(location: Location) -> (Vec<Building>, Vec<Mesh>) {
    let mut buildings = Vec::new();
    let mut meshes = Vec::new();
    let mut rng = rand::rng();

    let doc = get_osm_for_location(location);
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
                meshes.push(spawn_fill_mesh(points, fill));
            }
            BuildInstruction::Stroke(stroke) => {
                meshes.push(spawn_stroke_mesh(points, stroke));
            }
            BuildInstruction::Building(building) => {
                buildings.push(polygon_building(&building, points, &mut rng));
            }
            BuildInstruction::None => {}
        }
    }

    println!(
        "Finished building {} buildings, {} meshes",
        buildings.len(),
        meshes.len()
    );
    (buildings, meshes)
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
