use std::{collections::HashMap, time::Instant};

use cgmath::{Euler, Quaternion};

use crate::{
    mesh_meta::MeshMeta, model_instance::ModelInstance, model_meta::ModelMeta, my_camera::MyCamera,
    opaque_mesh_instance::OpaqueMeshInstance, render_context::RenderContext,
};

// model path,

pub struct State {
    // camera stuff
    pub camera: MyCamera,
    // accumulated time
    pub timer: Option<Instant>,
    pub prev_time: Option<f32>,
    pub fps_timer: Option<Instant>,
    pub accumulated_frame_num: u32,
    pub render_submission: HashMap<ModelMeta, Vec<ModelInstance>>,
}

impl State {
    fn submit_renderable(&mut self, model_meta: ModelMeta, instance: ModelInstance) {
        self.render_submission
            .entry(model_meta)
            .or_insert_with(|| Vec::new())
            .push(instance);
    }

    pub fn update(&mut self) {
        // calculate fps every 1 second
        let fps_timer = self.fps_timer.get_or_insert_with(|| Instant::now());
        let current_time = fps_timer.elapsed().as_secs_f32();
        if current_time >= 1.0 {
            println!("FPS: {}", self.accumulated_frame_num);
            self.accumulated_frame_num = 0;
            *fps_timer = Instant::now();
        } else {
            self.accumulated_frame_num += 1;
        }
        let timer = self.timer.get_or_insert_with(|| Instant::now());
        let current_time = timer.elapsed().as_secs_f32();
        let prev_time = self.prev_time.get_or_insert(current_time);
        let delta_time = current_time - *prev_time;
        assert!(delta_time >= 0.0);
        *prev_time = current_time;
        let model_meta = ModelMeta {
            path: "assets/mesh.obj".to_string(),
        };
        let instance1 = ModelInstance {
            position: [0.0, 0.0, 0.0].into(),
            rotation: Quaternion::from(Euler::new(
                cgmath::Rad(current_time),
                cgmath::Rad(current_time),
                cgmath::Rad(current_time),
            )),
        };
        let instance2 = ModelInstance {
            position: [1.0, 0.0, 0.5].into(),
            rotation: Quaternion::from(Euler::new(
                cgmath::Rad(-current_time),
                cgmath::Rad(current_time),
                cgmath::Rad(-current_time),
            )),
        };
        self.submit_renderable(model_meta.clone(), instance1);
        self.submit_renderable(model_meta.clone(), instance2);
    }
}

impl Default for State {
    fn default() -> Self {
        State {
            camera: MyCamera::default(),
            timer: None,
            prev_time: None,
            fps_timer: None,
            accumulated_frame_num: 0,
            render_submission: HashMap::new(),
        }
    }
}
