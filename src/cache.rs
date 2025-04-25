use std::sync::Arc;

use lazy_static::lazy_static;
use moka::sync::Cache;
use rusttype::Font;

use crate::{
    model_data::ModelData, model_meta::ModelMeta, my_texture::{MyTexture, TextureSource}, ui_node::UIIdentifier, ui_renderable::{TextureMeta, UIRenderable}
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CacheKey {
    ModelMeta(ModelMeta),
    UIRenderableMeta(TextureMeta),
    Texture(TextureSource),
    UITexture(UIIdentifier), // to do
    Font(String),
    PlaceholderTexture,
    Placeholder,
}

pub enum CacheValue {
    ModelData(ModelData),
    UIRenderable(UIRenderable),
    Texture(MyTexture),
    UITexture{
        texture: MyTexture,
        version: u64,
    },
    Font(Font<'static>),
    Placeholder,
}

lazy_static! {
    // This is a simple in-memory cache to store the last computed value of a function.
    // In a real-world application, you might want to use a more sophisticated caching mechanism.
    pub static ref CACHE: Cache<CacheKey, Arc<CacheValue>> = {
        // Create a cache with a maximum size of 100 items and an expiration time of 60 seconds.
        Cache::builder()
            .max_capacity(100) // Maximum number of items in the cache
            // .time_to_live(std::time::Duration::from_secs(60)) // Time to live for each item in the cache
            .eviction_listener(|key, _value, cause| {
                // This function is called when an item is evicted from the cache.
                // You can use it to log or perform any cleanup if necessary.
                println!("Evicted key {key:?}. Cause: {cause:?}");
            })
            .build()
    };
}

pub fn get_font(font_file_path: String) -> Arc<CacheValue> {
    CACHE.get_with(CacheKey::Font(font_file_path.clone()), || {
        let font_data = std::fs::read(font_file_path).unwrap();
        let font = Font::try_from_vec(font_data).unwrap();
        let font = CacheValue::Font(font);
        Arc::new(font)
    })
}
