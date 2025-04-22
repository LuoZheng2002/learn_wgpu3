use std::sync::Arc;

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
pub enum TextureMeta {
    Texture { path: String },
    Font { font_path: String, character: char },
}

pub fn create_placeholder_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> Arc<CacheValue> {
    let texture = MyTexture::load(
        TextureSource::FilePath("assets/placeholder.png".to_string()),
        device,
        queue,
    )
    .unwrap();
    Arc::new(CacheValue::Texture(texture))
}
impl TextureMeta {
    pub fn to_ui_renderable(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        ui_pipeline: &UIPipeline,
    ) -> UIRenderable {
        let texture: Arc<CacheValue> = match self {
            TextureMeta::Texture { path } => CACHE.get_with(
                CacheKey::Texture(TextureSource::FilePath(path.clone())),
                || {
                    let texture =
                        MyTexture::load(TextureSource::FilePath(path.clone()), device, queue)
                            .unwrap();
                    Arc::new(CacheValue::Texture(texture))
                },
            ),
            TextureMeta::Font {
                font_path,
                character,
            } => CACHE.get_with(
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
        let material_bind_group = ui_pipeline.create_material_bind_group(device, texture);
        UIRenderable {
            material_bind_group,
        }
    }
}

// #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
// pub struct SortOrder{
//     pub canvas_order: u32,
//     pub element_order: u32,
// }
// impl PartialOrd for SortOrder {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         Some(self.cmp(other))
//     }
// }
// impl Ord for SortOrder {
//     fn cmp(&self, other: &Self) -> Ordering {
//         match self.canvas_order.cmp(&other.canvas_order) {
//             Ordering::Equal => self.element_order.cmp(&other.element_order),
//             ord => ord,
//         }
//     }
// }

// depth off
#[derive(Debug, Clone)]
pub struct UIInstance {
    pub location_left: f32,
    pub location_top: f32,
    pub location_right: f32,
    pub location_bottom: f32,
    pub flip_vertically: bool,
}

impl UIInstance {
    pub fn to_raw(&self) -> UIInstanceRaw {
        UIInstanceRaw {
            location: [
                self.location_left,
                self.location_top,
                self.location_right,
                self.location_bottom,
            ],
            flip_vertically: if self.flip_vertically { 1 } else { 0 },
        }
    }
}

// NEW!
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UIInstanceRaw {
    pub location: [f32; 4],
    pub flip_vertically: u32,
}

impl UIInstanceRaw {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x4, 1 => Uint32];
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
