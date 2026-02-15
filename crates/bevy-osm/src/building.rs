use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use geo::algorithm::TriangulateEarcut;
use geo::{LineString, Winding};
use geo_types::Polygon;
use lyon::geom::Point;
use rand::RngExt;
use rand::rngs::ThreadRng;
use std::f32::consts::FRAC_PI_2;
use std::ops::Sub;

use crate::mesh::BuildingInstruction;
use crate::osm_types::BuildingClass;

#[derive(Component, Debug)]
pub struct Building {
    pub _class: Option<BuildingClass>,
    pub translate: [f32; 2],
    pub height: f32,
    pub _levels: Option<f32>,
    pub line: Vec<[f32; 2]>,
    pub vertices: Vec<[f32; 3]>,
    pub triangle_indices: Vec<u32>,
}

pub fn polygon_building(
    building_instruction: &BuildingInstruction,
    polygon: Vec<Point<f32>>,
    rng: &mut ThreadRng,
) -> Building {
    let mut polygon = Polygon::new(
        LineString::from(
            polygon
                .iter()
                .rev()
                .map(|v| (v.x, v.y))
                .collect::<Vec<(f32, f32)>>(),
        ),
        vec![],
    );

    polygon.exterior_mut(|exterior| {
        if exterior.is_ccw() {
            exterior.make_cw_winding();
        }
    });

    let line: Vec<[f32; 2]> = polygon
        .exterior()
        .coords()
        .map(|c| [(c.x), (c.y)])
        .collect();

    let triangles = polygon.earcut_triangles_raw();

    let height: f32 = match building_instruction.height {
        Some(h) => h,
        None => match building_instruction.levels {
            Some(levels) => levels * 3.0,
            None => rng.random_range(6.0..12.5),
        },
    };
    let height = height / 40.0;
    Building {
        _class: building_instruction.class,
        translate: [0.0, 0.0],
        height,
        _levels: building_instruction.levels,
        line,
        vertices: triangles
            .vertices
            .chunks(2)
            .map(|i| [(i[0]), 0., (i[1])])
            .collect(),
        triangle_indices: triangles
            .triangle_indices
            .iter()
            .map(|i| *i as u32)
            .collect(),
    }
    // spawn_building(commands, meshes, &b, map_materials);
}

pub fn spawn_building(building: &Building) -> Vec<(Mesh, Transform)> {
    let mut result = Vec::new();
    let wall = Wall::new(&building.line, building.height);
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::from(wall.vertices),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::from(wall.normals),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::from(wall.uvs));
    mesh.insert_indices(Indices::U32(wall.indices));
    mesh.compute_normals();

    let translate: Vec3 = Vec3::new(building.translate[0], 0., building.translate[1]);
    let transform = Transform::from_translation(translate);
    result.push((mesh, transform));

    // ROOF
    let mut roof = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    let vertices: Vec<[f32; 3]> = building.vertices.iter().map(|v| v.map(|p| p)).collect();
    roof.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::from(vertices.clone()),
    );
    roof.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::from(
            building
                .vertices
                .iter()
                .map(|_| [0., 1., 0.] as [f32; 3])
                .collect::<Vec<[f32; 3]>>(),
        ),
    );
    let uvs: Vec<[f32; 2]> = vertices.clone().iter().map(|p| [p[0], p[2]]).collect();
    roof.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::from(uvs));
    let bs = building.triangle_indices.clone();
    roof.insert_indices(Indices::U32(bs));
    roof.invert_winding().unwrap();
    roof.compute_normals();

    let translation = transform.translation + Vec3::new(0., building.height, 0.);
    let transform: Transform = Transform::from_translation(translation);

    result.push((roof, transform));
    result
}

#[derive(Component, Debug)]
pub struct Wall {
    pub points: Vec<Vec3>,
    pub indices: Vec<u32>,
    pub norm: Vec<Vec3>,
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
}

impl Wall {
    pub fn empty() -> Self {
        Wall {
            points: vec![],
            indices: vec![],
            norm: vec![],
            vertices: vec![],
            normals: vec![],
            uvs: vec![],
        }
    }
    pub fn new(line: &[[f32; 2]], height: f32) -> Self {
        let mut wall = Wall::empty();
        wall.points = line
            .iter()
            .map(|pos| Vec3::new(pos[0], 0., pos[1]))
            .collect::<Vec<Vec3>>();

        let heightv: Vec3 = Vec3::Y * height;
        let material_lengh = 1.;
        let mut len: f32 = 0.;
        let points_len = wall.points.len();

        for (i, p) in wall.points.iter().enumerate() {
            // println!("{:?}", &point);
            let last: bool = i + 1 == points_len;
            let ix2: u32 = i as u32 * 4;
            if !last {
                let (i1, i2) = ([ix2, ix2 + 2, ix2 + 1], [ix2 + 2, ix2 + 3, ix2 + 1]); // Yto-Z
                wall.indices.extend(i1);
                wall.indices.extend(i2);
                let point_next = wall.points[i + 1];
                let dir: Vec3 = (point_next - *p).normalize();
                // println!("{:?}", &dir);
                let norm = Quat::from_rotation_y(-FRAC_PI_2).mul_vec3(dir); // Yto-Z
                wall.norm.push(norm);
                let i_next: usize = if last { 0 } else { i + 1 };
                let point: Vec3 = *p;
                let point_next: Vec3 = wall.points[i_next];
                wall.vertices.push((point).into());
                wall.vertices.push((point + heightv).into());
                wall.vertices.push((point_next).into());
                wall.vertices.push((point_next + heightv).into());

                let diff = point_next.sub(point).length();
                wall.uvs.push([len / material_lengh, 0.]);
                wall.uvs.push([len / material_lengh, 1.]);
                wall.uvs.push([len / material_lengh, 0.]);
                wall.uvs.push([len / material_lengh, 1.]);

                let norm_arr = norm.to_array();
                wall.normals.push(norm_arr);
                wall.normals.push(norm_arr);
                wall.normals.push(norm_arr);
                wall.normals.push(norm_arr);
                len += diff;
            }
        }
        wall
    }
}
