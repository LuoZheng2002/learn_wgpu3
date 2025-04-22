use std::{collections::HashMap, io::Cursor, sync::Arc};

use image::{ImageBuffer, ImageReader};
use russimp::{
    material::{DataContent, Material, PropertyTypeInfo, TextureType},
    scene::{PostProcess, Scene},
};
use wgpu::{BindGroup, util::DeviceExt};

use crate::{
    model_data::{ModelData, MyMesh},
    opaque_pipeline::OpaquePipeline,
    vertex::Vertex,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ModelMeta {
    pub path: String,
}

impl ModelMeta {
    pub fn new(path: String) -> Self {
        Self { path }
    }
    pub fn load_model(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        opaque_pipeline: &OpaquePipeline,
    ) -> ModelData {
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
        let mut material_bind_group_cache: HashMap<u32, Arc<BindGroup>> = HashMap::new();
        fn material_to_bind_group(
            opaque_pipeline: &OpaquePipeline,
            material: &Material,
            device: &wgpu::Device,
            queue: &wgpu::Queue,
        ) -> Arc<BindGroup> {
            let diffuse_texture =
                if let Some(texture) = material.textures.get(&TextureType::Diffuse) {
                    texture
                } else {
                    // to do
                    material.textures.get(&TextureType::BaseColor).unwrap()
                };

            let diffuse_texture = diffuse_texture.borrow();

            let data = match &diffuse_texture.data {
                DataContent::Bytes(bytes) => bytes,
                _ => panic!("Unsupported texture type"),
            };
            let diffuse_image = match diffuse_texture.height {
                0 => ImageReader::new(Cursor::new(data))
                    .with_guessed_format()
                    .unwrap()
                    .decode()
                    .unwrap()
                    .into_rgba8(),
                _ => ImageBuffer::from_raw(
                    diffuse_texture.width,
                    diffuse_texture.height,
                    data.clone(),
                )
                .unwrap(),
            };
            // let diffuse_image = flip_vertical(&diffuse_image);
            // to do
            let material_bind_group =
                opaque_pipeline.create_material_bind_group(device, queue, &diffuse_image);
            Arc::new(material_bind_group)
        }

        println!("Number of meshes: {}", root.meshes.len());
        for mesh in root.meshes.iter() {
            let mesh = scene.meshes.get(*mesh as usize).unwrap();
            let material = scene.materials.get(mesh.material_index as usize).unwrap();
            let properties = material
                .properties
                .iter()
                .map(|property| {
                    let key = (property.key.clone(), property.semantic.clone());
                    let value = property.clone();
                    (key, value)
                })
                .collect::<HashMap<_, _>>();
            let diffuse_uv_channel_property = if let Some(property) =
                properties.get(&("$tex.uvwsrc".to_string(), TextureType::Diffuse))
            {
                property
            } else if let Some(property) =
                properties.get(&("$tex.uvwsrc".to_string(), TextureType::BaseColor))
            {
                property
            } else {
                panic!("No diffuse uv channel found");
            };
            let diffuse_uv_channel = &diffuse_uv_channel_property.data;
            let diffuse_uv_channel = match diffuse_uv_channel {
                PropertyTypeInfo::IntegerArray(value) => value,
                _ => panic!("Unsupported texture type"),
            };
            let diffuse_uv_channel_index = diffuse_uv_channel[0];

            let tex_coords = mesh
                .texture_coords
                .get(diffuse_uv_channel_index as usize)
                .unwrap();
            let tex_coords = tex_coords.as_ref().unwrap();

            let mut vertices: Vec<Vertex> = Vec::new();
            assert!(mesh.vertices.len() == tex_coords.len());
            for i in 0..mesh.vertices.len() {
                let vertex = mesh.vertices[i];
                let tex_coord = tex_coords[i];
                let normal = mesh.normals[i];
                vertices.push(Vertex {
                    position: [vertex.x, vertex.y, vertex.z],
                    tex_coords: [tex_coord.x, tex_coord.y],
                    normal: [normal.x, normal.y, normal.z],
                });
            }
            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let mut indices: Vec<u16> = Vec::new();
            for face in mesh.faces.iter() {
                assert!(face.0.len() == 3);
                for i in 0..3 {
                    indices.push(face.0[i] as u16);
                }
            }
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });
            let material_bind_group = material_bind_group_cache
                .entry(mesh.material_index)
                .or_insert_with(|| material_to_bind_group(opaque_pipeline, material, device, queue))
                .clone();

            let my_mesh = MyMesh {
                vertex_buffer,
                index_buffer,
                material_bind_group,
                num_indices: indices.len() as u32,
            };
            // determine if the mesh is opaque or transparent
            let opacity = if let Some(opacity_property) =
                properties.get(&("$mat.opacity".to_string(), TextureType::None))
            {
                let opacity_property = match &opacity_property.data {
                    PropertyTypeInfo::FloatArray(value) => value,
                    _ => panic!("Unsupported texture type"),
                };
                opacity_property[0]
            } else {
                println!("Warning: no opacity property found");
                1.0
            };
            if opacity < 1.0 {
                println!("Mesh is transparent");
                transparent_meshes.push(Arc::new(my_mesh));
            } else {
                println!("Mesh is opaque");
                opaque_meshes.push(Arc::new(my_mesh));
            }
        }
        ModelData {
            opaque_meshes,
            transparent_meshes,
        }
    }
}
