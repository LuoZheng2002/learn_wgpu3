use std::{collections::HashMap, time::Instant};

use cgmath::{Euler, Quaternion};
use either::Either;

use crate::{
    model_instance::ModelInstance,
    model_meta::ModelMeta,
    my_camera::MyCamera,
    ui::{Button, Span, SpanDirection, Text, ToUINode},
    ui_node::{
        BoundedLength, HorizontalAlignment, RelativeLength, UIRenderInstruction, VerticalAlignment,
    }, ui_renderable::TextureMeta,
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
    pub model_render_submissions: HashMap<ModelMeta, Vec<ModelInstance>>,
    // use Arc here because we need to map the container to another container
    // pub ui_render_submissions: HashMap<TextureMeta, Vec<UIInstance>>,
    pub ui_render_instructions: Vec<UIRenderInstruction>,
    pub light_position: cgmath::Vector3<f32>,
    pub fps: u32,
    pub canvas: Option<Span>,
}

impl State {
    fn submit_renderable(&mut self, model_meta: ModelMeta, instance: ModelInstance) {
        self.model_render_submissions
            .entry(model_meta)
            .or_insert_with(|| Vec::new())
            .push(instance);
    }
    // fn submit_ui_renderable(&mut self, ui_meta: TextureMeta, instance: UIInstance) {
    //     self.ui_render_submissions
    //         .entry(ui_meta)
    //         .or_insert_with(|| Vec::new())
    //         .push(instance);
    // }
    fn submit_ui_render_instruction(&mut self, ui_render_instruction: UIRenderInstruction) {
        self.ui_render_instructions.push(ui_render_instruction);
    }

    pub fn init(&mut self) {
        let text = Text::new(
            // format!("fps: {}", self.fps).to_string(),
            "".into(),
            "assets/times.ttf".to_string(),
            50.0,
            Either::Left(RelativeLength::Pixels(0)),
            Either::Left(RelativeLength::Pixels(0)),
            cgmath::Vector4 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
                w: 1.0,
            },
            BoundedLength::fixed_pixels(800),
            BoundedLength::fixed_pixels(400),
        );
        let button = Button::new(
            BoundedLength::fixed_pixels(300),
            BoundedLength::fixed_pixels(100),
            Either::Left(RelativeLength::Pixels(20)),
            Either::Left(RelativeLength::Pixels(20)),
            cgmath::Vector4 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
                w: 1.0,
            },
        );
        let span2 = Span::new(
            SpanDirection::Horizontal,
            BoundedLength::fixed_pixels(80),
            BoundedLength::fixed_pixels(60),
            Either::Left(RelativeLength::Pixels(3)),
            Either::Left(RelativeLength::Pixels(3)),
            HorizontalAlignment::Left,
            VerticalAlignment::Top,
            true,
            TextureMeta::Texture { path: "assets/grass.jpg".into() }
        );
        let span3 = Span::new(
            SpanDirection::Horizontal,
            BoundedLength::fixed_pixels(80),
            BoundedLength::fixed_pixels(60),
            Either::Left(RelativeLength::Pixels(3)),
            Either::Left(RelativeLength::Pixels(3)),
            HorizontalAlignment::Left,
            VerticalAlignment::Top,
            true,
            TextureMeta::Texture { path: "assets/grass.jpg".into() }
        );
        let mut span = Span::new(
            SpanDirection::Horizontal,
            BoundedLength::fixed_dependent(RelativeLength::RelativeScreenWidth(0.5)),
            BoundedLength::fixed_dependent(RelativeLength::RelativeScreenHeight(0.5)),
            Either::Left(RelativeLength::Pixels(3)),
            Either::Left(RelativeLength::Pixels(20)),
            HorizontalAlignment::Left,
            VerticalAlignment::Top,
            true,
            TextureMeta::Texture { path: "assets/genshin.jpg".into() }
        );
        span.push_child(Box::new(span2));
        span.push_child(Box::new(span3));
        // span.push_child(Box::new(button));
        self.canvas = Some(span);
    }

    pub fn update(&mut self, window_size: &winit::dpi::PhysicalSize<u32>) {
        // calculate fps every 1 second
        let fps_timer = self.fps_timer.get_or_insert_with(|| Instant::now());
        let current_time = fps_timer.elapsed().as_secs_f32();
        if current_time >= 1.0 {
            println!("FPS: {}", self.accumulated_frame_num);
            self.fps = self.accumulated_frame_num;
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
            path: "assets/rabbit2.glb".to_string(),
        };

        // rotate light in a unit circle
        let light_radius = 5.0;
        let light_angle = current_time * 0.5;
        self.light_position.x = light_radius * light_angle.cos();
        self.light_position.z = light_radius * light_angle.sin();

        let scale = 1.0;
        let speed = 0.0;
        let delta_angle = current_time * speed;
        let instance1 = ModelInstance {
            position: [-1.0, 0.0, 0.0].into(),
            rotation: Quaternion::from(Euler::new(
                cgmath::Rad(delta_angle),
                cgmath::Rad(delta_angle),
                cgmath::Rad(delta_angle),
            )),
            scale: cgmath::Vector3::new(scale, scale, scale),
        };
        let instance2 = ModelInstance {
            position: [1.0, 0.0, 0.0].into(),
            rotation: Quaternion::from(Euler::new(
                cgmath::Rad(-delta_angle),
                cgmath::Rad(delta_angle),
                cgmath::Rad(-delta_angle),
            )),
            scale: cgmath::Vector3::new(scale, scale, scale),
        };
        self.submit_renderable(model_meta.clone(), instance1);
        self.submit_renderable(model_meta.clone(), instance2);

        // let ui_meta1 = UIRenderableMeta::Font { character: 'F' };
        // let ui_instance1 = UIInstance {
        //     color: cgmath::Vector4::new(1.0, 0.0, 1.0, 1.0),
        //     location: [-0.2, 0.9, 0.7, -0.1],
        //     sort_order: 0,
        //     use_texture: true,
        // };
        // self.submit_ui_renderable(ui_meta1, ui_instance1);

        assert!(self.ui_render_instructions.is_empty());
        // to do
        let screen_width = window_size.width;
        let screen_height = window_size.height;
        let ui_node = self.canvas.as_ref().unwrap().to_ui_node();
        let ui_node =
            ui_node.calculate_dimensions(screen_width, screen_height, screen_width, screen_height);
        let ui_node = ui_node.flatten_children();
        let render_instruction = ui_node.to_ui_render_instruction(screen_width, screen_height);
        self.submit_ui_render_instruction(render_instruction);
        // panic!()
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
            model_render_submissions: HashMap::new(),
            // ui_render_submissions: HashMap::new(),
            ui_render_instructions: Vec::new(),
            light_position: cgmath::Vector3::new(0.0, 0.0, 0.0),
            fps: 0,
            canvas: None,
        }
    }
}

// if use event or callback, need mutable shared reference.
// if use traversal, do not need shared reference, but is hard to communicate between each component.
// mouse event needs to specify layers, distributive calculation.
// pure tree -> component models with states (here we update the states) -> renderables
// length formula + dependencies -> actual length, manual override
// update actual length

// traverse twice to send mouse events.

// the first time gathers all elements that responds to the mouse event
// the second time notifies the one that wins the bid
// (actually, we hardly have any circumstances where we have competing elements)

// priority: bound > coop > manual override = preferred

// manual override will report a length, and the elements on the direction of the modification will be affected.
// the total least bound of all the elements will be calculated, if the least bound is greater than the target one, it will limit the manual override
// if the target length is less than the least bound, the changes will be applied to all the elements as uniformly as possible.

// so we have actual start point, actual end point, etc. preferred length, lower bound, upper bound, ...

// if one bound is not satisfied (actual length too small), then it will try to subtract length from other elements.
// other elements will report the actual length they have reduced

// only parents are allowed to change size
// child does not need to react to parents
// the split of a span can be moved around (min, max, preferred for each cell of a span)
// so, a span should have its elements uniformly distributed
// parents should never try to fit children
// if we want parents to fit children, we can set the length of the children to be the same as the parents
// if the children fails to fit, ...
// do not consider different resolution / screen size
