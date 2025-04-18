// game engine user's interface:
// layout information
// an element can have: zero children, one child, or many children

use either::Either;
use rusttype::{point, Font, Point, Rect};

use crate::{
    cache::{get_font, CacheValue},
    ui_node::{self, BoxDimensions, BoxModel, CanvasPadding, BoundedLength, DependentLength, UINode},
    ui_renderable::UIRenderableMeta,
};

pub trait ToUINode {
    fn to_ui_node(&self) -> UINode<BoundedLength, DependentLength>;
}


pub enum Children {
    NoChildren,
    OneChild(Box<dyn ToUINode>),
    HorizontalLayout(Vec<Box<dyn ToUINode>>),
    // HorizontalWrap(Vec<Box<dyn ToUINode>>),
    VerticalLayout(Vec<Box<dyn ToUINode>>),
    GridLayout(Vec<Vec<Box<dyn ToUINode>>>),
}





// ui root
pub struct Canvas{
    pub padding: [CanvasPadding; 4],
    pub screen_width: i32,
    pub screen_height: i32,
    pub child: Option<Box<dyn ToUINode>>,
}

pub struct Text {
    pub text: String,
    pub font_path: String,
    pub scale: f32,
    pub margin: [DependentLength; 4],
    pub padding: [DependentLength; 4],
    pub color: cgmath::Vector4<f32>,
    pub width: BoundedLength,
    pub height: BoundedLength,
}




impl Text {
    pub fn new(initial_text: String,
        font_path: String,
        scale: f32,
        margin: Either<DependentLength, [DependentLength; 4]>,
        padding: Either<DependentLength, [DependentLength; 4]>,
        color: cgmath::Vector4<f32>,
        width: BoundedLength,
        height: BoundedLength,
    ) -> Self {
        let margin = match margin {
            Either::Left(m) => [m.clone(), m.clone(), m.clone(), m.clone()],
            Either::Right(m) => m,
        };
        let padding = match padding {
            Either::Left(p) => [p.clone(), p.clone(), p.clone(), p.clone()],
            Either::Right(p) => p,
        };
        Self {
            text: initial_text,
            font_path,
            scale,
            margin,
            padding,
            color,
            width,
            height,
        }
    }
    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }
}

pub struct Char<'a> {
    pub character: char,
    pub font_path: String,
    pub font: &'a Font<'static>,
    pub scale: f32,
}

impl<'a> ToUINode for Char<'a> {
    fn to_ui_node(&self) -> UINode<BoundedLength, DependentLength> {
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
        let width = bounding_box.width() as i32;
        let height = bounding_box.height() as i32;
        let box_dimensions: BoxDimensions<BoundedLength, DependentLength> = BoxDimensions {
            width: BoundedLength::fixed_pixels(width),
            height: BoundedLength::fixed_pixels(height),
            margin: [
                DependentLength::Pixels(margin_top),
                DependentLength::Pixels(margin_right),
                DependentLength::Pixels(margin_bottom),
                DependentLength::Pixels(margin_left),
            ],
            padding: [
                DependentLength::Pixels(0),
                DependentLength::Pixels(0),
                DependentLength::Pixels(0),
                DependentLength::Pixels(0),
            ],
        };
        let box_model: BoxModel<BoundedLength, DependentLength> = BoxModel {
            dimensions: box_dimensions,
            h_alignment: ui_node::HorizontalAlignment::Left,
            v_alignment: ui_node::VerticalAlignment::Top,
            color: cgmath::Vector4::new(0.0, 0.0, 0.0, 1.0),
            uniform_division: false,
        };
        UINode {
            box_model,
            children: crate::ui_node::Children::NoChildren,
            meta: UIRenderableMeta::Font {
                character: self.character,
                font_path: self.font_path.clone(),
            },
        }
    }
}

impl ToUINode for Text {
    fn to_ui_node(&self) -> UINode<BoundedLength, DependentLength> {
        let font = get_font(self.font_path.clone());
        let font = match font.as_ref() {
            CacheValue::Font(font) => font,
            _ => panic!("Font not found"),
        };
        let children_ui_nodes = self
            .text
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
        let box_dimensions: BoxDimensions<BoundedLength, DependentLength> = BoxDimensions {
            width: self.width.clone(),
            height: self.height.clone(),
            margin: self.margin.clone(),
            padding: self.padding.clone(),
        };
        let box_model = BoxModel {
            dimensions: box_dimensions,
            h_alignment: ui_node::HorizontalAlignment::Left,
            v_alignment: ui_node::VerticalAlignment::Top,
            color: self.color,
            uniform_division: false,
        };
        UINode{
            box_model,
            children: crate::ui_node::Children::HorizontalLayout(
                children_ui_nodes
            ),
            meta: UIRenderableMeta::Color,
        }
    }
}

pub struct Button{
    pub box_model: BoxModel<BoundedLength, DependentLength>,
    child: Option<Box<dyn ToUINode>>,
}

// callback function




impl Button{
    pub fn new(
        width: BoundedLength,
        height: BoundedLength,
        margin: Either<DependentLength, [DependentLength; 4]>,
        padding: Either<DependentLength, [DependentLength; 4]>,
        color: cgmath::Vector4<f32>,
    ) -> Self {
        let margin = match margin {
            Either::Left(m) => [m.clone(), m.clone(), m.clone(), m.clone()],
            Either::Right(m) => m,
        };
        let padding = match padding {
            Either::Left(p) => [p.clone(), p.clone(), p.clone(), p.clone()],
            Either::Right(p) => p,
        };
        let box_dimensions: BoxDimensions<BoundedLength, DependentLength> = BoxDimensions {
            width,
            height,
            margin,
            padding,
        };
        let box_model = BoxModel {
            dimensions: box_dimensions,
            h_alignment: ui_node::HorizontalAlignment::Center,
            v_alignment: ui_node::VerticalAlignment::Center,
            color,
            uniform_division: false,
        };
        Self { 
            box_model,
            child: None,
        }
    }
    pub fn set_child(&mut self, child: Box<dyn ToUINode>) {
        self.child = Some(child);
    }
}
impl ToUINode for Button {
    fn to_ui_node(&self) -> UINode<BoundedLength, DependentLength> {
        let children = match &self.child{
            Some(child) => {
                let child_ui_node = child.to_ui_node();
                crate::ui_node::Children::OneChild(Box::new(child_ui_node))
            }
            None => crate::ui_node::Children::NoChildren,
        };
        UINode{
            box_model: self.box_model.clone(),
            children,
            meta: UIRenderableMeta::Color,
        }
    }
}

pub enum SpanDirection{
    Horizontal,
    Vertical,
}

pub struct Span {
    pub direction: SpanDirection,
    pub children: Vec<Box<dyn ToUINode>>,
    pub box_model: BoxModel<BoundedLength, DependentLength>,
}

impl Span{
    pub fn new(
        direction: SpanDirection,
        width: BoundedLength,
        height: BoundedLength,
        margin: Either<DependentLength, [DependentLength; 4]>,
        padding: Either<DependentLength, [DependentLength; 4]>,
        color: cgmath::Vector4<f32>,
        uniform_division: bool,
    ) -> Self {
        let margin = match margin {
            Either::Left(m) => [m.clone(), m.clone(), m.clone(), m.clone()],
            Either::Right(m) => m,
        };
        let padding = match padding {
            Either::Left(p) => [p.clone(), p.clone(), p.clone(), p.clone()],
            Either::Right(p) => p,
        };
        let box_dimensions: BoxDimensions<BoundedLength, DependentLength> = BoxDimensions {
            width,
            height,
            margin,
            padding,
        };
        let box_model = BoxModel {
            dimensions: box_dimensions,
            h_alignment: ui_node::HorizontalAlignment::Center,
            v_alignment: ui_node::VerticalAlignment::Center,
            color,
            uniform_division,
        };
        Self { 
            direction,
            children: Vec::new(),
            box_model,
        }
    }
    pub fn push_child(&mut self, child: Box<dyn ToUINode>) {
        self.children.push(child);
    }
}

impl ToUINode for Span {
    fn to_ui_node(&self) -> UINode<BoundedLength, DependentLength> {
        let children_ui_nodes = self
            .children
            .iter()
            .map(|c| c.to_ui_node())
            .collect::<Vec<_>>();
        UINode{
            box_model: self.box_model.clone(),
            children: match &self.direction{
                SpanDirection::Horizontal => crate::ui_node::Children::HorizontalLayout(
                    children_ui_nodes
                ),
                SpanDirection::Vertical => crate::ui_node::Children::VerticalLayout(
                    children_ui_nodes
                ),
            },
            meta: UIRenderableMeta::Color,
        }
    }
}