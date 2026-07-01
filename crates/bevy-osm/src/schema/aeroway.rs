use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Aeroway {
    Aerodrome,
    Apron,
    Gate,
    Helipad,
    Heliport,
    Runway,
    Taxiway,
}
