use std::collections::HashMap;

use bevy::{
    color::Color,
    log::{debug, warn},
};

use crate::{
    layer::OMTLayer,
    mesh::{BuildingInstruction, Layer},
    osm_types::BuildingClass,
    tag::Tag,
};

use super::mesh::{BuildInstruction, FillInstruction, StrokeInstruction};

pub fn get_way_build_instruction_openfreemap(
    tags: Vec<Tag>,
    layer_name: OMTLayer,
) -> BuildInstruction {
    if let Some(tag) = tags.iter().find(|x| x.key == "class") {
        match tag.val.as_str() {
            "grass" | "meadow" => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.0, 0.8, 0.4),
                    layer: Layer::Background,
                });
            }
            "wetland" => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.0, 0.3, 1.0),
                    layer: Layer::Background,
                });
            }
            "surface" => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::WHITE,
                    layer: Layer::Background,
                });
            }
            "ocean" => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.0, 0.0, 1.0),
                    layer: Layer::Foreground,
                });
            }
            "water" | "lake" | "pond" => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.0, 0.2, 0.9),
                    layer: Layer::Foreground,
                });
            }
            "park" => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.0, 0.8, 0.4),
                    layer: Layer::Foreground,
                });
            }
            "sand" => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.8, 0.8, 0.0),
                    layer: Layer::Foreground,
                });
            }
            "wood" => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.0, 1., 0.0),
                    layer: Layer::Foreground,
                });
            }
            "farmland" => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.5, 0.8, 0.4),
                    layer: Layer::Foreground,
                });
            }
            "river" | "ditch" | "stream" | "swimming_pool" | "drain" => {
                if layer_name != OMTLayer::Waterway {
                    return BuildInstruction::Fill(FillInstruction {
                        color: Color::linear_rgb(0.0, 0.2, 0.9),
                        layer: Layer::Foreground,
                    });
                } else {
                    return BuildInstruction::None;
                }
            }
            "rail" => {
                return BuildInstruction::Stroke(StrokeInstruction {
                    color: Color::linear_rgb(0.3, 0.3, 0.3),
                    width: 6.0 / 4096.,
                });
            }
            "primary" | "raceway" => {
                return BuildInstruction::Stroke(StrokeInstruction {
                    color: Color::linear_rgb(0.3, 0.3, 0.3),
                    width: 7.0 / 4096.,
                });
            }
            "runway" => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::BLACK,
                    layer: Layer::OnTop,
                });
            }
            "taxiway" | "apron" | "helipad" => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.3, 0.3, 0.3),
                    layer: Layer::Foreground,
                });
            }
            "aerodrome" | "heliport" => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.1, 0.1, 0.1),
                    layer: Layer::Background,
                });
            }
            "playground" | "stadium" | "track" | "pitch" => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(1.0, 0.2, 0.2),
                    layer: Layer::Foreground,
                });
            }
            "secondary" | "railway" => {
                return BuildInstruction::Stroke(StrokeInstruction {
                    color: Color::linear_rgb(0.3, 0.3, 0.3),
                    width: 4.0 / 4096.,
                });
            }
            "busway" => {
                return BuildInstruction::Stroke(StrokeInstruction {
                    color: Color::linear_rgb(1.0, 0.2, 0.2),
                    width: 5.0 / 4096.,
                });
            }
            "motorway" | "motorway_link" => {
                return BuildInstruction::Stroke(StrokeInstruction {
                    color: Color::BLACK,
                    width: 10.0 / 4096.,
                });
            }
            "minor" => {
                return BuildInstruction::Stroke(StrokeInstruction {
                    color: Color::linear_rgb(0.3, 0.3, 0.3),
                    width: 3.0 / 4096.,
                });
            }
            "canal" => {
                // return BuildInstruction::Stroke(StrokeInstruction {
                //     color: Color::linear_rgb(1.0, 0.2, 0.2),
                //     width: 2.0/ 4096.,
                // });
                return BuildInstruction::None;
            }
            "tertiary" | "path" | "service" => {
                // return BuildInstruction::Stroke(StrokeInstruction {
                //     color: Color::linear_rgb(1.0, 0.2, 0.2),
                //     width: 2.0/ 4096.,
                // });
                return BuildInstruction::None;
            }
            "coastline" | "pier" | "bridge" | "trunk" => {
                return BuildInstruction::None;
            }
            "residential" => {
                return BuildInstruction::None;
            }
            "secondary_construction"
            | "tertiary_construction"
            | "minor_construction"
            | "path_construction" => {
                return BuildInstruction::None;
            }
            "rock" | "quarry" => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(42. / 255., 35. / 255., 35. / 255.),
                    layer: Layer::Background,
                });
            }
            "cemetery" | "college" | "hospital" | "kindergarten" | "library" | "school"
            | "university" | "zoo" | "ferry" => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(123. / 255., 55. / 255., 38. / 255.),
                    layer: Layer::Background,
                });
            }
            "cliff"
            | "commercial"
            | "education"
            | "industrial"
            | "military"
            | "nature_reserve"
            | "quarter"
            | "bus_station"
            | "neighbourhood"
            | "healthcare"
            | "suburb"
            | "ridge"
            | "aerialway"
            | "theme_park"
            | "naturentwicklungsgebiet"
            | "naturschutzgebiet"
            | "protected_area"
            | "retail"
            | "totalreservat" => {
                return BuildInstruction::None;
            }
            _ => {
                debug!("Unknown class found: {:?}", tags);
                return BuildInstruction::None;
            }
        }
    }

    match layer_name {
        OMTLayer::Building => {
            let tag_map = tags
                .iter()
                .map(|tag| (tag.key.clone(), tag.val.clone()))
                .collect::<HashMap<String, String>>();

            BuildInstruction::Building(BuildingInstruction {
                class: Some(BuildingClass::Agricultural),
                height: None,
                levels: tag_map
                    .get("render_height")
                    .and_then(|x| x.parse::<i32>().ok().map(|x| x.max(5) as f32)),
            })
        }
        _ => BuildInstruction::None,
    }
}

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
                    return BuildInstruction::None;
                }
            }
            return BuildInstruction::Fill(FillInstruction {
                color,
                layer: Layer::Background,
            });
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
            return BuildInstruction::Fill(FillInstruction {
                color,
                layer: Layer::Background,
            });
        }
        if tag.key == "landuse" {
            if tag.val == "forest" || tag.val == "meadow" || tag.val == "grass" {
                color = Color::linear_rgb(0.0, 1.0, 0.0);
                return BuildInstruction::Fill(FillInstruction {
                    color,
                    layer: Layer::Background,
                });
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
                        warn!("Could not parse building levels: {}", h);
                        1.0
                    })
                }),
            });
        }
    }
    BuildInstruction::None
}

pub fn get_rel_build_instruction(_tags: &Vec<Tag>) -> BuildInstruction {
    // TODO: handle relations that are only partially included in a dataset
    return BuildInstruction::None;
    #[expect(unreachable_code)]
    let mut color = Color::WHITE;

    for tag in _tags {
        if tag.key == "ocean" && tag.val == "yes" {
            color = Color::linear_rgb(0.0, 0.2, 0.9);
            return BuildInstruction::Fill(FillInstruction {
                color,
                layer: Layer::Background,
            });
        }
        if tag.key == "place" && tag.val == "island" {
            color = Color::linear_rgb(1.0, 1.0, 0.9);
            return BuildInstruction::Fill(FillInstruction {
                color,
                layer: Layer::Background,
            });
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
            return BuildInstruction::Fill(FillInstruction {
                color,
                layer: Layer::Background,
            });
        }
    }
    BuildInstruction::None
}
