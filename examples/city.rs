use mvt_reader::{Reader, error::ParserError};

fn main() -> Result<(), ParserError> {
    // Read a vector tile from file or data
    let data = include_bytes!("../assets/osm/169.pbf").into();
    let reader = Reader::new(data)?;

    // Get layer names
    let layer_names = reader.get_layer_metadata()?;
    for name in layer_names {
        println!("Layer: {:?}", name);
    }

    // Get features for a specific layer
    let layer_index = 1;
    let features = reader.get_features(layer_index)?;
    for feature in features {
        println!("feature: {:?}", feature.properties);
    }

    Ok(())
}
