use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    ecs::error::BevyError,
    log::debug,
    reflect::TypePath,
    tasks::ConditionalSendFuture,
};
use bevy_svg::prelude::Svg;

#[derive(Default, TypePath)]
pub struct SvgAssetLoader;

impl AssetLoader for SvgAssetLoader {
    type Asset = Svg;
    type Settings = ();
    type Error = BevyError;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            debug!("Parsing SVG: {} ...", load_context.path());
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await.expect("TODO");

            let mut svg =
                Svg::from_bytes(&bytes, load_context.path().path(), None::<&std::path::Path>)?;
            let name = &load_context
                .path()
                .path()
                .file_name()
                .expect("TODO")
                .to_string_lossy();
            svg.name = name.to_string();
            debug!("Parsing SVG: {} ... Done", load_context.path());

            debug!("Tessellating SVG: {} ...", load_context.path());
            let mesh = svg.tessellate();
            debug!("Tessellating SVG: {} ... Done", load_context.path());
            let mesh_handle = load_context.add_labeled_asset("mesh".to_string(), mesh);
            svg.mesh = mesh_handle;

            Ok(svg)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["svg", "svgz"]
    }
}
