use std::collections::HashMap;

use either::Either;

use crate::{ui::ToUINode, ui_node::{CanvasPadding, HorizontalAlignment, VerticalAlignment}, ui_renderable::{UIInstance, UIRenderableMeta}};




// Canvas is the root of the UI tree that always takes up the entire screen
pub struct Canvas{
    pub order: u32,
    pub child: Option<Box<dyn ToUINode>>,
    pub padding: [CanvasPadding; 4],
    pub h_alignment: HorizontalAlignment,
    pub v_alignment: VerticalAlignment,
}

impl Canvas{
    pub fn new(
        order: u32,
        padding: Either<CanvasPadding, [CanvasPadding; 4]>,
        h_alignment: HorizontalAlignment,
        v_alignment: VerticalAlignment,
    ) -> Self {
        let padding = match padding {
            Either::Left(p) => [p.clone(), p.clone(), p.clone(), p.clone()],
            Either::Right(p) => p,
        };
        Self {
            order,
            child: None,
            padding,
            h_alignment,
            v_alignment,
        }
    }
    pub fn set_child(&mut self, child: Box<dyn ToUINode>) {
        self.child = Some(child);
    }
    pub fn to_ui_renderables(&self, screen_width: u32, screen_height: u32)-> HashMap<UIRenderableMeta, Vec<UIInstance>>{
        let convert_padding_to_i32 = |padding: &CanvasPadding| -> i32 {
            match padding {
                CanvasPadding::Pixels(p) => *p,
                CanvasPadding::RelativeScreenWidth(ratio) => (screen_width as f32 * ratio) as i32,
                CanvasPadding::RelativeScreenHeight(ratio) => (screen_height as f32 * ratio) as i32,
            }
        };
        match &self.child{
            Some(child) => {
                let child_ui_node = child.to_ui_node();
                let child_ui_node = child_ui_node.calculate_dimensions(screen_width as i32, screen_height as i32, screen_width, screen_height);
                let padding = [
                    convert_padding_to_i32(&self.padding[0]),
                    convert_padding_to_i32(&self.padding[1]),
                    convert_padding_to_i32(&self.padding[2]),
                    convert_padding_to_i32(&self.padding[3]),
                ];
                child_ui_node.to_ui_renderables(self.order,0, screen_width, screen_height, padding[3], padding[0])
            }
            None => {
                HashMap::new()
            }
        }
    }
}