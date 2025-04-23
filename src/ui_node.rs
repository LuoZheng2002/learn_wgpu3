// length options: fixed, depends on siblings, depends on parent, depends on children

// decide parents first, then siblings, then children

// a span wrapping several padded characters

// padding, margin, children
// width, height,

use std::{any::TypeId, collections::HashMap, i32, sync::Mutex, vec};

use either::Either;
use lazy_static::lazy_static;

use crate::{ui::Cell, ui_renderable::TextureMeta};

#[derive(Clone)]
pub enum HorizontalAlignment {
    Left,
    Center,
    Right,
}
#[derive(Clone)]
pub enum VerticalAlignment {
    Top,
    Center,
    Bottom,
}
pub enum StructuredChildren<B: BoxDimensions> {
    NoChildren,
    OneChild {
        h_alignment: HorizontalAlignment,
        v_alignment: VerticalAlignment,
        child: Box<UINode<B, StructuredChildren<B>>>,
    },
    HorizontalLayout {
        h_alignment: HorizontalAlignment,
        v_alignment: VerticalAlignment,
        uniform_division: bool,
        children: Vec<UINode<B, StructuredChildren<B>>>,
    },
    // HorizontalWrap(Vec<Box<UINode>>),
    VerticalLayout {
        h_alignment: HorizontalAlignment,
        v_alignment: VerticalAlignment,
        uniform_division: bool,
        children: Vec<UINode<B, StructuredChildren<B>>>,
    },
    GridLayout {
        h_alignment: HorizontalAlignment,
        v_alignment: VerticalAlignment,
        uniform_division: bool,
        children: Vec<UINode<B, StructuredChildren<B>>>,
    },
}

// todo: not sure if putting alignment here is a good idea

/// it means that the UINode that owns this struct is actually the content, so it has alignment information for calculating
/// its position with respect to the parent (cell)
pub struct ChildrenAreCells {
    cells: Vec<UINode<BoxDimensionsWithGlobal, ChildIsContent>>,
    h_alignment: HorizontalAlignment,
    v_alignment: VerticalAlignment,
} //an ui node that owns the "Cells" struct is actually the content
// for each UINode<u32, u32, Content> we need to add position information
// a cell has a fixed size and position, but the content doesn't

// for a content to be rendered inside a cell, we need to know:
// cell width and height
// content width and height
// alignment

pub struct ChildIsContent {
    position_x: u32, // top left corner relative to parent
    position_y: u32,
    content: UINode<BoxDimensionsWithGlobal, ChildrenAreCells>,
}

pub struct UnifiedChildren {
    children: Vec<UINode<BoxDimensionsWithGlobal, UnifiedChildren>>,
}

impl StructuredChildren<BoxDimensionsRelative> {
    pub fn calculate_dimensions(
        self,
        parent_width: u32,
        parent_height: u32,
        screen_width: u32,
        screen_height: u32,
    ) -> StructuredChildren<BoxDimensionsAbsolute> {
        match self {
            StructuredChildren::NoChildren => StructuredChildren::NoChildren,
            StructuredChildren::OneChild {
                h_alignment,
                v_alignment,
                child,
            } => {
                let child = child.calculate_dimensions(
                    parent_width,
                    parent_height,
                    screen_width,
                    screen_height,
                );
                StructuredChildren::OneChild {
                    h_alignment,
                    v_alignment,
                    child: Box::new(child),
                }
            }
            StructuredChildren::HorizontalLayout {
                h_alignment,
                v_alignment,
                uniform_division,
                children,
            } => {
                let new_children = children
                    .into_iter()
                    .map(|child| {
                        child.calculate_dimensions(
                            parent_width,
                            parent_height,
                            screen_width,
                            screen_height,
                        )
                    })
                    .collect::<Vec<_>>();
                StructuredChildren::HorizontalLayout {
                    h_alignment,
                    v_alignment,
                    uniform_division,
                    children: new_children,
                }
            }
            StructuredChildren::VerticalLayout {
                h_alignment,
                v_alignment,
                uniform_division,
                children,
            } => {
                let new_children = children
                    .into_iter()
                    .map(|child| {
                        child.calculate_dimensions(
                            parent_width,
                            parent_height,
                            screen_width,
                            screen_height,
                        )
                    })
                    .collect::<Vec<_>>();
                StructuredChildren::VerticalLayout {
                    h_alignment,
                    v_alignment,
                    uniform_division,
                    children: new_children,
                }
            }
            StructuredChildren::GridLayout {
                h_alignment,
                v_alignment,
                uniform_division,
                children,
            } => {
                todo!()
            }
        }
    }
}

#[derive(Clone)]
pub enum RelativeLength {
    Pixels(u32),
    RelativeScreenWidth(f32),
    RelativeScreenHeight(f32),
    RelativeParent(f32),
}
impl RelativeLength {
    pub fn zero() -> Self {
        Self::Pixels(0)
    }
}

#[derive(Clone)]
pub struct BoundedLength {
    pub preferred_length: RelativeLength,
    pub min_length: Option<RelativeLength>,
    pub max_length: Option<RelativeLength>,
}

impl BoundedLength {
    pub fn zero() -> Self {
        Self {
            preferred_length: RelativeLength::zero(),
            min_length: Some(RelativeLength::zero()),
            max_length: Some(RelativeLength::zero()),
        }
    }
    pub fn fixed_dependent(length: RelativeLength) -> Self {
        Self {
            preferred_length: length.clone(),
            min_length: Some(length.clone()),
            max_length: Some(length),
        }
    }
    pub fn fixed_pixels(length: u32) -> Self {
        Self::fixed_dependent(RelativeLength::Pixels(length))
    }
}

pub trait UINodeLength1 {}
impl UINodeLength1 for u32 {}
impl UINodeLength1 for BoundedLength {}

pub trait UINodeLength2 {}
impl UINodeLength2 for u32 {}
impl UINodeLength2 for RelativeLength {}

pub trait UIChildren<B: BoxDimensions> {}

impl<B: BoxDimensions> UIChildren<B> for StructuredChildren<B> {}
impl UIChildren<BoxDimensionsWithGlobal> for ChildrenAreCells {}
impl UIChildren<BoxDimensionsWithGlobal> for ChildIsContent {}
impl UIChildren<BoxDimensionsWithGlobal> for UnifiedChildren {}

#[derive(Clone)]
pub struct BoxDimensionsRelative {
    pub width: BoundedLength,
    pub height: BoundedLength,
    pub margin: [RelativeLength; 4],  // top, right, bottom, left
    pub padding: [RelativeLength; 4], // top, right, bottom, left
}
#[derive(Clone)]
pub struct BoxDimensionsAbsolute {
    pub width: u32,
    pub height: u32,
    pub margin: [u32; 4],  // top, right, bottom, left
    pub padding: [u32; 4], // top, right, bottom, left
}

#[derive(Clone)]
pub struct BoxDimensionsWithGlobal {
    pub width: u32,
    pub height: u32,
    pub rel_pos_x: u32,
    pub rel_pos_y: u32,
    pub global_pos_x: u32,
    pub global_pos_y: u32,
    pub margin: [u32; 4],  // top, right, bottom, left
    pub padding: [u32; 4], // top, right, bottom, left
}
pub trait BoxDimensions {}
impl BoxDimensions for BoxDimensionsRelative {}
impl BoxDimensions for BoxDimensionsAbsolute {}
impl BoxDimensions for BoxDimensionsWithGlobal {}

impl BoxDimensionsAbsolute {
    pub fn width_with_margin(&self) -> u32 {
        self.width + self.margin[3] + self.margin[1] // margin_left + margin_right
    }
    pub fn height_with_margin(&self) -> u32 {
        self.height + self.margin[0] + self.margin[2] // margin_top + margin_bottom
    }
    pub fn inner_width(&self) -> u32 {
        // overflow protection
        let result: i32 = self.width as i32 - self.padding[3] as i32 - self.padding[1] as i32;
        if result < 0 {
            println!("Warning: inner width is negative");
            println!(
                "width: {}, padding_left: {}, padding_right: {}",
                self.width, self.padding[3], self.padding[1]
            );
        }

        i32::max(result, 0) as u32 // padding_left + padding_right
    }
    pub fn inner_height(&self) -> u32 {
        // overflow protection
        let result: i32 = self.height as i32 - self.padding[0] as i32 - self.padding[2] as i32;
        if result < 0 {
            println!("Warning: inner height is negative");
            println!(
                "height: {}, padding_top: {}, padding_bottom: {}",
                self.height, self.padding[0], self.padding[2]
            );
        }
        i32::max(result, 0) as u32 // padding_top + padding_bottom
    }
    pub fn inner_pos_x(&self) -> u32 {
        self.padding[3] // padding_left
    }
    pub fn inner_pos_y(&self) -> u32 {
        self.padding[0] // padding_top
    }
}

// #[derive(Clone)]
// pub struct BoxModel<L1: UINodeLength1, L2: UINodeLength2>{
//     pub dimensions: BoxDimensions<L1, L2>,
//     pub h_alignment: HorizontalAlignment,
//     pub v_alignment: VerticalAlignment,
//     pub uniform_division: bool,
// }

// UI (text, char, etc.) -> UINode<BoundedLength, DependentLength> -> UINode<u32, u32> -> UINode<u32, u32, ChildrenAreCells> -> UIRenderInstruction

// the callbacks are in UINode because it contains the transformed dimensions
// we need to pass all the callbacks from UI to UINode<u32, u32>.
// so, we still need all ui elements' global positions
// the significance of UINode: unify the interface of all UI elements, convert relative lengths to to absolute ones, calculate global positions

// is it possible to handle events in the UINode level?
// events: cursor inside element -> change color, cursor click: call callback, cursor drag: (cursor click + move)
// element resize: has to go to the UI level, takes effect at the next frame, because it may affect siblings

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct UIIdentifier {
    pub name: String,
    pub id: u64,
}
impl UIIdentifier {
    pub fn to_string(&self) -> String {
        format!("{}%{}", self.name, self.id)
    }
}

pub struct UIIdentifierGenerator {
    pub mapping: HashMap<TypeId, u64>,
}

impl UIIdentifierGenerator {
    pub fn new() -> Self {
        UIIdentifierGenerator {
            mapping: HashMap::new(),
        }
    }
    pub fn next_id(&mut self, ui_type: TypeId) -> u64 {
        let next = self.mapping.entry(ui_type).or_insert(0);
        let result = *next;
        *next += 1;
        result
    }
}

// a lazy static mutable hashmap that records the next id of each type of UI, with key being the typeid of the UI struct
lazy_static! {
    pub static ref UI_IDENTIFIER_MAP: Mutex<UIIdentifierGenerator> =
        Mutex::new(UIIdentifierGenerator::new());
}

pub struct UINode<B: BoxDimensions, C: UIChildren<B>> {
    pub box_dimensions: B,
    pub children: C,       // assuming horizontal layout
    pub meta: TextureMeta, // contains optional texture information
    pub identifier: UIIdentifier,
    pub version: u64,
}

impl UINode<BoxDimensionsRelative, StructuredChildren<BoxDimensionsRelative>> {
    pub fn calculate_dimensions(
        self,
        parent_width: u32,
        parent_height: u32,
        screen_width: u32,
        screen_height: u32,
    ) -> UINode<BoxDimensionsAbsolute, StructuredChildren<BoxDimensionsAbsolute>> {
        let dimensions = &self.box_dimensions;
        fn convert_dependent_length_to_u32(
            length_unit: &RelativeLength,
            parent_length: u32,
            screen_width: u32,
            screen_height: u32,
        ) -> u32 {
            match length_unit {
                RelativeLength::Pixels(pixels) => *pixels,
                RelativeLength::RelativeScreenWidth(relative) => {
                    (screen_width as f32 * relative) as u32
                }
                RelativeLength::RelativeScreenHeight(relative) => {
                    (screen_height as f32 * relative) as u32
                }
                RelativeLength::RelativeParent(relative) => {
                    (parent_length as f32 * relative) as u32
                }
            }
        }
        fn convert_bounded_length_to_u32(
            length: &BoundedLength,
            parent_length: u32,
            screen_width: u32,
            screen_height: u32,
        ) -> u32 {
            let BoundedLength {
                preferred_length,
                min_length,
                max_length,
            } = length;
            let preferred_length = convert_dependent_length_to_u32(
                preferred_length,
                parent_length,
                screen_width,
                screen_height,
            );
            let min_length = match min_length {
                Some(length) => convert_dependent_length_to_u32(
                    length,
                    parent_length,
                    screen_width,
                    screen_height,
                ),
                None => u32::MIN,
            };
            let max_length = match max_length {
                Some(length) => convert_dependent_length_to_u32(
                    length,
                    parent_length,
                    screen_width,
                    screen_height,
                ),
                None => u32::MAX,
            };
            if min_length > max_length {
                panic!("min length is greater than max length");
            }
            // clamp preferred length to min and max
            preferred_length.clamp(min_length, max_length)
        }
        // result

        let width = convert_bounded_length_to_u32(
            &dimensions.width,
            parent_width,
            screen_width,
            screen_height,
        );
        let height = convert_bounded_length_to_u32(
            &dimensions.height,
            parent_height,
            screen_width,
            screen_height,
        );

        let children: StructuredChildren<BoxDimensionsAbsolute> = self
            .children
            .calculate_dimensions(width, height, screen_width, screen_height);

        let margin = [
            convert_dependent_length_to_u32(
                &dimensions.margin[0],
                parent_height,
                screen_width,
                screen_height,
            ),
            convert_dependent_length_to_u32(
                &dimensions.margin[1],
                parent_width,
                screen_width,
                screen_height,
            ),
            convert_dependent_length_to_u32(
                &dimensions.margin[2],
                parent_height,
                screen_width,
                screen_height,
            ),
            convert_dependent_length_to_u32(
                &dimensions.margin[3],
                parent_width,
                screen_width,
                screen_height,
            ),
        ];
        let padding = [
            convert_dependent_length_to_u32(
                &dimensions.padding[0],
                parent_height,
                screen_width,
                screen_height,
            ),
            convert_dependent_length_to_u32(
                &dimensions.padding[1],
                parent_width,
                screen_width,
                screen_height,
            ),
            convert_dependent_length_to_u32(
                &dimensions.padding[2],
                parent_height,
                screen_width,
                screen_height,
            ),
            convert_dependent_length_to_u32(
                &dimensions.padding[3],
                parent_width,
                screen_width,
                screen_height,
            ),
        ];
        let box_dimensions = BoxDimensionsAbsolute {
            width,
            height,
            margin,
            padding,
        };
        UINode {
            box_dimensions,
            children,
            meta: self.meta,
            identifier: self.identifier,
            version: self.version,
        }
    }
}

// the canvas will be rendered on the entire screen
pub struct UIRenderInstruction {
    pub version: u64,     // cache key
    pub id: UIIdentifier, // cache key
    pub texture_width: u32,
    pub texture_height: u32,
    pub location_left: f32, // inside parent
    pub location_top: f32,
    pub location_right: f32,
    pub location_bottom: f32,
    pub sub_instructions: Vec<UIRenderInstruction>,
    pub texture_meta: TextureMeta,
}

impl UINode<BoxDimensionsAbsolute, StructuredChildren<BoxDimensionsAbsolute>> {
    fn wrap_node_with_cell(
        node: UINode<BoxDimensionsAbsolute, StructuredChildren<BoxDimensionsAbsolute>>,
        cell_width: u32,
        cell_height: u32,
        cell_rel_pos_x: u32,
        cell_rel_pos_y: u32,
        parent_global_x: u32,
        parent_global_y: u32,
        h_alignment: HorizontalAlignment,
        v_alignment: VerticalAlignment,
    ) -> UINode<BoxDimensionsWithGlobal, ChildIsContent> {
        let cell_global_pos_x = parent_global_x + cell_rel_pos_x;
        let cell_global_pos_y = parent_global_y + cell_rel_pos_y;

        let cell_children = ChildIsContent {
            position_x: cell_rel_pos_x, // likely to be useless
            position_y: cell_rel_pos_y,
            content: node.flatten_children(
                cell_global_pos_x,
                cell_global_pos_y,
                cell_width,
                cell_height,
                h_alignment,
                v_alignment,
            ),
        };
        let cell_meta = TextureMeta::Texture {
            path: "assets/kiminonawa.jpg".into(),
        };
        let cell_dimensions = BoxDimensionsWithGlobal {
            width: cell_width,
            height: cell_height,
            rel_pos_x: cell_rel_pos_x,
            rel_pos_y: cell_rel_pos_y,
            global_pos_x: cell_global_pos_x,
            global_pos_y: cell_global_pos_y,
            margin: [0, 0, 0, 0],
            padding: [0, 0, 0, 0],
        };
        UINode {
            box_dimensions: cell_dimensions,
            children: cell_children,
            meta: cell_meta,
            // every cell is different
            identifier: UIIdentifier {
                name: "Cell".into(),
                id: UI_IDENTIFIER_MAP
                    .lock()
                    .unwrap()
                    .next_id(TypeId::of::<Cell>()),
            },
            version: 0,
        }
    }
    fn get_cell_lengths_and_positions_tangent_dir(
        total_length: u32,
        children_lengths: Vec<u32>,
        parent_padding: u32,
        uniform_division: bool,
        alignment: Either<HorizontalAlignment, VerticalAlignment>,
    ) -> Vec<(u32, u32)> {
        assert!(children_lengths.len() > 0);
        if uniform_division {
            let num_children = children_lengths.len();
            let cell_length = total_length / num_children as u32;
            let cell_lengths_and_positions = (0..num_children)
                .map(|i| {
                    let cell_pos = i as u32 * cell_length + parent_padding;
                    (cell_length, cell_pos)
                })
                .collect::<Vec<_>>();
            cell_lengths_and_positions
        } else {
            let children_length_sum = children_lengths.iter().sum::<u32>();
            let padding = (total_length as i32 - children_length_sum as i32) / 2;
            let padding = i32::max(padding, 0) as u32;
            let padding_factor = match alignment {
                Either::Left(HorizontalAlignment::Left) => 0,
                Either::Left(HorizontalAlignment::Center) => 1,
                Either::Left(HorizontalAlignment::Right) => 2,
                Either::Right(VerticalAlignment::Top) => 0,
                Either::Right(VerticalAlignment::Center) => 1,
                Either::Right(VerticalAlignment::Bottom) => 2,
            };
            let cell_lengths_and_positions = children_lengths
                .iter()
                .enumerate()
                .map(|(i, length)| {
                    let cell_pos = parent_padding
                        + padding * padding_factor
                        + children_lengths[..i].iter().sum::<u32>();
                    (*length, cell_pos)
                })
                .collect::<Vec<_>>();
            cell_lengths_and_positions
        }
    }
    fn get_cell_lengths_and_positions_normal_dir(
        length: u32,
        children_lengths: Vec<u32>,
        parent_padding: u32,
        alignment: Either<HorizontalAlignment, VerticalAlignment>,
    ) -> Vec<(u32, u32)> {
        let padding_factor = match alignment {
            Either::Left(HorizontalAlignment::Left) => 0,
            Either::Left(HorizontalAlignment::Center) => 1,
            Either::Left(HorizontalAlignment::Right) => 2,
            Either::Right(VerticalAlignment::Top) => 0,
            Either::Right(VerticalAlignment::Center) => 1,
            Either::Right(VerticalAlignment::Bottom) => 2,
        };
        let cell_lengths_and_positions = children_lengths
            .iter()
            .map(|child_length| {
                let padding = (length - child_length) / 2;
                let cell_pos = parent_padding + padding * padding_factor;
                (length, cell_pos)
            })
            .collect::<Vec<_>>();
        cell_lengths_and_positions
    }
    pub fn flatten_children(
        self,
        parent_global_x: u32, // assume it does not take into account parent's padding
        parent_global_y: u32,
        parent_width: u32,
        parent_height: u32,
        h_alignment: HorizontalAlignment, // if the outermost node is a canvas that covers the entire screen, it does not matter
        v_alignment: VerticalAlignment,
    ) -> UINode<BoxDimensionsWithGlobal, ChildrenAreCells> {
        let UINode {
            box_dimensions,
            children,
            meta,
            identifier,
            version,
        } = self;
        let width_difference = parent_width as i32 - box_dimensions.width_with_margin() as i32;
        let width_difference = i32::max(width_difference, 0) as u32;
        let height_difference = parent_height as i32 - box_dimensions.height_with_margin() as i32;
        let height_difference = i32::max(height_difference, 0) as u32;
        let left_padding = match h_alignment {
            HorizontalAlignment::Left => 0,
            HorizontalAlignment::Center => width_difference / 2,
            HorizontalAlignment::Right => width_difference,
        };
        let top_padding = match v_alignment {
            VerticalAlignment::Top => 0,
            VerticalAlignment::Center => height_difference / 2,
            VerticalAlignment::Bottom => height_difference,
        };
        let self_rel_x = left_padding + box_dimensions.margin[3];
        let self_rel_y = top_padding + box_dimensions.margin[0];
        let self_global_x = parent_global_x + self_rel_x;
        let self_global_y = parent_global_y + self_rel_y;
        let children: ChildrenAreCells = match children {
            StructuredChildren::NoChildren => {
                ChildrenAreCells {
                    cells: vec![],
                    h_alignment: HorizontalAlignment::Left, //don't care
                    v_alignment: VerticalAlignment::Top,    // don't care
                }
            }
            StructuredChildren::OneChild {
                h_alignment,
                v_alignment,
                child,
            } => {
                let cell_width = box_dimensions.inner_width();
                let cell_height = box_dimensions.inner_height();
                let cell_pos_x = box_dimensions.inner_pos_x();
                let cell_pos_y = box_dimensions.inner_pos_y();
                let cell = Self::wrap_node_with_cell(
                    *child,
                    cell_width,
                    cell_height,
                    cell_pos_x,
                    cell_pos_y,
                    self_global_x,
                    self_global_y,
                    h_alignment.clone(),
                    v_alignment.clone(),
                );
                ChildrenAreCells {
                    cells: vec![cell],
                    h_alignment, // this is likely to have no effect now
                    v_alignment,
                }
            }
            StructuredChildren::HorizontalLayout {
                h_alignment,
                v_alignment,
                uniform_division,
                children,
            } => {
                if children.len() == 0 {
                    ChildrenAreCells {
                        cells: vec![],
                        h_alignment,
                        v_alignment,
                    }
                } else {
                    let num_children = children.len();
                    let total_width = box_dimensions.inner_width();
                    let children_widths = children
                        .iter()
                        .map(|child| {
                            let child_dimensions = &child.box_dimensions;
                            child_dimensions.width_with_margin()
                        })
                        .collect::<Vec<_>>();
                    let cell_widths_and_positions =
                        Self::get_cell_lengths_and_positions_tangent_dir(
                            total_width,
                            children_widths,
                            box_dimensions.inner_pos_x(),
                            uniform_division,
                            Either::Left(h_alignment.clone()),
                        );
                    let cell_height = box_dimensions.inner_height();
                    let cell_heights_and_positions =
                        Self::get_cell_lengths_and_positions_normal_dir(
                            cell_height,
                            vec![cell_height; num_children],
                            box_dimensions.inner_pos_y(),
                            Either::Right(v_alignment.clone()),
                        );
                    let cell_widths_and_heights_and_positions = cell_widths_and_positions
                        .into_iter()
                        .zip(cell_heights_and_positions.into_iter())
                        .collect::<Vec<_>>();
                    let cells = children
                        .into_iter()
                        .zip(cell_widths_and_heights_and_positions)
                        .map(
                            |(child, ((cell_width, cell_pos_x), (cell_height, cell_pos_y)))| {
                                Self::wrap_node_with_cell(
                                    child,
                                    cell_width,
                                    cell_height,
                                    cell_pos_x, // it includes parent's padding
                                    cell_pos_y,
                                    self_global_x, // so this does not include parent's padding
                                    self_global_y,
                                    h_alignment.clone(),
                                    v_alignment.clone(),
                                )
                            },
                        )
                        .collect();
                    ChildrenAreCells {
                        cells,
                        h_alignment,
                        v_alignment,
                    }
                }
            }
            StructuredChildren::VerticalLayout {
                h_alignment,
                v_alignment,
                uniform_division,
                children,
            } => {
                if children.len() == 0 {
                    ChildrenAreCells {
                        cells: vec![],
                        h_alignment,
                        v_alignment,
                    }
                } else {
                    let num_children = children.len();
                    let total_height = box_dimensions.inner_height();
                    let children_heights = children
                        .iter()
                        .map(|child| {
                            let child_dimensions = &child.box_dimensions;
                            child_dimensions.height_with_margin()
                        })
                        .collect::<Vec<_>>();
                    let cell_heights_and_positions =
                        Self::get_cell_lengths_and_positions_tangent_dir(
                            total_height,
                            children_heights,
                            box_dimensions.inner_pos_y(),
                            uniform_division,
                            Either::Right(v_alignment.clone()),
                        );
                    let cell_width = box_dimensions.inner_width();
                    let cell_widths_and_positions = Self::get_cell_lengths_and_positions_normal_dir(
                        cell_width,
                        vec![cell_width; num_children],
                        box_dimensions.inner_pos_x(),
                        Either::Left(h_alignment.clone()),
                    );
                    let cell_widths_and_heights_and_positions = cell_widths_and_positions
                        .into_iter()
                        .zip(cell_heights_and_positions.into_iter())
                        .collect::<Vec<_>>();
                    let cells = children
                        .into_iter()
                        .zip(cell_widths_and_heights_and_positions)
                        .map(
                            |(child, ((cell_width, cell_pos_x), (cell_height, cell_pos_y)))| {
                                Self::wrap_node_with_cell(
                                    child,
                                    cell_width,
                                    cell_height,
                                    cell_pos_x,
                                    cell_pos_y,
                                    self_global_x,
                                    self_global_y,
                                    h_alignment.clone(),
                                    v_alignment.clone(),
                                )
                            },
                        )
                        .collect();
                    ChildrenAreCells {
                        cells,
                        h_alignment,
                        v_alignment,
                    }
                }
            }
            StructuredChildren::GridLayout {
                h_alignment,
                v_alignment,
                uniform_division,
                children,
            } => {
                todo!()
            }
        };
        let box_dimensions = BoxDimensionsWithGlobal {
            width: box_dimensions.width,
            height: box_dimensions.height,
            rel_pos_x: self_rel_x,
            rel_pos_y: self_rel_y,
            global_pos_x: self_global_x,
            global_pos_y: self_global_y,
            margin: box_dimensions.margin,
            padding: box_dimensions.padding,
        };
        UINode {
            box_dimensions,
            children,
            meta,
            identifier,
            version,
        }
    }
}

impl UINode<BoxDimensionsWithGlobal, ChildrenAreCells> {
    pub fn to_unified(self) -> UINode<BoxDimensionsWithGlobal, UnifiedChildren> {
        let UINode {
            box_dimensions,
            children,
            meta,
            identifier,
            version,
        } = self;
        let children = children
            .cells
            .into_iter()
            .map(|child| child.to_unified())
            .collect();
        let children = UnifiedChildren { children };
        UINode {
            box_dimensions,
            children,
            meta,
            identifier,
            version,
        }
    }
}
impl UINode<BoxDimensionsWithGlobal, ChildIsContent> {
    pub fn to_unified(self) -> UINode<BoxDimensionsWithGlobal, UnifiedChildren> {
        let UINode {
            box_dimensions,
            children,
            meta,
            identifier,
            version,
        } = self;
        let children = vec![children.content.to_unified()];
        let children = UnifiedChildren { children };
        UINode {
            box_dimensions,
            children,
            meta,
            identifier,
            version,
        }
    }
}

impl UINode<BoxDimensionsWithGlobal, UnifiedChildren> {
    pub fn to_ui_render_instruction(
        &self,
        parent_width: u32,
        parent_height: u32,
    ) -> UIRenderInstruction {
        let UINode {
            box_dimensions,
            children,
            meta,
            identifier,
            version,
        } = self;
        let texture_width = box_dimensions.width;
        let texture_height = box_dimensions.height;
        let sub_instructions = children
            .children
            .iter()
            .map(|child| {
                let parent_width = texture_width;
                let parent_height = texture_height;
                child.to_ui_render_instruction(parent_width, parent_height)
            })
            .collect::<Vec<_>>();
        let location_left = box_dimensions.rel_pos_x;
        let location_top = box_dimensions.rel_pos_y;
        let location_right = location_left + box_dimensions.width;
        let location_bottom = location_top + box_dimensions.height;
        let location_left = location_left as f32 / parent_width as f32;
        let location_top = location_top as f32 / parent_height as f32;
        let location_right = location_right as f32 / parent_width as f32;
        let location_bottom = location_bottom as f32 / parent_height as f32;
        UIRenderInstruction {
            version: *version,
            id: identifier.clone(),
            texture_width,
            texture_height,
            location_top,
            location_left,
            location_bottom,
            location_right,
            sub_instructions,
            texture_meta: meta.clone(),
        }
    }
    pub fn to_string(&self, indent: u32) -> String {
        let UINode {
            box_dimensions,
            children,
            meta,
            identifier,
            version,
        } = self;
        let pad = " ".repeat((indent * 4) as usize);
        let mut result = format!(
            "{}ID: {}, Version: {}, w: {}, h: {}, rel_x: {}, rel_y:{}, glo_x: {}, glo_y:{}, margin: {:?}, padding: {:?}, meta: {:?}",
            pad,
            identifier.to_string(),
            version,
            box_dimensions.width,
            box_dimensions.height,
            box_dimensions.rel_pos_x,
            box_dimensions.rel_pos_y,
            box_dimensions.global_pos_x,
            box_dimensions.global_pos_y,
            box_dimensions.margin,
            box_dimensions.padding,
            meta
        );
        for child in children.children.iter() {
            result.push('\n');
            result.push_str(&child.to_string(indent + 1));
        }
        result
    }
}
