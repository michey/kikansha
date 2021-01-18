use crate::figure::Figure;
use crate::figure::IndexedMesh;
use crate::figure::MeshPoint;
use crate::figure::RegularMesh;
use crate::figure::RenderableMesh;
use crate::scene::Scene;
use crate::scene::ViewAndProject;
use gltf::mesh::util::ReadIndices::{U16, U32, U8};
use gltf::Gltf;

pub enum LoadingError {
    Ooops,
}

impl From<gltf::Error> for LoadingError {
    fn from(err: gltf::Error) -> Self {
        match err {
            _ => LoadingError::Ooops,
        }
    }
}

pub fn load_figures(path: &str) -> Result<Vec<RenderableMesh>, LoadingError> {
    let mut figures: Vec<RenderableMesh> = Vec::new();
    let (gltf, buffers, _) = gltf::import(path)?;
    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            let mut points: Vec<MeshPoint> = Vec::new();
            println!("- Primitive #{}, {:?}", primitive.index(), primitive.mode());
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let norm_iter = reader.read_normals();
            let vert_iter = reader.read_positions();

            // let color_iter = reader.read_colors();

            match (vert_iter, norm_iter) {
                (Some(verts), Some(norms)) => {
                    let iter = verts.zip(norms);
                    for (vert, norm) in iter {
                        points.push(MeshPoint::new(vert, norm, [1.0, 1.0, 1.0]))
                    }
                }
                (_, _) => {}
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
                Some(indices) => RenderableMesh::Indexed(IndexedMesh {
                    points: points,
                    indices: indices,
                }),
                None => RenderableMesh::Regular(RegularMesh { points: points }),
            };
            figures.push(mesh);
        }
    }
    Ok(figures)
}

pub fn load_scene_from_file<T: ViewAndProject + Sized>(
    path: &str,
) -> Result<Scene<T>, LoadingError> {
    // let (document, buffers, images) = gltf::import(path)?;
    let gltf = Gltf::open(path)?;
    for scene in gltf.scenes() {
        for node in scene.nodes() {
            println!(
                "Node #{} has {} children",
                node.index(),
                node.children().count(),
            );
        }
    }
    Err(LoadingError::Ooops)
}
