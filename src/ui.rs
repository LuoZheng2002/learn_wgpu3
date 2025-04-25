// game engine user's interface:
// layout information
// an element can have: zero children, one child, or many children

use std::{any::TypeId, sync::{Arc, Mutex}};

use either::Either;
use wgpu::naga::back::spv;
use winit::event;

use crate::{
    cache::{get_font, CacheValue},
    ui_node::{
        self, BoundedLength, BoxDimensionsRelative, HorizontalAlignment, RelativeLength, StructuredChildren, UIIdentifier, UINode, UINodeEventProcessed, UINodeEventRaw, VerticalAlignment, UI_IDENTIFIER_MAP
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

pub struct Cell;

pub struct Text {    
    pub font_path: String,
    pub scale: f32,
    pub margin: [RelativeLength; 4],
    pub padding: [RelativeLength; 4],
    pub color: cgmath::Vector4<f32>,
    pub width: BoundedLength,
    pub height: BoundedLength,
    pub id: UIIdentifier,
    pub text_state: Arc<Mutex<TextState>>,
}

pub struct TextState {
    pub state_changed: bool,
    pub change_parent_state: bool,
    pub version: u64,
    pub text: Vec<(char, Char)>,
}

impl TextState{
    pub fn set_text(&mut self, text: String, font_path: String, scale: f32) {
        self.text = Text::chars_to_nodes(text, font_path, scale);
        self.state_changed = true;
        self.change_parent_state = true;
    }
}

impl Text {
    fn chars_to_nodes(s: String, font_path: String, scale: f32) -> Vec<(char, Char)> {
        s.chars()
            .map(|c| {
                let char = Char::new(c, font_path.clone(), scale);
                (c, char)
            })
            .collect::<Vec<_>>()
    }
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
        let text = Self::chars_to_nodes(initial_text, font_path.clone(), scale);
        let id = UI_IDENTIFIER_MAP
            .lock()
            .unwrap()
            .next_id(TypeId::of::<Text>());
        let id = UIIdentifier {
            id,
            name: format!("Text"),
        };
        let text_state = Arc::new(Mutex::new(TextState {
            state_changed: false,
            change_parent_state: false,
            version: 0,
            text,
        }));
        Self {
            font_path,
            scale,
            margin,
            padding,
            color,
            width,
            height,
            id,
            text_state,
        }
    }
    pub fn set_text(&mut self, text: String) {
        let mut text_state = self.text_state.lock().unwrap();
        text_state.text = Self::chars_to_nodes(text, self.font_path.clone(), self.scale);
        text_state.state_changed = true;
    }
}

pub struct Char {
    pub character: char,
    pub font_path: String,
    pub scale: f32,
    pub id: UIIdentifier,
}

impl Char {
    pub fn new(character: char, font_path: String, scale: f32) -> Self {
        let id = UI_IDENTIFIER_MAP
            .lock()
            .unwrap()
            .next_id(TypeId::of::<Char>());
        let id = UIIdentifier {
            id: id,
            name: format!("Char({})", character),
        };
        Self {
            character,
            font_path,
            scale,
            id,
        }
    }
}

impl ToUINode for Char {
    fn to_ui_node(
        &self,
    ) -> UINode<BoxDimensionsRelative, StructuredChildren<BoxDimensionsRelative>> {
        let font = get_font(self.font_path.clone());
        let font = match font.as_ref() {
            CacheValue::Font(font) => font,
            _ => panic!("Font not found"),
        };
        let scale = rusttype::Scale::uniform(self.scale);
        let v_metrics = font.v_metrics(scale);
        // round ascent to the nearest integer
        let ascent = v_metrics.ascent.round() as i32;
        let descent = v_metrics.descent.round() as i32;
        let line_gap = v_metrics.line_gap.round() as u32;
        let glyph = font.glyph(self.character).scaled(scale);
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
            identifier: self.id.clone(),
            version: 0,
            event_handler: None,
            state_changed_handler: None,
        }
    }
}

impl ToUINode for Text {
    fn to_ui_node(
        &self,
    ) -> UINode<BoxDimensionsRelative, StructuredChildren<BoxDimensionsRelative>> {
        
        let box_dimensions = BoxDimensionsRelative {
            width: self.width.clone(),
            height: self.height.clone(),
            margin: self.margin.clone(),
            padding: self.padding.clone(),
        };
        let event_handler = {
            let text_state = self.text_state.clone();
            let event_handler = move |_event: &UINodeEventProcessed|->bool {
                let mut text_state = text_state.lock().unwrap();
                let change_parent_state = text_state.change_parent_state;
                text_state.change_parent_state = false;
                change_parent_state
            };
            Some(Box::new(event_handler) as Box<dyn Fn(&UINodeEventProcessed)->bool>)
        };

        let state_changed_handler = {
            let text_state = self.text_state.clone();
            let state_changed_handler = move ||{
                let mut text_state = text_state.lock().unwrap();
                text_state.state_changed = true;
                println!("Text state changed");
            };
            Some(Box::new(state_changed_handler) as Box<dyn Fn()>)
        };
        let mut text_state = self.text_state.lock().unwrap();
        if text_state.state_changed {
            text_state.state_changed = false;
            text_state.version += 1;
        }
        let children_ui_nodes = text_state
            .text
            .iter()
            .map(|(_, char)| char.to_ui_node())
            .collect::<Vec<_>>();

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
            identifier: self.id.clone(),
            version: text_state.version,
            event_handler,
            state_changed_handler,
        }
    }
}

pub struct Button {
    pub box_dimensions: BoxDimensionsRelative,
    child: Option<Box<dyn ToUINode>>,
    pub id: UIIdentifier,
    pub button_state: Arc<Mutex<ButtonState>>,
    pub click_callback: Option<Arc<dyn Fn()>>,
}

// callback function

pub struct ButtonState{
    pub hovered: bool,
    pub clicked: bool,
    pub state_changed: bool,
    pub version: u64,
}

impl Button {
    pub fn new(
        width: BoundedLength,
        height: BoundedLength,
        margin: Either<RelativeLength, [RelativeLength; 4]>,
        padding: Either<RelativeLength, [RelativeLength; 4]>,
        click_callback: Option<Box<dyn Fn()>>,
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
        let id = UI_IDENTIFIER_MAP
            .lock()
            .unwrap()
            .next_id(TypeId::of::<Button>());
        let id = UIIdentifier {
            id,
            name: format!("Button"),
        };
        let button_state = Arc::new(Mutex::new(ButtonState {
            hovered: false,
            clicked: false,
            state_changed: false,
            version: 0,
        }));
        let click_callback = match click_callback {
            Some(callback) => Some(Arc::new(callback) as Arc<dyn Fn()>),
            None => None,
        };
        Self {
            box_dimensions,
            child: None,
            id,
            button_state,
            click_callback,
        }
    }
    pub fn set_child(&mut self, child: Box<dyn ToUINode>) {
        self.child = Some(child);
    }
    pub fn handle_event(&mut self, event: &UINodeEventRaw) {
        
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
        let event_handler = {
            let button_state = self.button_state.clone();
            let click_callback = self.click_callback.clone();
            let event_handler = move |event: &UINodeEventProcessed|->bool {
                let mut button_state = button_state.lock().unwrap();
                let prev_button_clicked = button_state.clicked;
                let prev_button_hovered = button_state.hovered;
                if event.left_clicked_inside{
                    button_state.clicked = true;
                    if let Some(callback) = click_callback.as_ref() {
                        callback();
                    }
                } else if event.left_released {
                    button_state.clicked = false;
                }
                if event.mouse_hover{
                    button_state.hovered = true;
                } else {
                    button_state.hovered = false;
                }
                button_state.clicked != prev_button_clicked || button_state.hovered != prev_button_hovered
            };
            Some(Box::new(event_handler) as Box<dyn Fn(&UINodeEventProcessed)->bool>)
        };
        let state_changed_handler = {
            let button_state = self.button_state.clone();
            let state_changed_handler = move ||{
                let mut button_state = button_state.lock().unwrap();
                button_state.state_changed = true;
                println!("Button state changed");
            };
            Some(Box::new(state_changed_handler) as Box<dyn Fn()>)
        };
        let mut button_state = self.button_state.lock().unwrap();        
        let meta = if button_state.clicked {
            TextureMeta::Texture {
                path: "assets/button3.jpg".into(),
            }}
            else if button_state.hovered{
                TextureMeta::Texture {
                    path: "assets/button2.jpg".into(),
                }
            }else{
                TextureMeta::Texture {
                    path: "assets/button.jpg".into(),
            }
        };
        if button_state.state_changed {
            button_state.state_changed = false;
            button_state.version += 1;
        }
        UINode {
            box_dimensions: self.box_dimensions.clone(),
            children,
            meta,
            identifier: self.id.clone(),
            version: button_state.version,
            event_handler,
            state_changed_handler,
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
    pub texture: TextureMeta,
    pub id: UIIdentifier,
    pub span_state: Arc<Mutex<SpanState>>,
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
        texture: TextureMeta,
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
        let id = UI_IDENTIFIER_MAP
            .lock()
            .unwrap()
            .next_id(TypeId::of::<Span>());
        let id = UIIdentifier {
            id,
            name: format!("Span"),
        };
        let span_state = Arc::new(Mutex::new(SpanState {
            state_changed: false,
            version: 0,
        }));
        Self {
            direction,
            children: Vec::new(),
            box_dimensions,
            h_alignment,
            v_alignment,
            uniform_division,
            texture,
            id,
            span_state,
        }
    }
    pub fn push_child(&mut self, child: Box<dyn ToUINode>) {
        self.children.push(child);
    }
}

pub struct SpanState{
    pub state_changed: bool,
    pub version: u64,
}

impl ToUINode for Span {
    fn to_ui_node(
        &self,
    ) -> UINode<BoxDimensionsRelative, StructuredChildren<BoxDimensionsRelative>> {
        let state_changed_handler = {
            let span_state = self.span_state.clone();
            let state_changed_handler = move ||{
                let mut span_state = span_state.lock().unwrap();
                span_state.state_changed = true;
                println!("Span state changed");
            };
            Some(Box::new(state_changed_handler) as Box<dyn Fn()>)
        };
        let mut span_state = self.span_state.lock().unwrap();
        if span_state.state_changed {
            span_state.state_changed = false;
            span_state.version += 1;
        }

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
            meta: self.texture.clone(),
            identifier: self.id.clone(),
            version: span_state.version,
            event_handler: None,
            state_changed_handler,
        }
    }
}
