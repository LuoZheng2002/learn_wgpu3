use std::{any::TypeId, sync::{Arc, Mutex, RwLock}};

use either::Either;
use winit::keyboard::KeyCode;

use crate::{cache::{get_font, CacheValue}, ui_node::{BoundedLength, BoxDimensionsRelative, ComponentIdentifier, HorizontalAlignment, RelativeLength, StructuredChildren, ToUINode, UIIdentifier, UINode, UINodeEventProcessed, VerticalAlignment, UI_IDENTIFIER_MAP}, ui_renderable::TextureMeta};

use super::ui_char::{CharCursor, UIChar};


#[derive(Clone, Copy, Debug)]
pub enum CharEvent{
    LeftPartClicked,
    RightPartClicked,
}

pub struct UITextInner {
    pub box_dimensions: BoxDimensionsRelative,

    pub render_state_changed: bool,
    pub change_parent_render_state: bool,
    pub render_version: u64,

    pub text: Vec<(char, UIChar)>,
    pub dummy_ui_char: DummyUIChar,

    pub pending_char_event: Option<(u64, CharEvent)>,
    pub self_clicked: bool,

    pub font_path: String,
    pub character_scale: f32,

    pub id: UIIdentifier,    

    pub current_blinking_index: Option<u64>,
    
    pub char_event_callback: Arc<dyn Fn(u64, CharEvent)>, // to do

    pub scheduled_insert: Option<String>,
    pub scheduled_delete: bool,
}

impl UITextInner{
    pub fn stop_blinking_all(&mut self){
        for (_, c) in self.text.iter() {
            let mut char_inner = c.inner.write().unwrap();
            char_inner.blinking = false;
            char_inner.showing_cursor = false;
        }
        let mut dummy_ui_char_inner = self.dummy_ui_char.inner.write().unwrap();
        dummy_ui_char_inner.blinking = false;
        dummy_ui_char_inner.showing_cursor = false;
        self.current_blinking_index = None;
    }
    pub fn start_blinking_one(&mut self, index: u64){
        println!("start_blinking_one: {}", index);
        assert!(index <= self.text.len() as u64, "Index out of bounds");
        self.current_blinking_index = Some(index);
        if index == self.text.len() as u64 {            
            let mut char_state = self.dummy_ui_char.inner.write().unwrap();
            char_state.blinking = true;
            char_state.showing_cursor = true;
        }else{ // index < self.text.len() as u64
            let ui_char = &self.text.get(index as usize).as_ref().unwrap().1;
            let mut char_state = ui_char.inner.write().unwrap();
            char_state.blinking = true;
            char_state.showing_cursor = true;
        }
    }
    pub fn insert_string(&mut self, pressed_str: &str){
        let pressed_str = pressed_str.chars().filter(|c| c.is_ascii_graphic() || *c == ' ').collect::<String>();
        if pressed_str.is_empty() {
            return;
        }
        if let Some(index) = self.current_blinking_index {
            println!("insert_string: {}", pressed_str);
            let old_raw_text = self.text.iter().map(|(c, _)| *c).collect::<String>();
            let mut new_text = old_raw_text.clone();
            let new_cursor_position = index as usize + pressed_str.len();
            if index == old_raw_text.len() as u64 { // insert at the end
                new_text += pressed_str.as_str();
            }else{ // insert at the index
                let index = index as usize;
                let (left, right) = new_text.split_at(index);
                new_text = format!("{}{}{}", left, pressed_str, right);
            }
            self.set_text(new_text);
            self.stop_blinking_all();
            self.start_blinking_one(new_cursor_position as u64);
        }
    }
    pub fn delete_char(&mut self){
        if let Some(index) = self.current_blinking_index {
            if index != 0{          
                let old_raw_text = self.text.iter().map(|(c, _)| *c).collect::<String>();
                let mut new_text = old_raw_text.clone();
                let new_cursor_position = index as usize - 1;
                let index = index as usize;
                let (left, right) = new_text.split_at(index);
                let mut left = left.to_string();
                left.pop(); // remove the last character
                new_text = format!("{}{}", left, right.to_string());
                println!("new_text: {}", new_text);
                self.set_text(new_text);
                self.stop_blinking_all();
                self.start_blinking_one(new_cursor_position as u64);
            }
        }
    }
    pub fn set_text(&mut self, text: String) {
        self.text = text.chars()
        .enumerate()
            .map(|(index, c)| {
                let char = UIChar::new(c, self.font_path.clone(), self.character_scale, self.char_event_callback.clone(), index as u64);
                (c, char)
            })
            .collect::<Vec<_>>();
        self.render_state_changed = true;
    }
}

#[derive(Clone)]
pub struct UIText {    
    pub inner: Arc<RwLock<UITextInner>>,
}

impl UIText {
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
            Either::Left(m) => [m, m, m, m],
            Either::Right(m) => m,
        };
        let padding = match padding {
            Either::Left(p) => [p, p, p, p],
            Either::Right(p) => p,
        };
        
        let id = UI_IDENTIFIER_MAP
            .lock()
            .unwrap()
            .next_id(TypeId::of::<UIText>());
        let id = UIIdentifier::Component(ComponentIdentifier::Default{
            id,
            name: format!("Text"),
        });
        let dummy_ui_char = DummyUIChar::new(font_path.clone(), scale);

        let box_dimensions = BoxDimensionsRelative {
            width,
            height,
            margin,
            padding,
        };                        
        let dummy_event_callback = |index: u64, event: CharEvent|{
            println!("Dummy event callback: {} {:?}", index, event);
        };
        let inner = UITextInner {
            box_dimensions,
            render_state_changed: false,
            change_parent_render_state: false,
            render_version: 0,
            text: Vec::new(),
            dummy_ui_char,
            pending_char_event: None,
            self_clicked: false,
            font_path,
            character_scale: scale,
            id,
            char_event_callback: Arc::new(dummy_event_callback),
            current_blinking_index: None,
            scheduled_insert: None,
            scheduled_delete: false,
        };     
        let inner = Arc::new(RwLock::new(inner));
        let char_event_callback = {
            let inner = Arc::downgrade(&inner);
            move |index: u64, event: CharEvent|{
                let inner = inner.upgrade().unwrap();
                let mut inner = inner.write().unwrap();
                println!("CharEvent: {:?}", event);
                inner.pending_char_event = Some((index, event));
            }
        };
        {
            let mut inner = inner.write().unwrap();
            inner.char_event_callback = Arc::new(char_event_callback);
        }
        let result = Self {
            inner,
        };
        result.set_text(initial_text);
        result
    }
    pub fn set_text(&self, text: String) {
        let mut inner = self.inner.write().unwrap();
        inner.set_text(text);
    }
}


impl ToUINode for UIText {
    fn to_ui_node(
        &self,
    ) -> UINode<BoxDimensionsRelative, StructuredChildren<BoxDimensionsRelative>> {        
        let event_handler = {
            let inner = Arc::downgrade(&self.inner);
            let event_handler = move |event: &UINodeEventProcessed|->bool {
                let inner = inner.upgrade().unwrap();
                let mut inner = inner.write().unwrap();
                if event.lose_focus{
                    inner.stop_blinking_all();
                    inner.render_state_changed = true;
                    inner.change_parent_render_state = true;
                }
                if event.left_clicked_inside{
                    inner.self_clicked = true;
                    inner.render_state_changed = true;
                    inner.change_parent_render_state = true;
                }
                if let Some(pressed_str) = event.pressed_str.as_ref() {
                        inner.scheduled_insert = Some(pressed_str.clone());
                        inner.render_state_changed = true;
                        inner.change_parent_render_state = true;
                }
                if let Some(key) = event.key_down {
                    println!("Key pressed: {:?}", key);
                    if key == KeyCode::Backspace {
                        inner.scheduled_delete = true;
                        inner.render_state_changed = true;
                        inner.change_parent_render_state = true;
                    }
                }
                let change_parent_state = inner.change_parent_render_state;
                inner.change_parent_render_state = false;
                change_parent_state
            };
            Some(Box::new(event_handler) as Box<dyn Fn(&UINodeEventProcessed)->bool>)
        };
        let state_changed_handler = {
            let inner = Arc::downgrade(&self.inner);
            let state_changed_handler = move ||{
                let inner = inner.upgrade().unwrap();
                let mut inner = inner.write().unwrap();
                inner.render_state_changed = true;
            };
            Some(Box::new(state_changed_handler) as Box<dyn Fn()>)
        };
        let mut inner = self.inner.write().unwrap();
        // if there is a pending char event and it is not self clicked
        let mut cursor_position: Option<u64> = None;
        if let Some(pressed_str) = inner.scheduled_insert.take() {
            inner.insert_string(&pressed_str);
            inner.scheduled_insert = None;
        }
        if inner.scheduled_delete {
            inner.delete_char();
            inner.scheduled_delete = false;
        }
        if let Some((index, event)) = &inner.pending_char_event {
            let pos = match event {
                CharEvent::LeftPartClicked => *index,
                CharEvent::RightPartClicked => *index + 1,
            };
            cursor_position = Some(pos);
        }else if inner.self_clicked {
            cursor_position = Some(inner.text.len() as u64);
        }
        if let Some(index) = cursor_position {
            inner.stop_blinking_all();
            inner.start_blinking_one(index);
        }
        inner.pending_char_event = None;
        inner.self_clicked = false;
        if inner.render_state_changed {
            inner.render_state_changed = false;
            inner.render_version += 1;
        }
        let mut children_ui_nodes = inner
            .text
            .iter()
            .map(|(_, char)| char.to_ui_node())
            .collect::<Vec<_>>();
        
        let dummy_ui_node = inner.dummy_ui_char.to_ui_node();
        children_ui_nodes.push(dummy_ui_node);
        

        UINode {
            box_dimensions: inner.box_dimensions.clone(),
            children: StructuredChildren::HorizontalLayout {
                h_alignment: HorizontalAlignment::Left,
                v_alignment:VerticalAlignment::Top,
                children: children_ui_nodes,
                uniform_division: false,
            },
            texture_meta: TextureMeta::Texture {
                path: "assets/placeholder.png".into(),
            },
            identifier: inner.id.clone(),
            render_version: inner.render_version,
            event_handler,
            render_state_changed_handler: state_changed_handler,
        }
    }
}

pub struct DummyUICharInner{
    pub font_path: String,
    pub scale: f32,
    pub blinking: bool,
    pub showing_cursor: bool,
}

pub struct DummyUIChar{    
    pub inner: Arc<RwLock<DummyUICharInner>>,
}

impl DummyUIChar{
    pub fn new(font_path: String, scale: f32) -> Self {
        let inner = DummyUICharInner {
            blinking: true,
            showing_cursor: false,
            font_path,
            scale,
        };
        Self {            
            inner: Arc::new(RwLock::new(inner)),
        }
    }
    pub fn get_id(&self) -> UIIdentifier {
        let inner = self.inner.read().unwrap();
        let show_cursor = inner.blinking && inner.showing_cursor;
        UIIdentifier::Component(ComponentIdentifier::DummyChar { 
            show_cursor
        })
    }
}

impl ToUINode for DummyUIChar {
    fn to_ui_node(
        &self,
    ) -> UINode<BoxDimensionsRelative, StructuredChildren<BoxDimensionsRelative>> {
        let event_handler = {
            let inner = Arc::downgrade(&self.inner);
            let event_handler = move |event: &crate::ui_node::UINodeEventProcessed|->bool {
                let mut change_parent_state = false;
                if event.cursor_blink{                    
                    let inner = inner.upgrade().unwrap();
                    let mut inner = inner.write().unwrap();
                    if inner.blinking{
                        inner.showing_cursor = !inner.showing_cursor;
                        change_parent_state = true;
                        println!("DummyUIChar blinking");
                    }
                }
                change_parent_state
            };
            Some(Box::new(event_handler) as Box<dyn Fn(&crate::ui_node::UINodeEventProcessed)->bool>)
        };
        let mut inner = self.inner.read().unwrap();
        let font = get_font(inner.font_path.clone());
        let font = match font.as_ref() {
            CacheValue::Font(font) => font,
            _ => panic!("Font not found"),
        };
        let scale = rusttype::Scale::uniform(inner.scale);
        let v_metrics = font.v_metrics(scale);
        // round ascent to the nearest integer
        let ascent = v_metrics.ascent.round() as i32;
        let descent = v_metrics.descent.round() as i32;
        let line_gap = v_metrics.line_gap.round() as u32;
        let height = (ascent - descent) as u32;
        let width = height;

        
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
            children,
            texture_meta: TextureMeta::Font {
                character: ' '.into(),
                font_path: inner.font_path.clone(),
            },
            identifier: id,
            render_version: 0,
            event_handler,
            render_state_changed_handler: None,
        }
    }
}