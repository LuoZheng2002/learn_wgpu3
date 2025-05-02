use std::{any::TypeId, sync::{Arc, Mutex, RwLock}};

use either::Either;

use crate::{ui_node::{BoundedLength, BoxDimensionsRelative, ComponentIdentifier, HorizontalAlignment, RelativeLength, StructuredChildren, ToUINode, UIIdentifier, UINode, VerticalAlignment, UI_IDENTIFIER_MAP}, ui_renderable::TextureMeta};

pub enum SpanDirection {
    Horizontal,
    Vertical,
}

struct UISpanInner{
    pub render_state_changed: bool,
    pub render_version: u64,
    pub direction: SpanDirection,
    pub children: Vec<Box<dyn ToUINode>>,
    pub box_dimensions: BoxDimensionsRelative,
    pub h_alignment: HorizontalAlignment,
    pub v_alignment: VerticalAlignment,
    pub uniform_division: bool,
    pub texture_meta: TextureMeta,
    pub id: UIIdentifier,
}

#[derive(Clone)]
pub struct UISpan {
    inner: Arc<RwLock<UISpanInner>>,
}

impl UISpan {
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
            Either::Left(m) => [m, m, m, m],
            Either::Right(m) => m,
        };
        let padding = match padding {
            Either::Left(p) => [p, p, p, p],
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
            .next_id(TypeId::of::<UISpan>());
        let id = UIIdentifier::Component(ComponentIdentifier::Default {
            id,
            name: format!("Span"),
        });
        let inner = UISpanInner {
            render_state_changed: false,
            render_version: 0,
            direction,
            children: Vec::new(),
            box_dimensions,
            h_alignment,
            v_alignment,
            uniform_division,
            texture_meta: texture,
            id,
        };
        Self {
            inner: Arc::new(RwLock::new(inner)),
        }
    }
    pub fn push_child(&self, child: Box<dyn ToUINode>) {
        let mut inner = self.inner.write().unwrap();
        inner.children.push(child);
    }
}


// we want other contexts to access the ui and modify it

impl ToUINode for UISpan {
    fn to_ui_node(
        &self,
    ) -> UINode<BoxDimensionsRelative, StructuredChildren<BoxDimensionsRelative>> {
        
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
        // respond to pending changes
        if inner.render_state_changed {
            inner.render_state_changed = false;
            inner.render_version += 1;
        }
        let children_ui_nodes = inner
            .children
            .iter()
            .map(|c| c.to_ui_node())
            .collect::<Vec<_>>();
        UINode {
            box_dimensions: inner.box_dimensions.clone(),
            children: match &inner.direction {
                SpanDirection::Horizontal => StructuredChildren::HorizontalLayout {
                    h_alignment: inner.h_alignment,
                    v_alignment: inner.v_alignment,
                    uniform_division: inner.uniform_division,
                    children: children_ui_nodes,
                },
                SpanDirection::Vertical => StructuredChildren::VerticalLayout {
                    h_alignment: inner.h_alignment,
                    v_alignment: inner.v_alignment,
                    uniform_division: inner.uniform_division,
                    children: children_ui_nodes,
                },
            },
            texture_meta: inner.texture_meta.clone(),
            identifier: inner.id.clone(),
            render_version: inner.render_version,
            event_handler: None,
            render_state_changed_handler,
        }
    }
}
