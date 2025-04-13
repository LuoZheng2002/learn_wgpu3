// length options: fixed, depends on siblings, depends on parent, depends on children

// decide parents first, then siblings, then children

// a span wrapping several padded characters

// padding, margin, children
// width, height,

use std::collections::HashMap;

use crate::ui_renderable::{UIInstance, UIRenderableMeta};

pub enum Children {
    NoChildren,
    OneChild(Box<UINode>),
    HorizontalLayout(Vec<UINode>),
    // HorizontalWrap(Vec<Box<UINode>>),
    VerticalLayout(Vec<UINode>),
    GridLayout(Vec<Vec<UINode>>),
}
pub struct UINode {
    pub width: i32,         // in pixels
    pub height: i32,        // in pixels
    pub children: Children, // assuming horizontal layout
    pub margin_left: i32,
    pub margin_right: i32,
    pub margin_top: i32,
    pub margin_bottom: i32,
    pub padding: i32,
    pub meta: UIRenderableMeta, // temp, should be element types
    pub color: cgmath::Vector4<f32>,
}

impl UINode{
    pub fn to_ui_renderables(&self, sort_order: u32, screen_width: u32, screen_height: u32, pos_x: i32, pos_y: i32) -> HashMap<UIRenderableMeta, Vec<UIInstance>>{
        let mut ui_renderables = HashMap::new();
        // current pivot
        let pos_x = pos_x + self.margin_left;
        let pos_y = pos_y + self.margin_top;
        let normalized_x_start = (pos_x as f32 / screen_width as f32) * 2.0 - 1.0;
        let normalized_y_start = (pos_y as f32 / screen_height as f32) * 2.0 - 1.0;
        let normalized_width = (self.width as f32 / screen_width as f32) * 2.0;
        let normalized_height = (self.height as f32 / screen_height as f32) * 2.0;
        let normalized_x_end = normalized_x_start + normalized_width;
        let normalized_y_end = normalized_y_start + normalized_height;
        // flip y
        let normalized_y_start = - normalized_y_start;
        let normalized_y_end = - normalized_y_end;
        let ui_instance = UIInstance {
            location: [
                normalized_x_start,
                normalized_y_start,
                normalized_x_end,
                normalized_y_end,
            ],
            color: self.color,
            sort_order,
            use_texture: self.meta.uses_texture(),
        };
        ui_renderables.entry(self.meta.clone()).or_insert_with(Vec::new).push(ui_instance);
        match self.children {
            Children::NoChildren => {}
            Children::OneChild(ref child) => {
                let child_renderables = child.to_ui_renderables(sort_order + 1, screen_width, screen_height, pos_x + self.padding, pos_y + self.padding);
                for (meta, instances) in child_renderables {
                    ui_renderables.entry(meta).or_insert_with(Vec::new).extend(instances);
                }
            }
            Children::HorizontalLayout(ref children) => {
                let mut pos_x = pos_x + self.padding;
                let pos_y = pos_y + self.padding;
                for child in children {
                    let child_renderables = child.to_ui_renderables(sort_order + 1, screen_width, screen_height, pos_x, pos_y);
                    for (meta, instances) in child_renderables {
                        ui_renderables.entry(meta).or_insert_with(Vec::new).extend(instances);
                    }
                    pos_x += child.width + child.margin_left + child.margin_right;
                }
            }
            Children::VerticalLayout(ref children) => {
                let pos_x = pos_x + self.padding;
                let mut pos_y = pos_y + self.padding;
                for child in children {
                    let child_renderables = child.to_ui_renderables(sort_order + 1, screen_width, screen_height, pos_x, pos_y);
                    for (meta, instances) in child_renderables {
                        ui_renderables.entry(meta).or_insert_with(Vec::new).extend(instances);
                    }
                    pos_y += child.height + child.margin_top + child.margin_bottom;
                }
            }
            Children::GridLayout(ref children) => {
                todo!()
            }
        }
        ui_renderables
    }
}

pub struct Button {}
