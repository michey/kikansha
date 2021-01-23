use crate::figure::IndexedMesh;
use crate::figure::MeshPoint;
use crate::figure::RegularMesh;
use crate::figure::RenderableMesh;
use gltf::mesh::util::ReadIndices::{U16, U32, U8};
use nalgebra::Vector3;
use nalgebra_glm::cross;
use nalgebra_glm::length;
use nalgebra_glm::normalize;

pub enum LoadingError {
    Ooops,
}

impl From<gltf::Error> for LoadingError {
    fn from(_: gltf::Error) -> Self {
        LoadingError::Ooops
    }
}

pub fn load_figures(path: &str) -> Result<Vec<RenderableMesh>, LoadingError> {
    let mut figures: Vec<RenderableMesh> = Vec::new();
    let (gltf, buffers, _) = gltf::import(path)?;
    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            let mut points: Vec<MeshPoint> = Vec::new();
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let norm_iter = reader.read_normals();
            let vert_iter = reader.read_positions();

            let tangents_iter = reader.read_tangents();

            match (vert_iter, norm_iter, tangents_iter) {
                (Some(verts), Some(norms), Some(tangents)) => {
                    let iter = verts.zip(norms).zip(tangents);
                    for ((vert, norm), tang) in iter {
                        points.push(MeshPoint::new(
                            vert,
                            [1.0, 1.0, 1.0],
                            norm,
                            [tang[0], tang[1], tang[2]],
                        ))
                    }
                }
                (Some(verts), Some(norms), None) => {
                    let iter = verts.zip(norms);
                    for (vert, norm) in iter {
                        let tangent = calc_tangent(norm.clone());
                        points.push(MeshPoint::new(vert, [1.0, 1.0, 1.0], norm, tangent))
                    }
                }
                (_, _, _) => {}
            }

            let o_indices = reader.read_indices().map(|indcs| {
                let indices: Vec<u32> = match indcs {
                    U8(iter) => iter.map(|i| i as u32).collect(),
                    U16(iter) => iter.map(|i| i as u32).collect(),
                    U32(iter) => iter.map(|i| i as u32).collect(),
                };
                indices
            });

            let mesh = match o_indices {
                Some(indices) => RenderableMesh::Indexed(IndexedMesh { points, indices }),
                None => RenderableMesh::Regular(RegularMesh { points }),
            };
            figures.push(mesh);
        }
    }
    Ok(figures)
}

pub fn calc_tangent(norm: [f32; 3]) -> [f32; 3] {
    let v1 = Vector3::new(0.0, 0.0, 1.0);
    let v2 = Vector3::new(0.0, 1.0, 0.0);

    let v_norm = Vector3::new(norm[0], norm[1], norm[2]);
    let c1: Vector3<f32> = cross(&v_norm, &v1);
    let c2: Vector3<f32> = cross(&v_norm, &v2);

    let mut tang = if length(&c1) > length(&c2) { c1 } else { c2 };

    tang = normalize(&tang);
    tang.into()
}
