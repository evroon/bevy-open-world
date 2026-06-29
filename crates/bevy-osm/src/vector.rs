use std::fs::File;
use std::io::Read;

use crate::building::{polygon_building, spawn_building};
use crate::cache::{cache_vector_tile_for_chunk, get_openfreemap_cache_path};
use crate::chunk::Chunk;
use crate::layer::OMTLayer;
use crate::material::MapMaterialHandle;
use crate::mesh::{BuildInstruction, spawn_fill_mesh, spawn_stroke_mesh};
use crate::tag::Tag;
use crate::theme::get_way_build_instruction_openfreemap;
use bevy::prelude::*;
use geo_types::Geometry;
use lyon::geom::euclid::{Point2D, UnknownUnit};
use lyon::math::point;
use mvt_reader::feature::Feature;
use mvt_reader::layer::Layer;
use mvt_reader::{Reader, error::ParserError, feature::Value};

pub type PolygonInstruction = (Vec<Tag>, OMTLayer, Vec<Point2D<f32, UnknownUnit>>);

pub fn spawn_pbf(
    instructions: Vec<PolygonInstruction>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    map_materials: Res<MapMaterialHandle>,
) {
    let mut rng = rand::rng();
    let building_material: Handle<StandardMaterial> = map_materials.unknown_building.clone();

    for (tags, layer, polygon) in instructions {
        match get_way_build_instruction_openfreemap(
            tags.iter()
                .map(|tag| Tag {
                    key: tag.key.clone(),
                    val: tag.val.clone(),
                })
                .collect(),
            layer,
        ) {
            BuildInstruction::Fill(fill) => {
                let mesh = spawn_fill_mesh(polygon.iter().map(|p| point(p.x, p.y)).collect(), fill);

                commands.spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(building_material.clone()),
                    Transform::IDENTITY,
                ));
            }
            BuildInstruction::Stroke(stroke) => {
                let mesh =
                    spawn_stroke_mesh(polygon.iter().map(|p| point(p.x, p.y)).collect(), stroke);

                commands.spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(building_material.clone()),
                    Transform::IDENTITY,
                ));
            }
            BuildInstruction::Building(building) => {
                let building = polygon_building(&building, polygon, &mut rng);
                let mesh3ds = spawn_building(&building)
                    .into_iter()
                    .map(|(mesh, t)| (Mesh3d(meshes.add(mesh)), t))
                    .collect::<Vec<(Mesh3d, Transform)>>();

                for (mesh, trans) in mesh3ds {
                    commands.spawn((mesh, MeshMaterial3d(building_material.clone()), trans));
                }
            }
            BuildInstruction::Light(_light) => {}
            BuildInstruction::None => {}
        }
    }
}

pub fn spawn_chunk(
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    map_materials: Res<MapMaterialHandle>,
    chunk: &Chunk,
) {
    cache_vector_tile_for_chunk(&chunk);
    let mut bytes = Vec::new();
    File::open(get_openfreemap_cache_path(&chunk))
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    spawn_pbf(parse_pbf(bytes).unwrap(), commands, meshes, map_materials);
}

pub fn parse_pbf(pbf_data: Vec<u8>) -> Result<Vec<PolygonInstruction>, ParserError> {
    let reader = Reader::new(pbf_data)?;
    let mut polygons = Vec::new();

    for layer in reader.get_layer_metadata()? {
        let layer_name = OMTLayer::from_name(&layer.name);
        if layer_name != OMTLayer::TransportationName && layer_name != OMTLayer::WaterName {
            polygons.extend(process_layer(&reader, &layer)?);
        }
    }

    Ok(polygons)
}

fn get_tags(properties: &Feature<i32>) -> Vec<Tag> {
    properties
        .properties
        .as_ref()
        .unwrap()
        .iter()
        .map(|(key, value)| Tag {
            key: key.clone(),
            val: match value {
                Value::String(s) => s.clone(),
                Value::Null => ("null").to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Float(f) => f.to_string(),
                Value::Double(d) => d.to_string(),
                Value::Int(i) | Value::SInt(i) => i.to_string(),
                Value::UInt(i) => i.to_string(),
            },
        })
        .collect::<Vec<Tag>>()
}

fn process_layer(reader: &Reader, layer: &Layer) -> Result<Vec<PolygonInstruction>, ParserError> {
    let mut polygons = Vec::new();
    let features = reader.get_features_as::<i32>(layer.layer_index)?;
    let layer_name = OMTLayer::from_name(&layer.name);

    for feature in features {
        let tags = get_tags(&feature);

        match &feature.geometry {
            Geometry::LineString(line_string) => {
                polygons.push((
                    tags.clone(),
                    layer_name.clone(),
                    line_string
                        .into_iter()
                        .map(|x| point(x.x as f32, x.y as f32) / 4096.)
                        .collect(),
                ));
            }
            Geometry::MultiLineString(multi_line_string) => {
                for polygon in multi_line_string {
                    polygons.push((
                        tags.clone(),
                        layer_name.clone(),
                        polygon
                            .into_iter()
                            .map(|x| point(x.x as f32, x.y as f32) / 4096.)
                            .collect(),
                    ));
                }
            }
            Geometry::MultiPolygon(multi_polygon) => {
                for polygon in multi_polygon {
                    polygons.push((
                        tags.clone(),
                        layer_name.clone(),
                        polygon
                            .exterior()
                            .into_iter()
                            .map(|x| point(x.x as f32, x.y as f32) / 4096.)
                            .collect(),
                    ));
                }
            }
            Geometry::MultiPoint(_) => {
                // println!("Not implemented multipoint: {layer:?} {feature:?}");
            }
            _ => {
                panic!("Not implemented: {feature:?}");
            }
        }
    }
    Ok(polygons)
}
