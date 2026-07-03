use std::collections::HashMap;

use bevy::{color::Color, image::Image, log::warn, math::Vec3};
use lyon::geom::euclid::{Point2D, UnknownUnit};

use crate::{
    load_data::get_elevation,
    mesh::{BuildingInstruction, Layer, LightColor, LightInstruction},
    osm_types::BuildingClass,
    schema::{
        LayerClass, aeroway::Aeroway, landcover::Landcover, layer::OMTLayer, parse_class,
        transportation::Transportation, waterway::Waterway,
    },
    tag::Tag,
};

use super::mesh::{BuildInstruction, FillInstruction, StrokeInstruction};

fn snap_to_heightmap(center: Point2D<f32, UnknownUnit>, heightmap: &Image) -> Vec3 {
    Vec3::new(
        center.x,
        get_elevation(Vec3::new(center.x, 0.0, center.y), heightmap).unwrap_or(0.0) + 2.0,
        center.y,
    )
}

pub fn get_way_build_instruction_openfreemap(
    tags: Vec<Tag>,
    layer_name: OMTLayer,
    polygon: Vec<Point2D<f32, UnknownUnit>>,
    heightmap: &Image,
) -> Vec<BuildInstruction> {
    if let Some(tag) = tags.iter().find(|x| x.key == "class") {
        match parse_class(&layer_name, &tag.val) {
            LayerClass::Landcover(Landcover::Grass) => {
                return vec![BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.0, 0.8, 0.4),
                    layer: Layer::Background,
                    polygon,
                })];
            }
            LayerClass::Landcover(Landcover::Wetland) => {
                return vec![BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.0, 0.3, 1.0),
                    layer: Layer::Background,
                    polygon,
                })];
            }
            LayerClass::Waterway(Waterway::Ocean) => {
                return vec![BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.0, 0.0, 1.0),
                    layer: Layer::Foreground,
                    polygon,
                })];
            }
            LayerClass::Waterway(_) => {
                return vec![BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.0, 0.2, 0.9),
                    layer: Layer::Foreground,
                    polygon,
                })];
            }
            LayerClass::Water(_) => {
                return vec![BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.0, 0.2, 0.9),
                    layer: Layer::Foreground,
                    polygon,
                })];
            }
            LayerClass::Landcover(Landcover::Ice) => {
                return vec![BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(1.0, 1.0, 1.0),
                    layer: Layer::Foreground,
                    polygon,
                })];
            }
            LayerClass::Landcover(Landcover::Sand) => {
                return vec![BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.8, 0.8, 0.0),
                    layer: Layer::Foreground,
                    polygon,
                })];
            }
            LayerClass::Landcover(Landcover::Wood) => {
                return vec![BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.0, 1., 0.0),
                    layer: Layer::Foreground,
                    polygon,
                })];
            }
            LayerClass::Landcover(Landcover::Farmland) => {
                return vec![BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.5, 0.8, 0.4),
                    layer: Layer::Foreground,
                    polygon,
                })];
            }
            LayerClass::Landcover(Landcover::Rock) => {
                return vec![BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.8, 0.8, 0.0),
                    layer: Layer::Foreground,
                    polygon,
                })];
            }
            LayerClass::Transportation(Transportation::Rail) => {
                return vec![
                    BuildInstruction::Light(LightInstruction {
                        color: LightColor::Yellow,
                        trans: snap_to_heightmap(polygon[0], heightmap),
                    }),
                    BuildInstruction::Stroke(StrokeInstruction {
                        color: Color::linear_rgb(0.3, 0.3, 0.3),
                        width: 6.0 / 4096.,
                        polygon,
                    }),
                ];
            }
            LayerClass::Transportation(
                Transportation::Primary
                | Transportation::PrimaryConstruction
                | Transportation::Raceway
                | Transportation::RacewayConstruction,
            ) => {
                return vec![
                    BuildInstruction::Light(LightInstruction {
                        color: LightColor::Yellow,
                        trans: snap_to_heightmap(polygon[0], heightmap),
                    }),
                    BuildInstruction::Stroke(StrokeInstruction {
                        color: Color::linear_rgb(0.3, 0.3, 0.3),
                        width: 7.0 / 4096.,
                        polygon,
                    }),
                ];
            }
            LayerClass::Transportation(
                Transportation::Secondary | Transportation::SecondaryConstruction,
            ) => {
                return vec![
                    BuildInstruction::Light(LightInstruction {
                        color: LightColor::Orange,
                        trans: snap_to_heightmap(polygon[0], heightmap),
                    }),
                    BuildInstruction::Stroke(StrokeInstruction {
                        color: Color::linear_rgb(0.3, 0.3, 0.3),
                        width: 4.0 / 4096.,
                        polygon,
                    }),
                ];
            }
            LayerClass::Transportation(Transportation::Busway | Transportation::Busguideway) => {
                return vec![
                    BuildInstruction::Light(LightInstruction {
                        color: LightColor::Yellow,
                        trans: snap_to_heightmap(polygon[0], heightmap),
                    }),
                    BuildInstruction::Stroke(StrokeInstruction {
                        color: Color::linear_rgb(1.0, 0.2, 0.2),
                        width: 5.0 / 4096.,
                        polygon,
                    }),
                ];
            }
            LayerClass::Transportation(
                Transportation::Motorway | Transportation::MotorwayConstruction,
            ) => {
                return vec![
                    BuildInstruction::Light(LightInstruction {
                        color: LightColor::White,
                        trans: snap_to_heightmap(polygon[0], heightmap),
                    }),
                    BuildInstruction::Stroke(StrokeInstruction {
                        color: Color::BLACK,
                        width: 10.0 / 4096.,
                        polygon,
                    }),
                ];
            }
            LayerClass::Transportation(
                Transportation::Minor | Transportation::MinorConstruction,
            ) => {
                return vec![
                    BuildInstruction::Light(LightInstruction {
                        color: LightColor::Orange,
                        trans: snap_to_heightmap(polygon[0], heightmap),
                    }),
                    BuildInstruction::Stroke(StrokeInstruction {
                        color: Color::linear_rgb(0.3, 0.3, 0.3),
                        width: 3.0 / 4096.,
                        polygon,
                    }),
                ];
            }
            LayerClass::Transportation(
                Transportation::Tertiary
                | Transportation::TertiaryConstruction
                | Transportation::Path
                | Transportation::PathConstruction
                | Transportation::Service
                | Transportation::ServiceConstruction
                | Transportation::Track
                | Transportation::TrackConstruction
                | Transportation::Trunk
                | Transportation::TrunkConstruction
                | Transportation::Pier
                | Transportation::Bridge
                | Transportation::Transit,
            ) => {
                return vec![BuildInstruction::None];
            }
            LayerClass::Transportation(Transportation::Ferry) => {
                return vec![BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.0, 0.0, 1.0),
                    layer: Layer::OnTop,
                    polygon,
                })];
            }
            LayerClass::Aeroway(Aeroway::Runway) => {
                return vec![
                    BuildInstruction::Light(LightInstruction {
                        color: LightColor::Red,
                        trans: snap_to_heightmap(polygon[0], heightmap),
                    }),
                    BuildInstruction::Fill(FillInstruction {
                        color: Color::BLACK,
                        layer: Layer::OnTop,
                        polygon,
                    }),
                ];
            }
            LayerClass::Aeroway(Aeroway::Taxiway) => {
                return vec![
                    BuildInstruction::Light(LightInstruction {
                        color: LightColor::Blue,
                        trans: snap_to_heightmap(polygon[0], heightmap),
                    }),
                    BuildInstruction::Fill(FillInstruction {
                        color: Color::linear_rgb(0.3, 0.3, 0.3),
                        layer: Layer::Foreground,
                        polygon,
                    }),
                ];
            }
            LayerClass::Aeroway(Aeroway::Apron | Aeroway::Helipad | Aeroway::Gate)
            | LayerClass::Transportation(Transportation::Aerialway) => {
                return vec![BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.3, 0.3, 0.3),
                    layer: Layer::Foreground,
                    polygon,
                })];
            }
            LayerClass::Aeroway(Aeroway::Aerodrome | Aeroway::Heliport) => {
                return vec![BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.1, 0.1, 0.1),
                    layer: Layer::Background,
                    polygon,
                })];
            }
            LayerClass::Place => {
                return vec![BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(1.0, 0.2, 0.2),
                    layer: Layer::Foreground,
                    polygon,
                })];
            }
            LayerClass::Landuse(_) => {
                return vec![BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(42. / 255., 35. / 255., 35. / 255.),
                    layer: Layer::Background,
                    polygon,
                })];
            }
            LayerClass::MountainPeak
            | LayerClass::TransportationName
            | LayerClass::WaterName
            | LayerClass::Boundary
            | LayerClass::Building
            | LayerClass::AerodromeLabel
            | LayerClass::Housenumber
            | LayerClass::Poi(_)
            | LayerClass::Park => {
                return vec![BuildInstruction::None];
            }
            LayerClass::Unknown => {
                warn!("Unknown class found [{layer_name:?}]: {:?}", tag.val);
                return vec![BuildInstruction::None];
            }
        }
    }

    match layer_name {
        OMTLayer::Building => {
            let tag_map = tags
                .iter()
                .map(|tag| (tag.key.clone(), tag.val.clone()))
                .collect::<HashMap<String, String>>();

            let render_min_height = tag_map
                .get("render_min_height")
                .and_then(|x| x.parse::<i32>().ok().map(|x| x.max(0) as f32))
                .unwrap_or(0.0);

            vec![BuildInstruction::Building(BuildingInstruction {
                class: Some(BuildingClass::Agricultural),
                height: tag_map
                    .get("render_height")
                    .and_then(|x| x.parse::<i32>().ok().map(|x| x.max(5) as f32))
                    .unwrap_or(5.0)
                    - render_min_height,
                offset_above_ground_level: render_min_height,
                polygon,
            })]
        }
        _ => vec![BuildInstruction::None],
    }
}
