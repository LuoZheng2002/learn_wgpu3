use std::{collections::VecDeque, sync::Arc};

use russimp::scene::{PostProcess, Scene};

use crate::model_data::{ModelData, MyMaterial, MyMesh};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ModelMeta {
    pub path: String,
}

impl ModelMeta {
    pub fn new(path: String) -> Self {
        Self { path }
    }
    pub fn load_model(&self) -> ModelData {
        let scene = Scene::from_file(
            &self.path,
            vec![
                PostProcess::CalculateTangentSpace,
                PostProcess::Triangulate,
                PostProcess::JoinIdenticalVertices,
                PostProcess::SortByPrimitiveType,
            ],
        )
        .unwrap();

        let mut opaque_meshes = Vec::new();
        let mut transparent_meshes = Vec::new();
        let root = &scene.root.unwrap();
        let materials = scene.materials.iter().map(|material|{
            let mut color_diffuse: Option<f32> = None;

            let my_material = MyMaterial{
                color_diffuse: cgmath::Vector4 { x: material.color_diffuse.x, y: material.color_diffuse.y, z: material.color_diffuse.z, w: 1.0 },
                textures: material.textures.clone(),
                metallic: material.metallic,
                roughness: material.roughness,
                shininess: material.shininess,
                emmisive: cgmath::Vector4 { x: material.emmisive.x, y: material.emmisive.y, z: material.emmisive.z, w: 1.0 },
                opacity: material.opacity,
            };
            Arc::new(my_material);          
        });
        for mesh in root.meshes.iter(){
            let mesh = scene.meshes.get(*mesh as usize).unwrap();
            let material = scene.materials.get(mesh.material_index as usize).unwrap();
                

            let my_mesh = MyMesh{
                normals: mesh.normals.iter().map(|n| cgmath::Vector3 { x: n.x, y: n.y, z: n.z}).collect(),
                name: mesh.name.clone(),
                vertices: mesh.vertices.iter().map(|v| cgmath::Vector3 { x: v.x, y: v.y, z: v.z}).collect(),
                material: 
                transformation: mesh.transformation,
                texture_coords: mesh.texture_coords.clone(),
                indices: mesh.indices.clone(),
                colors: mesh.colors.clone(),
            }
        }
        ModelData {
            opaque_meshes,
            transparent_meshes,
        }
    }
}
