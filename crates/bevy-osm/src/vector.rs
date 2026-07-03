use crate::schema::layer::OMTLayer;
use crate::tag::Tag;
use bevy::prelude::*;
use geo::Coord;
use geo_types::Geometry;
use lyon::geom::euclid::{Point2D, UnknownUnit};
use lyon::math::point;
use mvt_reader::feature::Feature;
use mvt_reader::layer::Layer;
use mvt_reader::{Reader, error::ParserError, feature::Value};

pub type PolygonInstruction = (Vec<Tag>, OMTLayer, Vec<Point2D<f32, UnknownUnit>>);

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

#[inline]
fn transform_coord(coord: &Coord<i32>) -> Point2D<f32, UnknownUnit> {
    point(
        (coord.x as f32 - 2048.) / 4096.,
        (coord.y as f32 - 2048.) / 4096.,
    )
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
                    line_string.into_iter().map(transform_coord).collect(),
                ));
            }
            Geometry::MultiLineString(multi_line_string) => {
                for polygon in multi_line_string {
                    polygons.push((
                        tags.clone(),
                        layer_name.clone(),
                        polygon.into_iter().map(transform_coord).collect(),
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
                            .map(transform_coord)
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
