// game engine user's interface:
// layout information
// an element can have: zero children, one child, or many children

use rusttype::{Font, point};

use crate::{
    cache::{CacheValue, get_font},
    ui_node::UINode,
    ui_renderable::UIRenderableMeta,
};

pub trait ToUINode {
    fn to_ui_node(&self) -> UINode;
}

#[derive(Default, Clone)]
pub struct LayoutInfo {}

pub enum Children {
    NoChildren,
    OneChild(Box<dyn ToUINode>),
    HorizontalLayout(Vec<Box<dyn ToUINode>>),
    // HorizontalWrap(Vec<Box<dyn ToUINode>>),
    VerticalLayout(Vec<Box<dyn ToUINode>>),
    GridLayout(Vec<Vec<Box<dyn ToUINode>>>),
}

pub struct Span {
    pub children: Children,
    pub layout_info: LayoutInfo,
}

pub struct Text {
    pub initial_text: String,
    pub font_path: String,
    pub scale: f32,
    pub layout_info: LayoutInfo,
}

impl Text {
    pub fn new(initial_text: String, font_path: String, scale: f32) -> Self {
        Self {
            initial_text,
            font_path,
            scale,
            layout_info: LayoutInfo::default(),
        }
    }
}

pub struct Char<'a> {
    pub character: char,
    pub font: &'a Font<'static>,
    pub scale: f32,
}

impl<'a> ToUINode for Char<'a> {
    fn to_ui_node(&self) -> UINode {
        let scale = rusttype::Scale::uniform(self.scale);
        let v_metrics = self.font.v_metrics(scale);
        // round ascent to the nearest integer
        let ascent = v_metrics.ascent.round() as i32;
        let descent = v_metrics.descent.round() as i32;
        let line_gap = v_metrics.line_gap.round() as i32;
        let height = ascent - descent + line_gap; // ascent + (abs descent) + line_gap
        let glyph = self.font.glyph(self.character).scaled(scale);
        let h_metrics = glyph.h_metrics();
        let left_side_bearing = h_metrics.left_side_bearing.round() as i32;
        println!("left side bearing: {}", left_side_bearing);
        let advance_width = h_metrics.advance_width.round() as i32;
        println!("advance width: {}", advance_width);

        let glyph = glyph.positioned(point(0.0, 0.0));
        let bounding_box = glyph.pixel_bounding_box().unwrap();
        let bounding_top = bounding_box.min.y;
        let bounding_bottom = bounding_box.max.y;
        let bounding_left = bounding_box.min.x;
        let bounding_right = bounding_box.max.x;
        let margin_top = ascent + bounding_top; // ascent - (abs bounding_top)
        let margin_bottom = -(descent + bounding_bottom); // -(bounding_bottom - (abs descent))
        assert!(margin_top >= 0, "margin top is negative: {}", margin_top);
        assert!(
            margin_bottom >= 0,
            "margin bottom is negative: {}",
            margin_bottom
        );
        let margin_left = left_side_bearing + bounding_left; // left_side_bearing - (abs bounding_left)
        let margin_right = advance_width - bounding_right; // advance_width - (abs bounding_right)

        println!("bounding top: {}", bounding_top);
        println!("bounding bottom: {}", bounding_bottom);
        println!("bounding left: {}", bounding_left);
        println!("bounding right: {}", bounding_right);
        let width = bounding_box.width() as i32;
        let height = bounding_box.height() as i32;
        UINode {
            width,
            height,
            children: crate::ui_node::Children::NoChildren,
            margin_top,
            margin_bottom,
            margin_left,
            margin_right,
            padding: 0,
            meta: UIRenderableMeta::Font {
                character: self.character,
            },
        }
    }
}

impl ToUINode for Text {
    fn to_ui_node(&self) -> UINode {
        let font = get_font(self.font_path.clone());
        let font = match font.as_ref() {
            CacheValue::Font(font) => font,
            _ => panic!("Font not found"),
        };
        let max_height_with_margin = self
            .initial_text
            .chars()
            .map(|c| {
                let c = Char {
                    character: c,
                    font: &font,
                    scale: self.scale,
                };
                let ui_node = c.to_ui_node();
                ui_node.height + ui_node.margin_top + ui_node.margin_bottom
            })
            .fold(0, i32::max);
        todo!()
        // UINode{
        //     width: 0,
        //     height: 0,
        //     children: Children::NoChildren,
        //     padding_left: 0,
        //     padding_right: 0,
        //     padding_top: 0,
        //     padding_bottom: 0,
        //     margin: 0,
        //     meta: UIRenderableMeta::Font{character: 'a'},
        // }
    }
}
