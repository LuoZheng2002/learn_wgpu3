use std::any::TypeId;

use crate::{cache::{get_font, CacheValue}, ui_node::{BoundedLength, BoxDimensionsRelative, RelativeLength, StructuredChildren, ToUINode, UIIdentifier, UINode, UI_IDENTIFIER_MAP}, ui_renderable::TextureMeta};


pub struct UIChar {
    pub character: char,
    pub font_path: String,
    pub scale: f32,
    pub id: UIIdentifier,
}

impl UIChar {
    pub fn new(character: char, font_path: String, scale: f32) -> Self {
        let id = UI_IDENTIFIER_MAP
            .lock()
            .unwrap()
            .next_id(TypeId::of::<UIChar>());
        let id = UIIdentifier {
            id: id,
            name: format!("Char({})", character),
        };
        Self {
            character,
            font_path,
            scale,
            id,
        }
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
        UINode {
            box_dimensions,
            children: crate::ui_node::StructuredChildren::NoChildren,
            meta: TextureMeta::Font {
                character: self.character,
                font_path: self.font_path.clone(),
            },
            identifier: self.id.clone(),
            version: 0,
            event_handler: None,
            state_changed_handler: None,
        }
    }
}