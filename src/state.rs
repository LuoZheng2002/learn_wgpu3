use std::{collections::HashMap, sync::{Arc, Mutex}, time::Instant};

use cgmath::{Euler, Quaternion};
use either::Either;

use crate::{
    input_context::InputContext, model_instance::ModelInstance, model_meta::ModelMeta, my_camera::MyCamera, ui::{ui_button::UIButton, ui_span::{UISpan, SpanDirection}, ui_text::{CharEvent, UIText, UITextInner}}, ui_node::{
        BoundedLength, HorizontalAlignment, RelativeLength, ToUINode, UINodeEventRaw, UIRenderInstruction, VerticalAlignment
    }, ui_renderable::TextureMeta
};

// model path,
pub struct State {
    // camera stuff
    pub camera: MyCamera,
    // accumulated time
    pub timer: Option<Instant>,
    pub prev_time: Option<f32>,
    pub fps_timer: Option<Instant>,
    pub cursor_timer: Option<Instant>,
    pub accumulated_frame_num: u32,
    pub model_render_submissions: HashMap<ModelMeta, Vec<ModelInstance>>,
    // use Arc here because we need to map the container to another container
    // pub ui_render_submissions: HashMap<TextureMeta, Vec<UIInstance>>,
    pub ui_render_instructions: Vec<UIRenderInstruction>,
    pub light_position: cgmath::Vector3<f32>,
    pub fps: u32,
    pub canvas: Option<UISpan>,
    pub text: Option<UIText>,
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
        let text = UIText::new(
            // format!("fps: {}", self.fps).to_string(),
            "fpsmnlk: 100".into(),
            "assets/consolas.ttf".to_string(),
            50.0,
            Either::Left(RelativeLength::Pixels(20)),
            Either::Left(RelativeLength::Pixels(20)),
            cgmath::Vector4 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
                w: 1.0,
            },
            BoundedLength::fixed_pixels(300),
            BoundedLength::fixed_pixels(200),
        );
        let text2 = UIText::new(
            // format!("fps: {}", self.fps).to_string(),
            "asdf/:?123".into(),
            "assets/consolas.ttf".to_string(),
            50.0,
            Either::Left(RelativeLength::Pixels(20)),
            Either::Left(RelativeLength::Pixels(20)),
            cgmath::Vector4 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
                w: 1.0,
            },
            BoundedLength::fixed_pixels(600),
            BoundedLength::fixed_pixels(200),
        );
        self.text = Some(text.clone());
        
        let button = UIButton::new(
            BoundedLength::fixed_pixels(300),
            BoundedLength::fixed_pixels(100),
            Either::Left(RelativeLength::Pixels(20)),
            Either::Left(RelativeLength::Pixels(20)),
            None,
        );
        let span = UISpan::new(
            SpanDirection::Horizontal,
            BoundedLength::fixed_dependent(RelativeLength::RelativeScreenWidth(0.9)),
            BoundedLength::fixed_dependent(RelativeLength::RelativeScreenHeight(0.9)),
            Either::Left(RelativeLength::Pixels(10)),
            Either::Left(RelativeLength::Pixels(10)),
            HorizontalAlignment::Left,
            VerticalAlignment::Top,
            false,
            TextureMeta::Texture {
                path: "assets/genshin.jpg".into(),
            },
        );
        // span.push_child(Box::new(span2));
        // span.push_child(Box::new(span3));
        span.push_child(Box::new(text));
        span.push_child(Box::new(button));
        span.push_child(Box::new(text2));
        // span.push_child(Box::new(button));
        self.canvas = Some(span);
    }

    pub fn update(&mut self, input_context: &mut InputContext, window_size: &winit::dpi::PhysicalSize<u32>) {
        // calculate fps every 1 second
        let fps_timer = self.fps_timer.get_or_insert_with(|| Instant::now());
        let cursor_timer = self.cursor_timer.get_or_insert_with(|| Instant::now());
        let current_fps_time = fps_timer.elapsed().as_secs_f32();
        if current_fps_time >= 1.0 {
            println!("FPS: {}", self.accumulated_frame_num);
            self.fps = self.accumulated_frame_num;
            self.accumulated_frame_num = 0;
            *fps_timer = Instant::now();
            let dummy_callback = |index: u64, event: CharEvent|{};
            let dummy_callback = Arc::new(dummy_callback);
            self.text.as_ref().unwrap().set_text(
                format!("FPS: {}", self.fps).to_string(),
            );
        } else {
            self.accumulated_frame_num += 1;
        }
        let current_cursor_time = cursor_timer.elapsed().as_secs_f32();
        let mut cursor_blink = false;
        if current_cursor_time >= 0.5 {
            // println!("cursor: {:?}", input_context.mouse_position());
            *cursor_timer = Instant::now();
            cursor_blink = true;
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
        // self.submit_renderable(model_meta.clone(), instance1);
        // self.submit_renderable(model_meta.clone(), instance2);

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
        let canvas = self.canvas.as_ref().unwrap();

        let cursor_position = input_context.mouse_position();
        let cursor_position = cursor_position.unwrap_or((0.0, 0.0));
        let pressed_str = input_context.get_pressed_str();
        let ui_node_event = UINodeEventRaw{
            mouse_x: cursor_position.0 as u32,
            mouse_y: cursor_position.1 as u32,
            mouse_left: input_context.mouse_left(),
            mouse_left_down: input_context.mouse_left_down(),
            mouse_left_up: input_context.mouse_left_up(),
            mouse_right: input_context.mouse_right(),
            mouse_right_down: input_context.mouse_right_down(),
            mouse_right_up: input_context.mouse_right_up(),
            key_down: input_context.get_current_key_down(),
            cursor_blink,
            pressed_str,
        };
        let render_instruction = canvas.update_and_to_instruction(screen_width, screen_height, &ui_node_event);
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
            cursor_timer: None,
            accumulated_frame_num: 0,
            model_render_submissions: HashMap::new(),
            // ui_render_submissions: HashMap::new(),
            ui_render_instructions: Vec::new(),
            light_position: cgmath::Vector3::new(0.0, 0.0, 0.0),
            fps: 0,
            canvas: None,
            text:None,
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
