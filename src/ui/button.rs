use std::{any::TypeId, sync::{Arc, Mutex}};

use either::Either;

use crate::{ui_node::{BoundedLength, BoxDimensionsRelative, ComponentIdentifier, HorizontalAlignment, RelativeLength, StructuredChildren, ToUINode, UIIdentifier, UINode, UINodeEventProcessed, UINodeEventRaw, VerticalAlignment, UI_IDENTIFIER_MAP}, ui_renderable::TextureMeta};


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
        let id = UIIdentifier::Component(ComponentIdentifier::Default {
            id,
            name: format!("Button"),
        });
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