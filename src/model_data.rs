use std::{collections::HashMap, sync::Arc};

use russimp::{
    Matrix4x4,
    material::{Material, TextureType},
    scene::Scene,
};

use crate::my_texture::MyTexture;

#[derive(Debug, Clone)]
pub struct MyMesh {
    pub normals: Vec<cgmath::Vector3<f32>>,
    pub name: String,
    pub vertices: Vec<cgmath::Vector3<f32>>,
    pub material: Arc<MyMaterial>,
    pub transformation: cgmath::Matrix4<f32>,
    pub texture_coords: HashMap<TextureType, Vec<cgmath::Vector3<f32>>>,
    pub indices: Vec<u32>,
    pub colors: HashMap<TextureType, Vec<cgmath::Vector4<f32>>>,
}

#[derive(Debug, Clone)]
pub struct MyMaterial {
    pub color_diffuse: cgmath::Vector4<f32>,
    pub textures: HashMap<TextureType, Arc<MyTexture>>,
    pub metallic: f32,
    pub roughness: f32,
    pub shininess: f32,
    pub emmisive: cgmath::Vector4<f32>,
    pub opacity: f32,
}

pub struct ModelData {
    pub opaque_meshes: Vec<MyMesh>,
    pub transparent_meshes: Vec<Arc<MyMesh>>,
}
