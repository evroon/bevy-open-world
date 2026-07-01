use crate::schema::{
    aeroway::Aeroway, landcover::Landcover, landuse::Landuse, layer::OMTLayer, poi::Poi,
    transportation::Transportation, water::Water, waterway::Waterway,
};
use serde::{Deserialize, Serialize};

pub mod aeroway;
pub mod landcover;
pub mod landuse;
pub mod layer;
pub mod poi;
pub mod transportation;
pub mod water;
pub mod waterway;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayerClass {
    AerodromeLabel,
    Aeroway(Aeroway),
    Boundary,
    Building,
    Housenumber,
    Landcover(Landcover),
    Landuse(Landuse),
    MountainPeak,
    Park,
    Place,
    Poi(Poi),
    Transportation(Transportation),
    TransportationName,
    Water(Water),
    Waterway(Waterway),
    Unknown,
}

/// Parse a layer and class combination to categorize an object.
///
/// https://openmaptiles.org/schema
pub fn parse_class(layer: &OMTLayer, class: &str) -> LayerClass {
    match layer {
        OMTLayer::Aeroway => match class {
            "aerodrome" => LayerClass::Aeroway(Aeroway::Aerodrome),
            "apron" => LayerClass::Aeroway(Aeroway::Apron),
            "gate" => LayerClass::Aeroway(Aeroway::Gate),
            "helipad" => LayerClass::Aeroway(Aeroway::Helipad),
            "heliport" => LayerClass::Aeroway(Aeroway::Heliport),
            "runway" => LayerClass::Aeroway(Aeroway::Runway),
            "taxiway" => LayerClass::Aeroway(Aeroway::Taxiway),
            _ => LayerClass::Unknown,
        },
        OMTLayer::Landcover => match class {
            "farmland" => LayerClass::Landcover(Landcover::Farmland),
            "grass" => LayerClass::Landcover(Landcover::Grass),
            "ice" => LayerClass::Landcover(Landcover::Ice),
            "rock" => LayerClass::Landcover(Landcover::Rock),
            "sand" => LayerClass::Landcover(Landcover::Sand),
            "wetland" => LayerClass::Landcover(Landcover::Wetland),
            "wood" => LayerClass::Landcover(Landcover::Wood),
            _ => LayerClass::Unknown,
        },
        OMTLayer::Landuse => match class {
            "animal_keeping" => LayerClass::Landuse(Landuse::AnimalKeeping),
            "bus_station" => LayerClass::Landuse(Landuse::Busstation),
            "cemetery" => LayerClass::Landuse(Landuse::Cemetery),
            "college" => LayerClass::Landuse(Landuse::College),
            "churchyard" => LayerClass::Landuse(Landuse::Churchyard),
            "parking" => LayerClass::Landuse(Landuse::Parking),
            "community_centre" => LayerClass::Landuse(Landuse::CommunityCentre),
            "recreation_ground" => LayerClass::Landuse(Landuse::RecreationGround),
            "grass" => LayerClass::Landuse(Landuse::Grass),
            "construction" => LayerClass::Landuse(Landuse::Construction),
            "commercial" => LayerClass::Landuse(Landuse::Commercial),
            "dam" => LayerClass::Landuse(Landuse::Dam),
            "education" => LayerClass::Landuse(Landuse::Education),
            "farmyard" => LayerClass::Landuse(Landuse::Farmyard),
            "garages" => LayerClass::Landuse(Landuse::Garages),
            "healthcare" => LayerClass::Landuse(Landuse::Healthcare),
            "hospital" => LayerClass::Landuse(Landuse::Hospital),
            "industrial" => LayerClass::Landuse(Landuse::Industrial),
            "kindergarten" => LayerClass::Landuse(Landuse::Kindergarten),
            "library" => LayerClass::Landuse(Landuse::Library),
            "military" => LayerClass::Landuse(Landuse::Military),
            "neighbourhood" => LayerClass::Landuse(Landuse::Neighbourhood),
            "park" => LayerClass::Landuse(Landuse::Park),
            "pitch" => LayerClass::Landuse(Landuse::Pitch),
            "playground" => LayerClass::Landuse(Landuse::Playground),
            "quarry" => LayerClass::Landuse(Landuse::Quarry),
            "quarter" => LayerClass::Landuse(Landuse::Quarter),
            "railway" => LayerClass::Landuse(Landuse::Railway),
            "religious" => LayerClass::Landuse(Landuse::Religous),
            "residential" => LayerClass::Landuse(Landuse::Residential),
            "retail" => LayerClass::Landuse(Landuse::Retail),
            "school" => LayerClass::Landuse(Landuse::School),
            "stadium" => LayerClass::Landuse(Landuse::Stadium),
            "suburb" => LayerClass::Landuse(Landuse::Suburb),
            "theme_park" => LayerClass::Landuse(Landuse::Themepark),
            "track" => LayerClass::Landuse(Landuse::Track),
            "university" => LayerClass::Landuse(Landuse::University),
            "zoo" => LayerClass::Landuse(Landuse::Zoo),
            _ => LayerClass::Unknown,
        },
        OMTLayer::Poi => match class {
            "aerialway" => LayerClass::Poi(Poi::Aerialway),
            "alcohol_shop" => LayerClass::Poi(Poi::Alcoholshop),
            "art_gallery" => LayerClass::Poi(Poi::Artgallery),
            "atm" => LayerClass::Poi(Poi::Atm),
            "attraction" => LayerClass::Poi(Poi::Attraction),
            "bar" => LayerClass::Poi(Poi::Bar),
            "beer" => LayerClass::Poi(Poi::Beer),
            "bus" => LayerClass::Poi(Poi::Bus),
            "cafe" => LayerClass::Poi(Poi::Cafe),
            "campsite" => LayerClass::Poi(Poi::Campsite),
            "car" => LayerClass::Poi(Poi::Car),
            "castle" => LayerClass::Poi(Poi::Castle),
            "cemetery" => LayerClass::Poi(Poi::Cemetery),
            "clothing_store" => LayerClass::Poi(Poi::Clothingstore),
            "college" => LayerClass::Poi(Poi::College),
            "entrance" => LayerClass::Poi(Poi::Entrance),
            "fast_food" => LayerClass::Poi(Poi::Fastfood),
            "fuel" => LayerClass::Poi(Poi::Fuel),
            "golf" => LayerClass::Poi(Poi::Golf),
            "grocery" => LayerClass::Poi(Poi::Grocery),
            "harbor" => LayerClass::Poi(Poi::Harbor),
            "hospital" => LayerClass::Poi(Poi::Hospital),
            "ice_cream" => LayerClass::Poi(Poi::Icecream),
            "laundry" => LayerClass::Poi(Poi::Laundry),
            "library" => LayerClass::Poi(Poi::Library),
            "lodging" => LayerClass::Poi(Poi::Lodging),
            "music" => LayerClass::Poi(Poi::Music),
            "office" => LayerClass::Poi(Poi::Office),
            "park" => LayerClass::Poi(Poi::Park),
            "railway" => LayerClass::Poi(Poi::Railway),
            "school" => LayerClass::Poi(Poi::School),
            "shop" => LayerClass::Poi(Poi::Shop),
            "stadium" => LayerClass::Poi(Poi::Stadium),
            "swimming" => LayerClass::Poi(Poi::Swimming),
            "town_hall" => LayerClass::Poi(Poi::Townhall),
            _ => LayerClass::Unknown,
        },
        OMTLayer::Transportation => match class {
            "aerialway" => LayerClass::Transportation(Transportation::Aerialway),
            "transit" => LayerClass::Transportation(Transportation::Transit),
            "bus_guideway" => LayerClass::Transportation(Transportation::Busguideway),
            "busway" => LayerClass::Transportation(Transportation::Busway),
            "ferry" => LayerClass::Transportation(Transportation::Ferry),
            "pier" => LayerClass::Transportation(Transportation::Pier),
            "bridge" => LayerClass::Transportation(Transportation::Bridge),
            "minor" => LayerClass::Transportation(Transportation::Minor),
            "motorway" => LayerClass::Transportation(Transportation::Motorway),
            "path" => LayerClass::Transportation(Transportation::Path),
            "primary" => LayerClass::Transportation(Transportation::Primary),
            "raceway" => LayerClass::Transportation(Transportation::Raceway),
            "secondary" => LayerClass::Transportation(Transportation::Secondary),
            "service" => LayerClass::Transportation(Transportation::Service),
            "tertiary" => LayerClass::Transportation(Transportation::Tertiary),
            "track" => LayerClass::Transportation(Transportation::Track),
            "trunk" => LayerClass::Transportation(Transportation::Trunk),
            "rail" => LayerClass::Transportation(Transportation::Rail),
            "motorway_construction" => {
                LayerClass::Transportation(Transportation::MotorwayConstruction)
            }
            "trunk_construction" => LayerClass::Transportation(Transportation::TrunkConstruction),
            "primary_construction" => {
                LayerClass::Transportation(Transportation::PrimaryConstruction)
            }
            "secondary_construction" => {
                LayerClass::Transportation(Transportation::SecondaryConstruction)
            }
            "tertiary_construction" => {
                LayerClass::Transportation(Transportation::TertiaryConstruction)
            }
            "minor_construction" => LayerClass::Transportation(Transportation::MinorConstruction),
            "path_construction" => LayerClass::Transportation(Transportation::PathConstruction),
            "service_construction" => {
                LayerClass::Transportation(Transportation::ServiceConstruction)
            }
            "track_construction" => LayerClass::Transportation(Transportation::TrackConstruction),
            "raceway_construction" => {
                LayerClass::Transportation(Transportation::RacewayConstruction)
            }
            _ => LayerClass::Unknown,
        },
        OMTLayer::Water => match class {
            "dock" => LayerClass::Water(Water::Dock),
            "lake" => LayerClass::Water(Water::Lake),
            "ocean" => LayerClass::Water(Water::Ocean),
            "pond" => LayerClass::Water(Water::Pond),
            "river" => LayerClass::Water(Water::River),
            "swimming_pool" => LayerClass::Water(Water::Swimmingpool),
            _ => LayerClass::Unknown,
        },
        OMTLayer::Waterway => match class {
            "bay" => LayerClass::Waterway(Waterway::Bay),
            "ditch" => LayerClass::Waterway(Waterway::Ditch),
            "canal" => LayerClass::Waterway(Waterway::Canal),
            "drain" => LayerClass::Waterway(Waterway::Drain),
            "lake" => LayerClass::Waterway(Waterway::Lake),
            "ocean" => LayerClass::Waterway(Waterway::Ocean),
            "river" => LayerClass::Waterway(Waterway::River),
            "sea" => LayerClass::Waterway(Waterway::Sea),
            "strait" => LayerClass::Waterway(Waterway::Strait),
            "stream" => LayerClass::Waterway(Waterway::Stream),
            _ => LayerClass::Unknown,
        },
        OMTLayer::Place => LayerClass::Place,
        OMTLayer::MountainPeak => LayerClass::MountainPeak,
        OMTLayer::TransportationName => LayerClass::TransportationName,
        OMTLayer::Boundary => LayerClass::Boundary,
        OMTLayer::Building => LayerClass::Building,
        OMTLayer::AerodromeLabel => LayerClass::AerodromeLabel,
        OMTLayer::Park => LayerClass::Park,
        _ => LayerClass::Unknown,
    }
}
