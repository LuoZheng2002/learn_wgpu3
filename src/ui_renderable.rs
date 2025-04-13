use std::sync::Arc;

use wgpu::{BindGroupLayout, Device};

use crate::{
    cache::{CACHE, CacheKey, CacheValue},
    my_texture::{MyTexture, TextureSource},
    ui_pipeline::UIPipeline,
};

// #[repr(C)]
// #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// pub struct MaterialUniform {
//     use_texture: u32,
//     _padding: [u32; 3],   // 12 bytes (to align the next vec4<f32>)
// }

// impl MaterialUniform {
//     pub fn new(use_texture: bool) -> Self {
//         Self {
//             use_texture: if use_texture { 1 } else { 0 },
//             _padding: [0; 3],
//         }
//     }
// }

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum UIRenderableMeta {
    Color,
    Texture { path: String },
    Font { character: char },
}

impl UIRenderableMeta {
    pub fn to_ui_renderable(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        ui_pipeline: &UIPipeline,
    ) -> UIRenderable {
        let font_path = "assets/times.ttf".to_string();
        let texture: Arc<CacheValue> = match self {
            UIRenderableMeta::Color => CACHE.get_with(CacheKey::PlaceholderTexture, || {
                let texture = MyTexture::load(
                    TextureSource::FilePath("assets/placeholder.png".to_string()),
                    device,
                    queue,
                )
                .unwrap();
                Arc::new(CacheValue::Texture(texture))
            }),
            UIRenderableMeta::Texture { path } => CACHE.get_with(
                CacheKey::Texture(TextureSource::FilePath(path.clone())),
                || {
                    let texture =
                        MyTexture::load(TextureSource::FilePath(path.clone()), device, queue)
                            .unwrap();
                    Arc::new(CacheValue::Texture(texture))
                },
            ),
            UIRenderableMeta::Font { character } => CACHE.get_with(
                CacheKey::Texture(TextureSource::TextCharacter {
                    character: *character,
                    font_file_path: font_path.clone(),
                }),
                || {
                    let texture = MyTexture::load(
                        TextureSource::TextCharacter {
                            font_file_path: font_path.clone(),
                            character: *character,
                        },
                        device,
                        queue,
                    )
                    .unwrap();
                    Arc::new(CacheValue::Texture(texture))
                },
            ),
        };
        let texture = match texture.as_ref() {
            CacheValue::Texture(texture) => texture,
            _ => unreachable!(),
        };
        let material_bind_group = ui_pipeline.create_material_bind_group(device, queue, texture);
        UIRenderable {
            material_bind_group,
        }
    }
}

pub struct UIInstance {
    pub location: [f32; 4],
    pub color: cgmath::Vector4<f32>,
    pub sort_order: u32,
    pub use_texture: bool,
}

impl UIInstance {
    pub fn to_raw(&self, norm_factor: f32) -> UIInstanceRaw {
        UIInstanceRaw {
            color: self.color.into(),
            location: self.location,
            depth: 1.0 - self.sort_order as f32 / norm_factor,
            use_texture: if self.use_texture { 1 } else { 0 },
        }
    }
}

// NEW!
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UIInstanceRaw {
    pub location: [f32; 4],
    pub color: [f32; 4],
    pub depth: f32,
    pub use_texture: u32,
}

impl UIInstanceRaw {
    const ATTRIBS: [wgpu::VertexAttribute; 4] =
        wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32, 3 =>Uint32];
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<UIInstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct UIRenderable {
    pub material_bind_group: wgpu::BindGroup,
}
