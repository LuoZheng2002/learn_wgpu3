// game engine user's interface:
// layout information
// an element can have: zero children, one child, or many children

use either::Either;
use rusttype::{Font, Point, Rect, point};

use crate::{
    cache::{CacheValue, get_font},
    ui_node::{
        self, BoundedLength, BoxDimensionsRelative, HorizontalAlignment, RelativeLength,
        StructuredChildren, UINode, VerticalAlignment,
    },
    ui_renderable::TextureMeta,
};

pub trait ToUINode {
    fn to_ui_node(
        &self,
    ) -> UINode<BoxDimensionsRelative, StructuredChildren<BoxDimensionsRelative>>;
}

pub enum Children {
    NoChildren,
    OneChild(Box<dyn ToUINode>),
    HorizontalLayout(Vec<Box<dyn ToUINode>>),
    // HorizontalWrap(Vec<Box<dyn ToUINode>>),
    VerticalLayout(Vec<Box<dyn ToUINode>>),
    GridLayout(Vec<Vec<Box<dyn ToUINode>>>),
}

// // ui root
// pub struct Canvas{
//     pub padding: [CanvasPadding; 4],
//     pub screen_width: i32,
//     pub screen_height: i32,
//     pub child: Option<Box<dyn ToUINode>>,
// }

pub struct Text {
    pub text: String,
    pub font_path: String,
    pub scale: f32,
    pub margin: [RelativeLength; 4],
    pub padding: [RelativeLength; 4],
    pub color: cgmath::Vector4<f32>,
    pub width: BoundedLength,
    pub height: BoundedLength,
}

impl Text {
    pub fn new(
        initial_text: String,
        font_path: String,
        scale: f32,
        margin: Either<RelativeLength, [RelativeLength; 4]>,
        padding: Either<RelativeLength, [RelativeLength; 4]>,
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
    fn to_ui_node(
        &self,
    ) -> UINode<BoxDimensionsRelative, StructuredChildren<BoxDimensionsRelative>> {
        let scale = rusttype::Scale::uniform(self.scale);
        let v_metrics = self.font.v_metrics(scale);
        // round ascent to the nearest integer
        let ascent = v_metrics.ascent.round() as i32;
        let descent = v_metrics.descent.round() as i32;
        let line_gap = v_metrics.line_gap.round() as u32;
        let glyph = self.font.glyph(self.character).scaled(scale);
        let h_metrics = glyph.h_metrics();

        let advance_width = h_metrics.advance_width.round() as u32;
        // let margin_top = (ascent + bounding_top + line_gap/2) as u32; // ascent - (abs bounding_top)
        // let margin_bottom = -((descent + bounding_bottom) + line_gap/2) as u32; // bounding_bottom - (abs descent)
        let width = advance_width;
        let height = (ascent - descent) as u32;
        let box_dimensions = BoxDimensionsRelative {
            width: BoundedLength::fixed_pixels(width),
            height: BoundedLength::fixed_pixels(height),
            margin: [
                RelativeLength::Pixels(line_gap / 2),
                RelativeLength::Pixels(0),
                RelativeLength::Pixels(line_gap / 2),
                RelativeLength::Pixels(0),
            ],
            padding: [
                RelativeLength::Pixels(0),
                RelativeLength::Pixels(0),
                RelativeLength::Pixels(0),
                RelativeLength::Pixels(0),
            ],
        };
        UINode {
            box_dimensions,
            children: crate::ui_node::StructuredChildren::NoChildren,
            meta: TextureMeta::Font {
                character: self.character,
                font_path: self.font_path.clone(),
            },
        }
    }
}

impl ToUINode for Text {
    fn to_ui_node(
        &self,
    ) -> UINode<BoxDimensionsRelative, StructuredChildren<BoxDimensionsRelative>> {
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
            })
            .collect::<Vec<_>>();
        let box_dimensions = BoxDimensionsRelative {
            width: self.width.clone(),
            height: self.height.clone(),
            margin: self.margin.clone(),
            padding: self.padding.clone(),
        };
        UINode {
            box_dimensions,
            children: crate::ui_node::StructuredChildren::HorizontalLayout {
                h_alignment: ui_node::HorizontalAlignment::Left,
                v_alignment: ui_node::VerticalAlignment::Top,
                children: children_ui_nodes,
                uniform_division: false,
            },
            meta: TextureMeta::Texture {
                path: "assets/placeholder.png".into(),
            },
        }
    }
}

pub struct Button {
    pub box_dimensions: BoxDimensionsRelative,
    child: Option<Box<dyn ToUINode>>,
}

// callback function

impl Button {
    pub fn new(
        width: BoundedLength,
        height: BoundedLength,
        margin: Either<RelativeLength, [RelativeLength; 4]>,
        padding: Either<RelativeLength, [RelativeLength; 4]>,
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
        let box_dimensions = BoxDimensionsRelative {
            width,
            height,
            margin,
            padding,
        };
        Self {
            box_dimensions,
            child: None,
        }
    }
    pub fn set_child(&mut self, child: Box<dyn ToUINode>) {
        self.child = Some(child);
    }
}
impl ToUINode for Button {
    fn to_ui_node(
        &self,
    ) -> UINode<BoxDimensionsRelative, StructuredChildren<BoxDimensionsRelative>> {
        let children = match &self.child {
            Some(child) => {
                let child_ui_node = child.to_ui_node();
                crate::ui_node::StructuredChildren::OneChild {
                    h_alignment: HorizontalAlignment::Center,
                    v_alignment: VerticalAlignment::Center,
                    child: Box::new(child_ui_node),
                }
            }
            None => crate::ui_node::StructuredChildren::NoChildren,
        };
        UINode {
            box_dimensions: self.box_dimensions.clone(),
            children,
            meta: TextureMeta::Texture {
                path: "assets/placeholder.png".into(),
            },
        }
    }
}

pub enum SpanDirection {
    Horizontal,
    Vertical,
}

pub struct Span {
    pub direction: SpanDirection,
    pub children: Vec<Box<dyn ToUINode>>,
    pub box_dimensions: BoxDimensionsRelative,
    pub h_alignment: HorizontalAlignment,
    pub v_alignment: VerticalAlignment,
    pub uniform_division: bool,
}

impl Span {
    pub fn new(
        direction: SpanDirection,
        width: BoundedLength,
        height: BoundedLength,
        margin: Either<RelativeLength, [RelativeLength; 4]>,
        padding: Either<RelativeLength, [RelativeLength; 4]>,
        h_alignment: HorizontalAlignment,
        v_alignment: VerticalAlignment,
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
        let box_dimensions: BoxDimensionsRelative = BoxDimensionsRelative {
            width,
            height,
            margin,
            padding,
        };
        Self {
            direction,
            children: Vec::new(),
            box_dimensions,
            h_alignment,
            v_alignment,
            uniform_division,
        }
    }
    pub fn push_child(&mut self, child: Box<dyn ToUINode>) {
        self.children.push(child);
    }
}

impl ToUINode for Span {
    fn to_ui_node(
        &self,
    ) -> UINode<BoxDimensionsRelative, StructuredChildren<BoxDimensionsRelative>> {
        let children_ui_nodes = self
            .children
            .iter()
            .map(|c| c.to_ui_node())
            .collect::<Vec<_>>();
        UINode {
            box_dimensions: self.box_dimensions.clone(),
            children: match &self.direction {
                SpanDirection::Horizontal => StructuredChildren::HorizontalLayout {
                    h_alignment: self.h_alignment.clone(),
                    v_alignment: self.v_alignment.clone(),
                    uniform_division: self.uniform_division,
                    children: children_ui_nodes,
                },
                SpanDirection::Vertical => StructuredChildren::VerticalLayout {
                    h_alignment: self.h_alignment.clone(),
                    v_alignment: self.v_alignment.clone(),
                    uniform_division: self.uniform_division,
                    children: children_ui_nodes,
                },
            },
            meta: TextureMeta::Texture {
                path: "assets/placeholder.png".into(),
            },
        }
    }
}
