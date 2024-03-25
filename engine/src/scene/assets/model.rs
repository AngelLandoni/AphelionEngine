use std::{error::Error, fmt, path::Path};

use crate::graphics::vertex::Vertex;

/// A representation of all the loadable model types.
pub enum ModelType<'a> {
    /// Represents an Obj model file.
    Obj(&'a Path),
}

pub struct Model {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

#[derive(Debug)]
pub enum ModelLoaderError {
    InvalidBuffer,
}

impl Error for ModelLoaderError {}

impl fmt::Display for ModelLoaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Oh no, something bad went down")
    }
}

impl<'a> ModelType<'a> {
    /// Loads the provided file and returns the model or an error if the model
    /// contains an incorrect format.
    pub fn load_model(&'a self) -> Result<Vec<Model>, ModelLoaderError> {
        match self {
            ModelType::Obj(path) => {
                let (models, _) = tobj::load_obj(
                    *path,
                    &tobj::LoadOptions {
                        triangulate: true,
                        ..Default::default()
                    },
                )
                .map_err(|_| ModelLoaderError::InvalidBuffer)?;

                let local_models = models
                    .iter()
                    .map(|m| Model {
                        name: m.name.clone(),
                        vertices: (0..m.mesh.positions.len() / 3)
                            .map(|i| Vertex {
                                pos: [
                                    m.mesh.positions[i * 3],
                                    m.mesh.positions[i * 3 + 1],
                                    m.mesh.positions[i * 3 + 2],
                                ],
                                col: [
                                    *m.mesh
                                        .vertex_color
                                        .get(i * 3)
                                        .unwrap_or(&1.0),
                                    *m.mesh
                                        .vertex_color
                                        .get(i * 3 + 1)
                                        .unwrap_or(&1.0),
                                    *m.mesh
                                        .vertex_color
                                        .get(i * 3 + 2)
                                        .unwrap_or(&1.0),
                                ],
                                uv: [
                                    *m.mesh
                                        .texcoords
                                        .get(i * 3)
                                        .unwrap_or(&1.0),
                                    *m.mesh
                                        .texcoords
                                        .get(i * 3 + 1)
                                        .unwrap_or(&1.0),
                                ],
                            })
                            .collect(),
                        indices: m
                            .mesh
                            .indices
                            .iter()
                            .map(|index| *index as u16)
                            .collect(),
                    })
                    .collect::<Vec<_>>();

                Ok(local_models)
            }
        }
    }
}
