use std::collections::HashMap;

use bevy::{color::Color, log::warn};

use crate::{
    mesh::{BuildingInstruction, Layer},
    osm_types::BuildingClass,
    schema::{
        LayerClass, aeroway::Aeroway, landcover::Landcover, layer::OMTLayer, parse_class,
        transportation::Transportation, waterway::Waterway,
    },
    tag::Tag,
};

use super::mesh::{BuildInstruction, FillInstruction, StrokeInstruction};

pub fn get_way_build_instruction_openfreemap(
    tags: Vec<Tag>,
    layer_name: OMTLayer,
) -> BuildInstruction {
    if let Some(tag) = tags.iter().find(|x| x.key == "class") {
        match parse_class(&layer_name, &tag.val) {
            LayerClass::Landcover(Landcover::Grass) => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.0, 0.8, 0.4),
                    layer: Layer::Background,
                });
            }
            LayerClass::Landcover(Landcover::Wetland) => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.0, 0.3, 1.0),
                    layer: Layer::Background,
                });
            }
            LayerClass::Waterway(Waterway::Ocean) => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.0, 0.0, 1.0),
                    layer: Layer::Foreground,
                });
            }
            LayerClass::Waterway(_) => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.0, 0.2, 0.9),
                    layer: Layer::Foreground,
                });
            }
            LayerClass::Water(_) => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.0, 0.2, 0.9),
                    layer: Layer::Foreground,
                });
            }
            LayerClass::Landcover(Landcover::Ice) => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(1.0, 1.0, 1.0),
                    layer: Layer::Foreground,
                });
            }
            LayerClass::Landcover(Landcover::Sand) => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.8, 0.8, 0.0),
                    layer: Layer::Foreground,
                });
            }
            LayerClass::Landcover(Landcover::Wood) => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.0, 1., 0.0),
                    layer: Layer::Foreground,
                });
            }
            LayerClass::Landcover(Landcover::Farmland) => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.5, 0.8, 0.4),
                    layer: Layer::Foreground,
                });
            }
            LayerClass::Landcover(Landcover::Rock) => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.8, 0.8, 0.0),
                    layer: Layer::Foreground,
                });
            }
            LayerClass::Transportation(Transportation::Rail) => {
                return BuildInstruction::Stroke(StrokeInstruction {
                    color: Color::linear_rgb(0.3, 0.3, 0.3),
                    width: 6.0 / 4096.,
                });
            }
            LayerClass::Transportation(
                Transportation::Primary
                | Transportation::PrimaryConstruction
                | Transportation::Raceway
                | Transportation::RacewayConstruction,
            ) => {
                return BuildInstruction::Stroke(StrokeInstruction {
                    color: Color::linear_rgb(0.3, 0.3, 0.3),
                    width: 7.0 / 4096.,
                });
            }
            LayerClass::Transportation(
                Transportation::Secondary | Transportation::SecondaryConstruction,
            ) => {
                return BuildInstruction::Stroke(StrokeInstruction {
                    color: Color::linear_rgb(0.3, 0.3, 0.3),
                    width: 4.0 / 4096.,
                });
            }
            LayerClass::Transportation(Transportation::Busway | Transportation::Busguideway) => {
                return BuildInstruction::Stroke(StrokeInstruction {
                    color: Color::linear_rgb(1.0, 0.2, 0.2),
                    width: 5.0 / 4096.,
                });
            }
            LayerClass::Transportation(
                Transportation::Motorway | Transportation::MotorwayConstruction,
            ) => {
                return BuildInstruction::Stroke(StrokeInstruction {
                    color: Color::BLACK,
                    width: 10.0 / 4096.,
                });
            }
            LayerClass::Transportation(
                Transportation::Minor | Transportation::MinorConstruction,
            ) => {
                return BuildInstruction::Stroke(StrokeInstruction {
                    color: Color::linear_rgb(0.3, 0.3, 0.3),
                    width: 3.0 / 4096.,
                });
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
                return BuildInstruction::None;
            }
            LayerClass::Transportation(Transportation::Ferry) => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.0, 0.0, 1.0),
                    layer: Layer::OnTop,
                });
            }
            LayerClass::Aeroway(Aeroway::Runway) => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::BLACK,
                    layer: Layer::OnTop,
                });
            }
            LayerClass::Aeroway(
                Aeroway::Taxiway | Aeroway::Apron | Aeroway::Helipad | Aeroway::Gate,
            )
            | LayerClass::Transportation(Transportation::Aerialway) => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.3, 0.3, 0.3),
                    layer: Layer::Foreground,
                });
            }
            LayerClass::Aeroway(Aeroway::Aerodrome | Aeroway::Heliport) => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(0.1, 0.1, 0.1),
                    layer: Layer::Background,
                });
            }
            LayerClass::Place => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(1.0, 0.2, 0.2),
                    layer: Layer::Foreground,
                });
            }
            LayerClass::Landuse(_) => {
                return BuildInstruction::Fill(FillInstruction {
                    color: Color::linear_rgb(42. / 255., 35. / 255., 35. / 255.),
                    layer: Layer::Background,
                });
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
                return BuildInstruction::None;
            }
            LayerClass::Unknown => {
                warn!("Unknown class found [{layer_name:?}]: {:?}", tag.val);
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
