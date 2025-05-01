use std::{any::TypeId, sync::{Arc, Mutex, Weak}};

use crate::{cache::{get_font, CacheValue}, ui_node::{BoundedLength, BoxDimensionsRelative, ComponentIdentifier, HorizontalAlignment, RelativeLength, StructuredChildren, ToUINode, UIIdentifier, UINode, VerticalAlignment, UI_IDENTIFIER_MAP}, ui_renderable::TextureMeta};

use super::text::CharEvent;

/// a blinking character has a different id than a non-blinking character
pub struct UIChar {
    pub character: char,
    pub font_path: String,
    pub scale: f32,
    pub char_state: Arc<Mutex<UICharState>>,    
}

pub struct UICharState{
    pub blinking: bool,
    pub showing_cursor: bool,
    pub char_event_callback: Weak<dyn Fn(u64, CharEvent)>,
    pub index: u64,
}

impl UIChar {
    pub fn new(character: char, 
        font_path: String, 
        scale: f32, 
        char_event_callback: Weak<dyn Fn(u64, CharEvent)>,
        index: u64,
    ) -> Self {
        let char_state = UICharState {
            blinking: true,
            showing_cursor: false,
            char_event_callback,
            index,
        };
        let char_state = Arc::new(Mutex::new(char_state));
        Self {
            character,
            font_path,
            scale,
            char_state,
        }
    }
    pub fn get_id(&self) -> UIIdentifier {
        let char_state = self.char_state.lock().unwrap();
        let show_cursor = char_state.blinking && char_state.showing_cursor;
        UIIdentifier::Component(ComponentIdentifier::Char { 
            character: self.character, 
            font_path: self.font_path.clone(), 
            show_cursor
        })
    }
}

impl ToUINode for UIChar {
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
        let id = self.get_id();
        let show_cursor = match id{
            UIIdentifier::Component(ComponentIdentifier::Char { show_cursor, .. }) => show_cursor,
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
            // let character = self.character;
            let event_handler = move |event: &crate::ui_node::UINodeEventProcessed|->bool {
                let char_state = char_state.upgrade().unwrap();
                let mut char_state = char_state.lock().unwrap();
                let mut change_parent_state = false;
                // handle mouse clicks
                let char_event_callback = char_state.char_event_callback.upgrade().unwrap();
                if event.left_clicked_inside{
                    char_event_callback(char_state.index, CharEvent::LeftPartClicked);
                    change_parent_state = true;
                }else if event.right_clicked_inside{
                    char_event_callback(char_state.index, CharEvent::RightPartClicked);
                    change_parent_state = true;
                }
                // handle toggling binking
                if event.cursor_blink{                    
                    if char_state.blinking{
                        char_state.showing_cursor = !char_state.showing_cursor;
                        change_parent_state = true;
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
                character: self.character,
                font_path: self.font_path.clone(),
            },
            identifier: id,
            version: 0,
            event_handler,
            state_changed_handler: None,
        }
    }
}

pub struct CharCursor{

}

impl ToUINode for CharCursor {
    fn to_ui_node(
        &self,
    ) -> UINode<BoxDimensionsRelative, StructuredChildren<BoxDimensionsRelative>> {
        let box_dimensions = BoxDimensionsRelative {
            width: BoundedLength::fixed_dependent(RelativeLength::RelativeParentHeight(0.05)),
            height: BoundedLength::fixed_dependent(RelativeLength::RelativeParentHeight(1.0)),
            margin: [
                RelativeLength::Pixels(0),
                RelativeLength::Pixels(0),
                RelativeLength::Pixels(0),
                RelativeLength::Pixels(0),
            ],
            padding: [
                RelativeLength::Pixels(0),
                RelativeLength::Pixels(0),
                RelativeLength::Pixels(0),
                RelativeLength::Pixels(0),
            ],
        };
        let id = UIIdentifier::Component(ComponentIdentifier::Default {
            id: 0,
            name: format!("CharCursor"),
        });
        UINode {
            box_dimensions,
            children: StructuredChildren::NoChildren,
            meta: TextureMeta::Texture {
                path: "assets/text_cursor.png".into(),
            },
            identifier: id,
            version: 0,
            event_handler: None,
            state_changed_handler: None,
        }
    }
}