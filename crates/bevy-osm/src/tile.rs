extern crate osm_xml as osm;
use crate::{
    building::{Building, polygon_building},
    chunk::{Chunk, get_osm_for_chunk},
    mesh::{BuildInstruction, LightInstruction, spawn_stroke_mesh},
    theme::get_way_build_instruction,
};
use bevy::prelude::*;
use lyon::math::point;

pub fn build_tile(
    chunk: Chunk,
    lat_lon_origin: Vec2,
) -> (Vec<Building>, Vec<Mesh>, Vec<LightInstruction>) {
    let mut buildings = Vec::new();
    let mut meshes = Vec::new();
    let mut lights = Vec::new();
    let mut rng = rand::rng();

    let doc = get_osm_for_chunk(chunk.clone());

    for way in doc.ways.values() {
        let mut points = Vec::new();
        for n in &way.nodes {
            if let osm::Reference::Node(node) = doc.resolve_reference(n) {
                let (x, z) = chunk
                    .lat_lon_to_world(Vec2::new(node.lat as f32, node.lon as f32), lat_lon_origin);
                points.push(point(x as f32, z as f32));
            }
        }
        match get_way_build_instruction(&way.tags) {
            BuildInstruction::Fill(_) => {
                // meshes.push(spawn_fill_mesh(points, fill));
            }
            BuildInstruction::Stroke(stroke) => {
                let center = points[0];
                meshes.push(spawn_stroke_mesh(points, stroke));
                lights.push(LightInstruction {
                    trans: Vec3::new(center.x, 2.0, center.y),
                });
            }
            BuildInstruction::Building(building) => {
                buildings.push(polygon_building(&building, points, &mut rng));
            }
            BuildInstruction::Light(_light) => {}
            BuildInstruction::None => {}
        }
    }

    info!(
        "Finished building {} buildings, {} meshes, {} lights",
        buildings.len(),
        meshes.len(),
        lights.len()
    );
    (buildings, meshes, lights)
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
