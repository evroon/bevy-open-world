#[derive(Debug, PartialEq, Clone)]
pub enum OMTLayer {
    AerodromeLabel,
    Aeroway,
    Boundary,
    Building,
    Housenumber,
    Landcover,
    Landuse,
    MountainPeak,
    Park,
    Place,
    Poi,
    Transportation,
    TransportationName,
    Water,
    WaterName,
    Waterway,
}

impl OMTLayer {
    pub fn from_name(key: &str) -> OMTLayer {
        match key {
            "aerodrome_label" => OMTLayer::AerodromeLabel,
            "aeroway" => OMTLayer::Aeroway,
            "boundary" => OMTLayer::Boundary,
            "building" => OMTLayer::Building,
            "housenumber" => OMTLayer::Housenumber,
            "landcover" => OMTLayer::Landcover,
            "landuse" => OMTLayer::Landuse,
            "mountain_peak" => OMTLayer::MountainPeak,
            "park" => OMTLayer::Park,
            "place" => OMTLayer::Place,
            "poi" => OMTLayer::Poi,
            "transportation" => OMTLayer::Transportation,
            "transportation_name" => OMTLayer::TransportationName,
            "water" => OMTLayer::Water,
            "water_name" => OMTLayer::WaterName,
            "waterway" => OMTLayer::Waterway,
            _ => {
                panic!("Could not parse OMTLayer: {key}")
            }
        }
    }
}
