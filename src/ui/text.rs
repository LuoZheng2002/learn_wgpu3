use std::{any::TypeId, sync::{Arc, Mutex}};

use either::Either;

use crate::{cache::{get_font, CacheValue}, ui_node::{BoundedLength, BoxDimensionsRelative, ComponentIdentifier, HorizontalAlignment, RelativeLength, StructuredChildren, ToUINode, UIIdentifier, UINode, UINodeEventProcessed, VerticalAlignment, UI_IDENTIFIER_MAP}, ui_renderable::TextureMeta};

use super::ui_char::{CharCursor, UIChar};


#[derive(Clone, Copy, Debug)]
pub enum CharEvent{
    LeftPartClicked,
    RightPartClicked,
}


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
    pub char_event_callback: Arc<dyn Fn(u64, CharEvent)>,
}

pub struct TextState {
    pub state_changed: bool,
    pub change_parent_state: bool,
    pub version: u64,
    pub text: Vec<(char, UIChar)>,
    pub dummy_ui_char: DummyUIChar,
    pub pending_char_event: Option<(u64, CharEvent)>,
    pub self_clicked: bool,
}

impl TextState{
    pub fn set_text(&mut self, text: String, font_path: String, scale: f32, char_event_callback: Arc<dyn Fn(u64, CharEvent)>) {
        println!("Set text: {}", text);
        self.text = Text::chars_to_nodes(text, font_path, scale, char_event_callback);
        self.state_changed = true;
        self.change_parent_state = true;
    }
    pub fn stop_blinking_all(&mut self){
        for (_, char) in self.text.iter_mut() {
            let mut char_state = char.char_state.lock().unwrap();
            char_state.blinking = false;
            char_state.showing_cursor = false;
        }
        let mut dummy_ui_char_state = self.dummy_ui_char.char_state.lock().unwrap();
        dummy_ui_char_state.blinking = false;
        dummy_ui_char_state.showing_cursor = false;
    }
    pub fn start_blinking_one(&mut self, index: u64){
        assert!(index <= self.text.len() as u64, "Index out of bounds");
        if index == self.text.len() as u64 {            
            let mut char_state = self.dummy_ui_char.char_state.lock().unwrap();
            char_state.blinking = true;
            char_state.showing_cursor = true;
        }else{ // index < self.text.len() as u64
            let ui_char = &self.text.get(index as usize).as_ref().unwrap().1;
            let mut char_state = ui_char.char_state.lock().unwrap();
            char_state.blinking = false;
            char_state.showing_cursor = false;
        }
    }
}

impl Text {
    fn chars_to_nodes(
        s: String, 
        font_path: String, 
        scale: f32, 
        char_event_callback: Arc<dyn Fn(u64, CharEvent)>,
    ) -> Vec<(char, UIChar)> {
        s.chars()
        .enumerate()
            .map(|(index, c)| {
                let char = UIChar::new(c, font_path.clone(), scale, Arc::downgrade(&char_event_callback), index as u64);
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
        
        let id = UI_IDENTIFIER_MAP
            .lock()
            .unwrap()
            .next_id(TypeId::of::<Text>());
        let id = UIIdentifier::Component(ComponentIdentifier::Default{
            id,
            name: format!("Text"),
        });
        let dummy_ui_char = DummyUIChar::new(font_path.clone(), scale);
        let text_state = Arc::new(Mutex::new(TextState {
            state_changed: false,
            change_parent_state: false,
            version: 0,
            text: Vec::new(),
            dummy_ui_char,
            pending_char_event: None,
            self_clicked: false,
        }));        
        let char_event_callback = {
            let text_state = Arc::downgrade(&text_state);
            move |index: u64, event: CharEvent|{
                let text_state = text_state.upgrade().unwrap();
                let mut text_state = text_state.lock().unwrap();
                text_state.pending_char_event = Some((index, event));
            }
        };
        let char_event_callback = Arc::new(char_event_callback);
        let text = Self::chars_to_nodes(initial_text, font_path.clone(), scale, char_event_callback.clone());
        {
            let mut text_state = text_state.lock().unwrap();
            text_state.text = text;
        }
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
            char_event_callback,
        }
    }
    pub fn set_text(&mut self, text: String) {
        let mut text_state = self.text_state.lock().unwrap();
        text_state.text = Self::chars_to_nodes(text, self.font_path.clone(), self.scale, self.char_event_callback.clone());
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
            let event_handler = move |event: &UINodeEventProcessed|->bool {
                let mut text_state = text_state.lock().unwrap();
                if event.lose_focus{
                    text_state.stop_blinking_all();
                    text_state.state_changed = true;
                    text_state.change_parent_state = true;
                }
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
            };
            Some(Box::new(state_changed_handler) as Box<dyn Fn()>)
        };
        let mut text_state = self.text_state.lock().unwrap();
        // if there is a pending char event and it is not self clicked
        let mut cursor_position: Option<u64> = None;
        if let Some((index, event)) = &text_state.pending_char_event {
            if !text_state.self_clicked {
                let pos = match event {
                    CharEvent::LeftPartClicked => *index,
                    CharEvent::RightPartClicked => *index + 1,
                };
                cursor_position = Some(pos);
            }
        }else if text_state.self_clicked {
            cursor_position = Some(text_state.text.len() as u64);
        }
        if let Some(index) = cursor_position {
            text_state.stop_blinking_all();
            text_state.start_blinking_one(index);
        }
        text_state.pending_char_event = None;
        text_state.self_clicked = false;
        if text_state.state_changed {
            text_state.state_changed = false;
            text_state.version += 1;
        }
        let mut children_ui_nodes = text_state
            .text
            .iter()
            .map(|(_, char)| char.to_ui_node())
            .collect::<Vec<_>>();
        
        let dummy_ui_node = text_state.dummy_ui_char.to_ui_node();
        children_ui_nodes.push(dummy_ui_node);
        

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
pub struct DummyUIChar{
    pub font_path: String,
    pub scale: f32,
    pub char_state: Arc<Mutex<DummyUICharState>>,
}
pub struct DummyUICharState{
    pub blinking: bool,
    pub showing_cursor: bool,
}

impl DummyUIChar{
    pub fn new(font_path: String, scale: f32) -> Self {
        let char_state = Arc::new(Mutex::new(DummyUICharState {
            blinking: true,
            showing_cursor: false,
        }));
        Self {
            font_path,
            scale,
            char_state,
        }
    }
    pub fn get_id(&self) -> UIIdentifier {
        let char_state = self.char_state.lock().unwrap();
        let show_cursor = char_state.blinking && char_state.showing_cursor;
        UIIdentifier::Component(ComponentIdentifier::DummyChar { 
            show_cursor
        })
    }
}

impl ToUINode for DummyUIChar {
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
        let height = (ascent - descent) as u32;
        let width = height;

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
        let id = self.get_id();
        let show_cursor = match id{
            UIIdentifier::Component(ComponentIdentifier::DummyChar { show_cursor }) => show_cursor,
            _ => unreachable!(),
        };
        let children: StructuredChildren<BoxDimensionsRelative> = match show_cursor {
            true => {
                let child = CharCursor{};
                let child_ui_node = child.to_ui_node();
                StructuredChildren::OneChild { 
                h_alignment: HorizontalAlignment::Left, 
                v_alignment: VerticalAlignment::Top, 
                child: Box::new(child_ui_node),
            }
            },
            false => StructuredChildren::NoChildren,
        };
        let event_handler = {
            let char_state = Arc::downgrade(&self.char_state);
            let event_handler = move |event: &crate::ui_node::UINodeEventProcessed|->bool {
                let mut change_parent_state = false;
                if event.cursor_blink{                    
                    let char_state = char_state.upgrade().unwrap();
                    let mut char_state = char_state.lock().unwrap();
                    if char_state.blinking{
                        char_state.showing_cursor = !char_state.showing_cursor;
                        change_parent_state = true;
                        println!("DummyUIChar blinking");
                    }
                }
                change_parent_state
            };
            Some(Box::new(event_handler) as Box<dyn Fn(&crate::ui_node::UINodeEventProcessed)->bool>)
        };
        UINode {
            box_dimensions,
            children,
            meta: TextureMeta::Font {
                character: ' '.into(),
                font_path: self.font_path.clone(),
            },
            identifier: id,
            version: 0,
            event_handler,
            state_changed_handler: None,
        }
    }
}