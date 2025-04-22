use std::collections::HashMap;

use either::Either;

use crate::{
    ui::ToUINode,
    ui_node::{HorizontalAlignment, VerticalAlignment},
    ui_renderable::{TextureMeta, UIInstance},
};

// AnchoredCanvas, DockCanvas

// Canvas is the root of the UI tree that always takes up the entire screen
// pub struct Canvas{
//     pub order: u32,
//     pub child: Option<Box<dyn ToUINode>>,
//     pub padding: [CanvasPadding; 4],
//     pub h_alignment: HorizontalAlignment,
//     pub v_alignment: VerticalAlignment,
// }

// impl Canvas{
//     pub fn new(
//         order: u32,
//         padding: Either<CanvasPadding, [CanvasPadding; 4]>,
//         h_alignment: HorizontalAlignment,
//         v_alignment: VerticalAlignment,
//     ) -> Self {
//         let padding = match padding {
//             Either::Left(p) => [p.clone(), p.clone(), p.clone(), p.clone()],
//             Either::Right(p) => p,
//         };
//         Self {
//             order,
//             child: None,
//             padding,
//             h_alignment,
//             v_alignment,
//         }
//     }
//     pub fn set_child(&mut self, child: Box<dyn ToUINode>) {
//         self.child = Some(child);
//     }
// }
