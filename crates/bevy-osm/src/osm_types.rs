use bevy::color::Color;
use bevy::color::palettes::css::*;
use strum_macros::EnumIter;

/// https://wiki.openstreetmap.org/wiki/Buildings#Building
#[derive(EnumIter, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BuildingClass {
    Residential,
    Outbuilding,
    Agricultural,
    Commercial,
    Industrial,
    Education,
    Service,
    Religious,
    Civic,
    Transportation,
    Medical,
    Entertainment,
    Military,
}
impl BuildingClass {
    pub fn from_string(s: &str) -> BuildingClass {
        match s {
            "residential" => BuildingClass::Residential,
            "outbuilding" => BuildingClass::Outbuilding,
            "agricultural" => BuildingClass::Agricultural,
            "commercial" => BuildingClass::Commercial,
            "industrial" => BuildingClass::Industrial,
            "education" => BuildingClass::Education,
            "service" => BuildingClass::Service,
            "religious" => BuildingClass::Religious,
            "civic" => BuildingClass::Civic,
            "transportation" => BuildingClass::Transportation,
            "medical" => BuildingClass::Medical,
            "entertainment" => BuildingClass::Entertainment,
            "military" => BuildingClass::Military,
            _ => BuildingClass::Residential,
        }
    }
}

impl From<&BuildingClass> for Color {
    fn from(building_class: &BuildingClass) -> Self {
        match building_class {
            BuildingClass::Residential => Color::linear_rgb(0.5, 0.45, 0.4),
            BuildingClass::Outbuilding => DARK_GRAY.into(),
            BuildingClass::Agricultural => GREEN.into(),
            BuildingClass::Commercial => Color::linear_rgb(0.3, 0.3, 0.4),
            BuildingClass::Industrial => SILVER.into(),
            BuildingClass::Education => ANTIQUE_WHITE.into(),
            BuildingClass::Service => BISQUE.into(),
            BuildingClass::Religious => AQUAMARINE.into(),
            BuildingClass::Civic => Color::linear_rgb(0.6, 0.6, 0.8),
            BuildingClass::Transportation => PURPLE.into(),
            BuildingClass::Medical => ORANGE_RED.into(),
            BuildingClass::Entertainment => AZURE.into(),
            BuildingClass::Military => NAVY.into(),
        }
    }
}
