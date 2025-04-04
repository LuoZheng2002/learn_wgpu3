use lazy_static::lazy_static;
use wgpu::{BindGroupLayout, util::DeviceExt};

use crate::{
    my_texture::{MyTexture, TextureSource},
    vertex::Vertex,
};

pub struct OpaqueMeshData {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub texture_bind_group: wgpu::BindGroup,
}

impl OpaqueMeshData {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = INDICES.len() as u32;
        let texture = MyTexture::load(
            TextureSource::FilePath("assets/grass.jpg".to_string()),
            device,
            queue,
            Some("grass texture"),
        )
        .unwrap();

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        });
        Self {
            vertex_buffer,
            index_buffer,
            num_indices,
            texture_bind_group,
        }
    }
}

lazy_static! {
    #[rustfmt::skip]
    static ref VERTICES: &'static [Vertex] = &[
        // Front face (Z = 1)
        Vertex {position: [-1.0, -1.0, 1.0],tex_coords: [1./4., 2./3.],},
        Vertex {position: [1.0, -1.0, 1.0],tex_coords: [2./4., 2./3.],},
        Vertex {position: [1.0, 1.0, 1.0],tex_coords: [2./4., 1./3.],},
        Vertex {position: [-1.0, 1.0, 1.0],tex_coords: [1./4., 1./3.],},
        // Back face (Z = -1)
        Vertex {position: [-1.0, -1.0, -1.0],tex_coords: [3./4., 2./3.],},
        Vertex {position: [1.0, -1.0, -1.0],tex_coords: [1.0, 2./3.],},
        Vertex {position: [1.0, 1.0, -1.0],tex_coords: [1.0, 1./3.],},
        Vertex {position: [-1.0, 1.0, -1.0],tex_coords: [3./4., 1./3.],},
        // Left face (X = -1)
        Vertex {position: [-1.0, -1.0, -1.0],tex_coords: [0.0, 2./3.],},
        Vertex {position: [-1.0, -1.0, 1.0],tex_coords: [1./4., 2./3.],},
        Vertex {position: [-1.0, 1.0, 1.0],tex_coords: [1./4., 1./3.],},
        Vertex {position: [-1.0, 1.0, -1.0],tex_coords: [0.0, 1./3.],},
        // Right face (X = 1)
        Vertex {position: [1.0, -1.0, -1.0],tex_coords: [0.0, 0.0],},
        Vertex {position: [1.0, 1.0, -1.0],tex_coords: [0.0, 1.0],},
        Vertex {position: [1.0, 1.0, 1.0],tex_coords: [1.0, 1.0],},
        Vertex {position: [1.0, -1.0, 1.0],tex_coords: [1.0, 0.0],},
        // Top face (Y = 1)
        Vertex {position: [-1.0, 1.0, -1.0],tex_coords: [0.0, 0.0],},
        Vertex {position: [-1.0, 1.0, 1.0],tex_coords: [0.0, 1.0],},
        Vertex {position: [1.0, 1.0, 1.0],tex_coords: [1.0, 1.0],},
        Vertex {position: [1.0, 1.0, -1.0],tex_coords: [1.0, 0.0],},
        // Bottom face (Y = -1)
        Vertex {position: [-1.0, -1.0, -1.0],tex_coords: [0.0, 0.0],},
        Vertex {position: [1.0, -1.0, -1.0],tex_coords: [1.0, 0.0],},
        Vertex {position: [1.0, -1.0, 1.0],tex_coords: [1.0, 1.0],},
        Vertex {position: [-1.0, -1.0, 1.0],tex_coords: [0.0, 1.0],},
    ];
    #[rustfmt::skip]
    static ref INDICES: &'static [u16] = &[
        0, 1, 2, 2, 3, 0, // Front
        4, 6, 5, 6, 4, 7, // Back
        8, 9, 10, 10, 11, 8, // Left
        12, 13, 14, 14, 15, 12, // Right
        16, 17, 18, 18, 19, 16, // Top
        20, 21, 22, 22, 23, 20, // Bottom
    ];
}
