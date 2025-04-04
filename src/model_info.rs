use std::sync::Arc;

use russimp::scene::Scene;




pub struct ModelInfo{
    pub opaque_mesh_indices: Vec<usize>,
    pub transparent_mesh_indices: Vec<usize>,
    // something that are similar to russimp::scene::Scene
}