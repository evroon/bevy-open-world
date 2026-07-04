use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
    reflect::TypePath,
};
use mvt_reader::error::ParserError;
use thiserror::Error;

use crate::vector::{PolygonInstruction, parse_pbf};

#[derive(Asset, TypePath, Debug)]
pub struct MVTAsset {
    pub instructions: Vec<PolygonInstruction>,
}

#[derive(Default, TypePath)]
pub struct MVTAssetLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum MVTAssetLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not parse PBF: {0}")]
    Parser(#[from] ParserError),
}

impl AssetLoader for MVTAssetLoader {
    type Asset = MVTAsset;
    type Settings = ();
    type Error = MVTAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        Ok(MVTAsset {
            instructions: parse_pbf(bytes)?,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["pbf"]
    }
}
