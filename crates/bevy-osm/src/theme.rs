use std::collections::HashMap;

use bevy::color::Color;
use bevy::log::error;
use osm_xml::Tag;

use crate::{mesh::BuildingInstruction, osm_types::BuildingClass};

use super::mesh::{BuildInstruction, FillInstruction, StrokeInstruction};

pub fn get_way_build_instruction(tags: &Vec<Tag>) -> BuildInstruction {
    let mut color = Color::BLACK;
    for tag in tags {
        if tag.key == "water" {
            color = Color::linear_rgb(0.0, 0.2, 0.9);
        }
        if tag.key == "bridge" {
            return BuildInstruction::None;
        }
        if tag.key == "natural" {
            match tag.val.as_str() {
                "water" => {
                    color = Color::linear_rgb(0.0, 0.2, 0.9);
                }
                "park" => {
                    color = Color::linear_rgb(0.0, 0.8, 0.4);
                }
                "sand" => {
                    color = Color::linear_rgb(0.8, 0.8, 0.0);
                }
                "wood" => {
                    color = Color::linear_rgb(0.0, 0.8, 0.0);
                }
                "coastline" => {
                    return BuildInstruction::None;
                }
                _ => {
                    // println!("{:?}", tag);
                    return BuildInstruction::None;
                }
            }
            return BuildInstruction::Fill(FillInstruction { color });
        }
        if tag.key == "location" && tag.val == "underground" {
            return BuildInstruction::None;
        }
        if tag.key == "highway" {
            match tag.val.as_str() {
                "cycleway" => {
                    color = Color::linear_rgb(0.8, 0.2, 0.2);
                }
                "service" => {
                    color = Color::linear_rgb(0.3, 0.3, 0.3);
                }
                "residential" | "tertiary" => {
                    color = Color::linear_rgb(0.3, 0.3, 0.3);
                    return BuildInstruction::Stroke(StrokeInstruction { color, width: 0.04 });
                }
                "motorway" | "motorway_link" => {
                    color = Color::linear_rgb(0.3, 0.3, 0.3);
                    return BuildInstruction::Stroke(StrokeInstruction { color, width: 0.06 });
                }
                "pedestrian" => {
                    color = Color::linear_rgb(0.3, 0.3, 0.4);
                    return BuildInstruction::Stroke(StrokeInstruction { color, width: 0.04 });
                }
                "footway" | "path" | "steps" => {
                    color = Color::linear_rgb(0.3, 0.3, 0.4);
                    return BuildInstruction::Stroke(StrokeInstruction {
                        color,
                        width: 0.015,
                    });
                }
                "unclassified" | "construction" => {
                    color = Color::linear_rgb(0.0, 0.0, 0.0);
                    return BuildInstruction::Stroke(StrokeInstruction { color, width: 0.04 });
                }
                _ => {
                    // println!("{:?}", tag);
                }
            };
            return BuildInstruction::Stroke(StrokeInstruction { color, width: 0.04 });
        }
        if tag.key == "railway" {
            color = Color::linear_rgb(0.4, 0.4, 0.4);
            return BuildInstruction::Stroke(StrokeInstruction { color, width: 0.05 });
        }
        if tag.key == "bridge" {
            color = Color::linear_rgb(0.5, 0.5, 0.5);
            return BuildInstruction::Stroke(StrokeInstruction { color, width: 0.05 });
        }
        if tag.key == "leisure" && (tag.val == "park" || tag.val == "garden" || tag.val == "meadow")
        {
            color = Color::linear_rgb(0.0, 1.0, 0.0);
            return BuildInstruction::Fill(FillInstruction { color });
        }
        if tag.key == "landuse" {
            if tag.val == "forest" || tag.val == "meadow" || tag.val == "grass" {
                color = Color::linear_rgb(0.0, 1.0, 0.0);
                return BuildInstruction::Fill(FillInstruction { color });
            }
            return BuildInstruction::None;
        }
        // if (tag.key == "amenity" && tag.val == "parking_space")
        //     || (tag.key == "parking" && tag.val == "surface")
        // {
        //     color = Color::linear_rgb(0.0, 0.0, 0.0);
        //     return BuildInstruction::Fill(FillInstruction { color });
        // }
        if tag.key == "aeroway" {
            match tag.val.as_str() {
                "jet_bridge" => {
                    color = Color::linear_rgb(0.0, 0.0, 0.0);
                    return BuildInstruction::Stroke(StrokeInstruction { color, width: 0.05 });
                }
                "runway" => {
                    color = Color::linear_rgb(0.0, 0.0, 0.0);
                    return BuildInstruction::Stroke(StrokeInstruction { color, width: 0.8 });
                }
                "taxiway" | "taxilane" | "holding_position" | "stopway" | "parking_position" => {
                    color = Color::linear_rgb(0.5, 0.5, 0.0);
                    return BuildInstruction::Stroke(StrokeInstruction { color, width: 0.01 });
                }
                _ => {}
            }

            return BuildInstruction::None;
        }
        if tag.key == "building" {
            let tag_map = tags
                .iter()
                .map(|tag| (tag.key.clone(), tag.val.clone()))
                .collect::<HashMap<String, String>>();

            return BuildInstruction::Building(BuildingInstruction {
                class: Some(BuildingClass::from_string(&tag.val)),
                height: tag_map.get("building:height").map(|h| h.parse().unwrap()),
                levels: tag_map.get("building:levels").map(|h| {
                    h.parse().unwrap_or_else(|_| {
                        error!("Could not parse building levels: {}", h);
                        1.0
                    })
                }),
            });
        }
    }
    BuildInstruction::None
}

#[expect(dead_code, unused_variables)]
pub fn get_rel_build_instruction(tags: &Vec<Tag>) -> BuildInstruction {
    // TODO: handle relations that are only partially included in a dataset
    return BuildInstruction::None;
    #[expect(unreachable_code)]
    let mut color = Color::WHITE;

    for tag in tags {
        if tag.key == "ocean" && tag.val == "yes" {
            color = Color::linear_rgb(0.0, 0.2, 0.9);
            return BuildInstruction::Fill(FillInstruction { color });
        }
        if tag.key == "place" && tag.val == "island" {
            color = Color::linear_rgb(1.0, 1.0, 0.9);
            return BuildInstruction::Fill(FillInstruction { color });
        }
        if tag.key == "natural" {
            match tag.val.as_str() {
                "water" | "bay" => {
                    color = Color::linear_rgb(0.0, 0.2, 0.9);
                }
                _ => {
                    return BuildInstruction::None;
                }
            }
            return BuildInstruction::Fill(FillInstruction { color });
        }
    }
    BuildInstruction::None
}
