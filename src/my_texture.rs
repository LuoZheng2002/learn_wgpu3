use std::{collections::HashMap, sync::Mutex};

use image::{GenericImageView, ImageBuffer, Rgba};
use lazy_static::lazy_static;
use rusttype::{Font, point};

use crate::{
    cache::{CacheValue, get_font},
    ui_node::UIIdentifier,
};

#[derive(Debug)]
pub struct MyTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}
#[derive(Hash, PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
pub enum TextureSource {
    FilePath(String),
    TextCharacter {
        character: char,
        font_file_path: String,
    },
    PureColor {
        red: u8,
        green: u8,
        blue: u8,
    },
    UI {
        version: u64,
        id: UIIdentifier,
    },
}

impl MyTexture {
    fn load_image_from_file_path(
        file_path: &str,
    ) -> Result<image::ImageBuffer<Rgba<u8>, Vec<u8>>, image::ImageError> {
        let img = image::open(file_path)?;
        Ok(img.to_rgba8())
    }
    fn load_image_from_text_character(
        character: char,
        font_file_path: String,
    ) -> image::ImageBuffer<Rgba<u8>, Vec<u8>> {
        let font = get_font(font_file_path);
        let font = match font.as_ref() {
            CacheValue::Font(font) => font,
            _ => panic!("Invalid cache value"),
        };
        let v_metrics = font.v_metrics(rusttype::Scale::uniform(1024.0));
        let ascent = v_metrics.ascent.round() as i32;
        let descent = v_metrics.descent.round() as i32;
        let scale = rusttype::Scale::uniform(1024.0);
        let glyph = font.glyph(character).scaled(scale);
        let h_metrics = glyph.h_metrics();
        let glyph = glyph.positioned(point(0.0, 0.0));
        let bounding_box = glyph.pixel_bounding_box().unwrap_or(rusttype::Rect {
            min: point(0, 0),
            max: point(1, 1),
        });
        let bounding_top = bounding_box.min.y;
        let bounding_bottom = bounding_box.max.y;
        let bounding_left = bounding_box.min.x;
        let bounding_right = bounding_box.max.x;
        let margin_top = ascent + bounding_top;
        // assert!(margin_top >= 0, "glpyh \"{}\" is above the top", character);
        // let margin_top = margin_top as u32;
        // let margin_bottom = -(descent + bounding_bottom);
        // assert!(
        //     margin_bottom >= 0,
        //     "glpyh \"{}\" is below the bottom",
        //     character
        // );
        // let _margin_bottom = margin_bottom as u32;

        let left_side_bearing = h_metrics.left_side_bearing.round() as i32;
        let advance_width = h_metrics.advance_width.round() as i32;

        let margin_left = left_side_bearing + bounding_left; // left_side_bearing - (abs bounding_left)
        // assert!(
        //     margin_left >= 0,
        //     "glyph \"{}\" margin left is negative",
        //     character
        // );
        // let margin_right = advance_width - bounding_right; // advance_width - (abs bounding_right)
        // assert!(
        //     margin_right >= 0,
        //     "glyph \"{}\" margin right is negative",
        //     character
        // );
        // let _margin_right = margin_right as u32;

        // let width = bounding_box.width() as u32;
        // let height = bounding_box.height() as u32;
        let width = advance_width as u32;
        let height = (ascent - descent) as u32;
        let mut image = image::ImageBuffer::new(width, height);
        glyph.draw(|x, y, v| {
            let x = x as i32 + margin_left;
            let y = y as i32 + margin_top;
            if x < 0 || x >= width as i32 || y < 0 || y >= height as i32 {
                return;
            }
            let intensity = (v * 255.0) as u8;
            image.put_pixel(x as u32, y as u32, Rgba([0, 255, 0, intensity]));
        });
        image
    }

    pub fn from_image(
        image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let image = image::imageops::flip_vertical(image);
        let dimensions = image.dimensions();
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
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
            &image,
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
        Self {
            texture,
            view,
            sampler,
        }
    }
    pub fn load(
        texture_source: TextureSource,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Self, image::ImageError> {
        let img = match texture_source {
            TextureSource::FilePath(ref file_path) => Self::load_image_from_file_path(file_path)?,
            TextureSource::TextCharacter {
                character,
                font_file_path,
            } => Self::load_image_from_text_character(character, font_file_path),
            _ => {
                panic!("Unsupported texture source");
            }
        };
        let my_texture = Self::from_image(&img, device, queue);
        Ok(my_texture)
    }

    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; // 1.

    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            // 2.
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
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            // 4.
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
        });
        Self {
            texture,
            view,
            sampler,
        }
    }

    pub fn create_render_attachment_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        label: Option<&str>,
    ) -> Self {
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self {
            texture,
            view,
            sampler,
        }
    }
}

lazy_static! {
    static ref FONTS: Mutex<HashMap<String, Font<'static>>> = Mutex::new(HashMap::new());
}
