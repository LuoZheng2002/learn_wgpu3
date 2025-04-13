// length options: fixed, depends on siblings, depends on parent, depends on children

// decide parents first, then siblings, then children

// a span wrapping several padded characters

// padding, margin, children
// width, height,

use crate::ui_renderable::UIRenderableMeta;

pub enum Children {
    NoChildren,
    OneChild(Box<UINode>),
    HorizontalLayout(Vec<Box<UINode>>),
    // HorizontalWrap(Vec<Box<UINode>>),
    VerticalLayout(Vec<Box<UINode>>),
    GridLayout(Vec<Vec<Box<UINode>>>),
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
}

pub struct Button {}
