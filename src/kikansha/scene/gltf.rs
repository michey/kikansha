use crate::figure::Figure;
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

pub fn load_figures(path: &str) -> Result<Vec<Figure>, LoadingError> {
    let mut figures: Vec<Figure> = Vec::new();
    let (gltf, buffers, _) = gltf::import(path)?;
    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            println!("- Primitive #{}, {:?}", primitive.index(), primitive.mode());
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let norm_iter = reader.read_normals();
            let vert_iter = reader.read_positions();

            // match (vert_iter, norm_iter)  {
            //     (Some(verts), Some(norms)) => {
            //         verts.chain(norms)
            //         .map(|(norm, vert)| {
            //             Triangle::new()
            //         } )
            //     },
            //     (_, _) => {}
            // }

            if let Some(norm_iter) = reader.read_normals() {
                let mut norm_count = 0;
                for norm in norm_iter {
                    // println!("Norm: {:?}", norm);
                    norm_count += 1;
                }
                println!("Norm: {:?}", norm_count);
            }

            if let Some(indcs) = reader.read_indices() {
                let mut ind_count = 0;

                match indcs {
                    U8(iter) => {
                        for i in iter {
                            // println!("Position: {:?}", vertex_position);
                            ind_count += 1;
                        }
                    }
                    U16(iter) => {
                        for i in iter {
                            // println!("Position: {:?}", vertex_position);
                            ind_count += 1;
                        }
                    }
                    U32(iter) => {
                        for i in iter {
                            // println!("Position: {:?}", vertex_position);
                            ind_count += 1;
                        }
                    }
                }

                println!("Ind: {:?}", ind_count);
            }

            if let Some(vert_iter) = reader.read_positions() {
                let mut vert_count = 0;
                for vertex_position in vert_iter {
                    // println!("Position: {:?}", vertex_position);
                    vert_count += 1;
                }
                println!("Vert: {:?}", vert_count);
            }
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
