use std::{any::TypeId, sync::{Arc, Mutex}};

use either::Either;

use crate::{ui_node::{BoundedLength, BoxDimensionsRelative, HorizontalAlignment, RelativeLength, StructuredChildren, ToUINode, UIIdentifier, UINode, UINodeEventProcessed, VerticalAlignment, UI_IDENTIFIER_MAP}, ui_renderable::TextureMeta};

use super::ui_char::UIChar;

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
    pub text: Vec<(char, UIChar)>,
}

impl TextState{
    pub fn set_text(&mut self, text: String, font_path: String, scale: f32) {
        self.text = Text::chars_to_nodes(text, font_path, scale);
        self.state_changed = true;
        self.change_parent_state = true;
    }
}

impl Text {
    fn chars_to_nodes(s: String, font_path: String, scale: f32) -> Vec<(char, UIChar)> {
        s.chars()
            .map(|c| {
                let char = UIChar::new(c, font_path.clone(), scale);
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
            children: StructuredChildren::HorizontalLayout {
                h_alignment: HorizontalAlignment::Left,
                v_alignment:VerticalAlignment::Top,
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
