use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Water {
    Dock,
    Lake,
    Ocean,
    Pond,
    River,
    Swimmingpool,
}
