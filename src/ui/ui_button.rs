use std::{any::TypeId, sync::{Arc, Mutex, RwLock}};

use either::Either;

use crate::{ui_node::{BoundedLength, BoxDimensionsRelative, ComponentIdentifier, HorizontalAlignment, RelativeLength, StructuredChildren, ToUINode, UIIdentifier, UINode, UINodeEventProcessed, UINodeEventRaw, VerticalAlignment, UI_IDENTIFIER_MAP}, ui_renderable::TextureMeta};

struct UIButtonInner{
    pub hovered: bool,
    pub clicked: bool,
    pub render_state_changed: bool,
    pub render_version: u64,
    pub box_dimensions: BoxDimensionsRelative,
    child: Option<Box<dyn ToUINode>>,
    pub id: UIIdentifier,
    pub click_callback: Option<Box<dyn Fn()>>,
}
#[derive(Clone)]
pub struct UIButton {
    inner: Arc<RwLock<UIButtonInner>>,
}

// callback function


impl UIButton {
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
            .next_id(TypeId::of::<UIButton>());
        let id = UIIdentifier::Component(ComponentIdentifier::Default {
            id,
            name: format!("Button"),
        });
        
        let click_callback = match click_callback {
            Some(callback) => Some(Box::new(callback) as Box<dyn Fn()>),
            None => None,
        };
        let inner =  UIButtonInner{
            hovered: false,
            clicked: false,
            render_state_changed: false,
            render_version: 0,
            box_dimensions,
            child: None,
            id,
            click_callback,
        };
        Self {
            inner: Arc::new(RwLock::new(inner)),
        }
    }
    pub fn set_child(&self, child: Box<dyn ToUINode>) {
        let mut inner = self.inner.write().unwrap();
        inner.child = Some(child);
    }
}
impl ToUINode for UIButton {
    fn to_ui_node(
        &self,
    ) -> UINode<BoxDimensionsRelative, StructuredChildren<BoxDimensionsRelative>> {
        let event_handler = {
            let inner = Arc::downgrade(&self.inner);
            let event_handler = move |event: &UINodeEventProcessed|->bool {
                let inner = inner.upgrade().unwrap();
                let mut inner = inner.write().unwrap();
                let prev_button_clicked = inner.clicked;
                let prev_button_hovered = inner.hovered;
                if event.left_clicked_inside{
                    inner.clicked = true;
                    if let Some(callback) = inner.click_callback.as_ref(){
                        callback();
                    }
                } else if event.left_released {
                    inner.clicked = false;
                }
                if event.mouse_hover{
                    inner.hovered = true;
                } else {
                    inner.hovered = false;
                }
                inner.clicked != prev_button_clicked || inner.hovered != prev_button_hovered
            };
            Some(Box::new(event_handler) as Box<dyn Fn(&UINodeEventProcessed)->bool>)
        };
        let render_state_changed_handler = {
            let inner = Arc::downgrade(&self.inner);
            let render_state_changed_handler = move ||{
                let inner = inner.upgrade().unwrap();
                let mut inner = inner.write().unwrap();
                inner.render_state_changed = true;
            };
            Some(Box::new(render_state_changed_handler) as Box<dyn Fn()>)
        };
        let mut inner = self.inner.write().unwrap();
        let children = match &inner.child {
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
        
        let texture_meta = if inner.clicked {
            TextureMeta::Texture {
                path: "assets/button3.jpg".into(),
            }}
            else if inner.hovered{
                TextureMeta::Texture {
                    path: "assets/button2.jpg".into(),
                }
            }else{
                TextureMeta::Texture {
                    path: "assets/button.jpg".into(),
            }
        };
        if inner.render_state_changed {
            inner.render_state_changed = false;
            inner.render_version += 1;
        }
        UINode {
            box_dimensions: inner.box_dimensions.clone(),
            children,
            texture_meta,
            identifier: inner.id.clone(),
            render_version: inner.render_version,
            event_handler,
            render_state_changed_handler,
        }
    }
}