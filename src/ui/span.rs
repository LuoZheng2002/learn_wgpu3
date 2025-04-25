use std::{any::TypeId, sync::{Arc, Mutex}};

use either::Either;

use crate::{ui_node::{BoundedLength, BoxDimensionsRelative, HorizontalAlignment, RelativeLength, StructuredChildren, ToUINode, UIIdentifier, UINode, VerticalAlignment, UI_IDENTIFIER_MAP}, ui_renderable::TextureMeta};

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
