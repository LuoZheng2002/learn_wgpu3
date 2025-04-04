use std::{any::TypeId, sync::Arc};

use lazy_static::lazy_static;
use moka::sync::Cache;

use crate::{
    mesh_meta::MeshMeta, model_data::ModelData, model_meta::ModelMeta,
    opaque_mesh_data::OpaqueMeshData,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CacheKey {
    ModelMeta(ModelMeta),
    OpaqueMeshMeta(MeshMeta),
    Placeholder,
}

pub enum CacheValue {
    ModelData(ModelData),
    OpaqueMeshModel(OpaqueMeshData),
    Placeholder,
}

lazy_static! {
    // This is a simple in-memory cache to store the last computed value of a function.
    // In a real-world application, you might want to use a more sophisticated caching mechanism.
    pub static ref CACHE: Cache<CacheKey, Arc<CacheValue>> = {
        // Create a cache with a maximum size of 100 items and an expiration time of 60 seconds.
        Cache::builder()
            .max_capacity(100) // Maximum number of items in the cache
            .time_to_live(std::time::Duration::from_secs(60)) // Time to live for each item in the cache
            .build()
    };
}
