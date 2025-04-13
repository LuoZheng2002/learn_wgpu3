// game engine user's interface:
// layout information
// an element can have: zero children, one child, or many children

use rusttype::{point, Font, Point, Rect};

use crate::{
    cache::{get_font, CacheValue},
    ui_node::{self, UINode},
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
    pub padding: i32,
    pub color: cgmath::Vector4<f32>,
}

impl Text {
    pub fn new(initial_text: String, font_path: String, scale: f32, padding: i32, color: cgmath::Vector4<f32>) -> Self {
        Self {
            initial_text,
            font_path,
            scale,
            layout_info: LayoutInfo::default(),
            padding,
            color,
        }
    }
}

pub struct Char<'a> {
    pub character: char,
    pub font_path: String,
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
        
        let glyph = self.font.glyph(self.character).scaled(scale);
        let h_metrics = glyph.h_metrics();
        let left_side_bearing = h_metrics.left_side_bearing.round() as i32;
        
        let advance_width = h_metrics.advance_width.round() as i32;
        
        let glyph = glyph.positioned(point(0.0, 0.0));
        let bounding_box = glyph.pixel_bounding_box().unwrap_or(Rect { min: Point{x: 0, y: 0}, max: Point{x: 0, y: 0} });
        let bounding_top = bounding_box.min.y;
        let bounding_bottom = bounding_box.max.y;
        let bounding_left = bounding_box.min.x;
        let bounding_right = bounding_box.max.x;
        let margin_top = ascent + bounding_top + line_gap/2; // ascent - (abs bounding_top)
        let margin_bottom = -(descent + bounding_bottom) + line_gap/2; // bounding_bottom - (abs descent)
        assert!(margin_top >= 0, "margin top is negative: {}", margin_top);
        assert!(
            margin_bottom >= 0,
            "margin bottom is negative: {}",
            margin_bottom
        );
        let margin_left = left_side_bearing + bounding_left; // left_side_bearing - (abs bounding_left)
        let margin_right = advance_width - bounding_right; // advance_width - (abs bounding_right)
        println!("ascent: {}", ascent);
        println!("descent: {}", descent);
        println!("line gap: {}", line_gap);
        println!("left side bearing: {}", left_side_bearing);
        println!("advance width: {}", advance_width);
        println!("font char: {}", self.character);
        println!("bounding top: {}", bounding_top);
        println!("bounding bottom: {}", bounding_bottom);
        println!("bounding left: {}", bounding_left);
        println!("bounding right: {}", bounding_right);
        let width = bounding_box.width() as i32;
        let height = bounding_box.height() as i32;
        println!("width: {}", width);
        println!("height: {}", height);
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
                font_path: self.font_path.clone(),
            },
            color: cgmath::Vector4::new(0.0, 0.0, 0.0, 1.0),
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
        let children_ui_nodes = self
            .initial_text
            .chars()
            .map(|c| {
                let c = Char {
                    character: c,
                    font: &font,
                    font_path: self.font_path.clone(),
                    scale: self.scale,
                };
                c.to_ui_node()
            }).collect::<Vec<_>>();
        let max_height_with_margin = children_ui_nodes
            .iter()
            .map(|ui_node| {
                ui_node.height + ui_node.margin_top + ui_node.margin_bottom
            })
            .fold(0, i32::max);
        let width_sum_with_margin: i32 = children_ui_nodes
            .iter()
            .map(|ui_node| {
                ui_node.width + ui_node.margin_left + ui_node.margin_right
            })
            .sum();
        UINode{
            width: width_sum_with_margin + 2* self.padding,
            height: max_height_with_margin + 2* self.padding,
            children: crate::ui_node::Children::HorizontalLayout(children_ui_nodes),
            margin_top: 0,
            margin_bottom: 0,
            margin_left: 0,
            margin_right: 0,
            padding: self.padding,
            meta: UIRenderableMeta::Color,
            color: self.color,
        }
    }
}
