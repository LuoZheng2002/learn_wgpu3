use std::{collections::HashMap, sync::Arc};

use russimp::{
    Matrix4x4,
    material::{Material, TextureType},
    scene::Scene,
};
use wgpu::BindGroup;

use crate::{model_meta::ModelMeta, my_texture::MyTexture};

#[derive(Debug, Clone)]
pub struct MyMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub material_bind_group: Arc<BindGroup>,
}

#[derive(Debug, Clone)]
pub struct MaterialBindGroup(BindGroup);
    // pub color_diffuse: cgmath::Vector4<f32>,
    // pub textures: HashMap<TextureType, Arc<MyTexture>>,
    // pub metallic: f32,
    // pub roughness: f32,
    // pub shininess: f32,
    // pub emmisive: cgmath::Vector4<f32>,
    // pub opacity: f32,

// This is a key to a mesh to be rendered
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct MeshMeta{
    pub model_meta: ModelMeta,
    pub mesh_index: usize,
}

pub struct ModelData {
    pub opaque_meshes: Vec<Arc<MyMesh>>,
    pub transparent_meshes: Vec<Arc<MyMesh>>,
}
