use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Landcover {
    Farmland,
    Grass,
    Ice,
    Rock,
    Sand,
    Wetland,
    Wood,
}
