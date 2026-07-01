use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Waterway {
    Bay,
    Canal,
    Drain,
    Ditch,
    Lake,
    Ocean,
    River,
    Sea,
    Strait,
    Stream,
}
