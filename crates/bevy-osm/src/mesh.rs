use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use lyon::{math::Point, path::Path};
use lyon_tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, FillVertexConstructor, StrokeOptions,
    StrokeTessellator, StrokeVertex, StrokeVertexConstructor,
};

use crate::osm_types::BuildingClass;

type IndexType = u32;
/// A vertex with all the necessary attributes to be inserted into a Bevy
/// [`Mesh`](bevy::render::mesh::Mesh).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}
pub type VertexBuffers = lyon_tessellation::VertexBuffers<Vertex, IndexType>;
/// Zero-sized type used to implement various vertex construction traits from
/// Lyon.
pub struct VertexConstructor {
    pub color: Color,
}

/// Enables the construction of a [`Vertex`] when using a `FillTessellator`.
impl FillVertexConstructor<Vertex> for VertexConstructor {
    fn new_vertex(&mut self, vertex: FillVertex) -> Vertex {
        Vertex {
            position: [vertex.position().x, vertex.position().y],
            color: self.color.to_linear().to_f32_array(),
        }
    }
}

/// Enables the construction of a [`Vertex`] when using a `StrokeTessellator`.
impl StrokeVertexConstructor<Vertex> for VertexConstructor {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> Vertex {
        Vertex {
            position: [vertex.position().x, vertex.position().y],
            color: self.color.to_linear().to_f32_array(),
        }
    }
}

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
pub struct Shape;

pub struct StrokeInstruction {
    pub color: Color,
    pub width: f32,
}
pub struct BuildingInstruction {
    pub class: Option<BuildingClass>,
    pub height: Option<f32>,
    pub levels: Option<f32>,
}

pub struct FillInstruction {
    pub color: Color,
}

pub enum BuildInstruction {
    Fill(FillInstruction),
    Stroke(StrokeInstruction),
    Building(BuildingInstruction),
    None,
}

pub fn spawn_stroke_mesh(points: Vec<Point>, instruction: StrokeInstruction) -> Mesh {
    let mut path_builder = Path::builder();
    path_builder.begin(points[0]);
    for p in points[1..].iter() {
        path_builder.line_to(*p);
    }
    path_builder.end(false);

    let mut buffers = VertexBuffers::new();
    let mut tess = StrokeTessellator::new();
    let constructor = VertexConstructor {
        color: instruction.color,
    };

    if let Err(e) = tess.tessellate_path(
        &path_builder.build(),
        &StrokeOptions::default().with_line_width(instruction.width),
        &mut BuffersBuilder::new(&mut buffers, constructor),
    ) {
        error!("StrokeTessellator error: {:?}", e);
    }

    build_mesh(&buffers)
}

pub fn spawn_fill_mesh(points: Vec<Point>, instruction: FillInstruction) -> Mesh {
    let mut path_builder = Path::builder();
    path_builder.begin(points[0]);
    for p in points[1..].iter() {
        path_builder.line_to(*p);
    }
    path_builder.end(false);

    let mut buffers = VertexBuffers::new();
    let mut tess = FillTessellator::new();
    let constructor = VertexConstructor {
        color: instruction.color,
    };

    if let Err(e) = tess.tessellate_path(
        &path_builder.build(),
        &FillOptions::default(),
        &mut BuffersBuilder::new(&mut buffers, constructor),
    ) {
        error!("FillTessellator error: {:?}", e);
    }

    build_mesh(&buffers)
}

pub fn build_mesh(buffers: &VertexBuffers) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_indices(Indices::U32(buffers.indices.clone()));
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        buffers
            .vertices
            .iter()
            .map(|v| [v.position[0], 0.0, v.position[1]])
            .collect::<Vec<[f32; 3]>>(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        buffers
            .vertices
            .iter()
            .map(|v| v.color)
            .collect::<Vec<[f32; 4]>>(),
    );

    mesh
}
