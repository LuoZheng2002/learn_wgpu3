use std::{collections::HashMap, sync::Mutex};

use image::{GenericImageView, Rgba};
use lazy_static::lazy_static;
use rusttype::{point, Font};

use crate::render_context;

pub struct MyTexture {
    #[allow(unused)]
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}
#[derive(Hash, PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
pub enum TextureSource{
    FilePath(String),
    TextCharacter{character: char, font_file_path: String},
}

impl MyTexture {
    fn load_image_from_file_path(file_path: &str) -> Result<image::ImageBuffer<Rgba<u8>, Vec<u8>>, image::ImageError> {
        let img = image::open(file_path)?;
        Ok(img.to_rgba8())
    }
    fn load_image_from_text_character(character: char, font_file_path: String) -> image::ImageBuffer<Rgba<u8>, Vec<u8>> {
        let mut fonts = FONTS.lock().unwrap();
        let font = fonts.entry(font_file_path.clone()).or_insert_with(|| {
            let font_data = std::fs::read(font_file_path).unwrap();
            Font::try_from_vec(font_data).unwrap()
        });
        let scale = rusttype::Scale::uniform(1024.0);
        let glyph = font.glyph(character).scaled(scale).positioned(point(0.0, 0.0));
        let bounding_box = glyph.pixel_bounding_box().unwrap();
        let width = bounding_box.width() as u32;
        let height = bounding_box.height() as u32;
        let mut image = image::ImageBuffer::new(width, height);
        glyph.draw(|x, y, v| {
            let intensity = (v * 128.0) as u8;
            image.put_pixel(x, y, Rgba([255, 255, 255, intensity]));
        });
        image::imageops::flip_vertical(&image)
    }
    pub fn load(
        texture_source: TextureSource,
        render_context: &render_context::RenderContext,
        label: Option<&str>,
    ) -> Result<Self, image::ImageError> {
        let device = &render_context.device;
        let queue = &render_context.queue;
        let img = match texture_source {
            TextureSource::FilePath(ref file_path) => Self::load_image_from_file_path(file_path)?,
            TextureSource::TextCharacter { character, font_file_path } => Self::load_image_from_text_character(character, font_file_path),
        };
        let dimensions = img.dimensions();
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &img,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }
    
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; // 1.
    
    pub fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, label: &str) -> Self {
        let size = wgpu::Extent3d { // 2.
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor { // 4.
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual), // 5.
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }
        );
        Self { texture, view, sampler }
    }
}


lazy_static!{
    static ref FONTS: Mutex<HashMap<String, Font<'static>>> = Mutex::new(HashMap::new());
}