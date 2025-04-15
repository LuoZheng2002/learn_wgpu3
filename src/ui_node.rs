// length options: fixed, depends on siblings, depends on parent, depends on children

// decide parents first, then siblings, then children

// a span wrapping several padded characters

// padding, margin, children
// width, height,

use std::{collections::HashMap, default, i32};

use crate::ui_renderable::{SortOrder, UIInstance, UIRenderableMeta};


#[derive(Clone)]
pub enum HorizontalAlignment{
    Left,
    Center,
    Right,
}
#[derive(Clone)]
pub enum VerticalAlignment{
    Top,
    Center,
    Bottom,
}
pub enum Children<L1: UINodeLength1, L2: UINodeLength2>{ 
    NoChildren,
    OneChild(Box<UINode<L1, L2>>),
    HorizontalLayout(Vec<UINode<L1, L2>>),
    // HorizontalWrap(Vec<Box<UINode>>),
    VerticalLayout(Vec<UINode<L1, L2>>),
    GridLayout(Vec<Vec<UINode<L1, L2>>>),
}

#[derive(Clone)]
pub enum CanvasPadding{
    Pixels(i32),
    RelativeScreenWidth(f32),
    RelativeScreenHeight(f32),
}

#[derive(Clone)]
pub enum LengthUnit{
    Pixels(i32),
    RelativeScreenWidth(f32),
    RelativeScreenHeight(f32),
    RelativeParent(f32),
}

// impl LengthUnit{
//     pub fn pixels(length: i32)-> Self{
//         Self::Pixels(length)
//     }
// }

#[derive(Clone)]
pub struct ChildrenIndependent{
    pub preferred_length: LengthUnit,
    pub min_length: Option<LengthUnit>,
    pub max_length: Option<LengthUnit>,
}

#[derive(Clone)]
pub enum DependentLength{
    ChildrenIndependent(ChildrenIndependent),
    FitChildren{
        default_length: LengthUnit,
    }
}

impl DependentLength{
    pub fn zero()-> Self{
        Self::ChildrenIndependent(
            ChildrenIndependent{
                preferred_length: LengthUnit::Pixels(0),
                min_length: Some(LengthUnit::Pixels(0)),
                max_length: Some(LengthUnit::Pixels(0)),
            }
        )
    }
    pub fn fixed_dependent(length: LengthUnit)-> Self{
        Self::ChildrenIndependent(
            ChildrenIndependent{
                preferred_length: length.clone(),
                min_length: Some(length.clone()),
                max_length: Some(length),
            }
        )
    }
    pub fn fixed_pixels(length: i32)-> Self{
        Self::fixed_dependent(LengthUnit::Pixels(length))
    }
    pub fn fit_children(default_length: LengthUnit)-> Self{
        Self::FitChildren{
            default_length,
        }
    }
    pub fn fit_children_default()-> Self{
        Self::fit_children(LengthUnit::RelativeScreenHeight(0.5))
    }
}

pub trait UINodeLength1{}
impl UINodeLength1 for i32{}
impl UINodeLength1 for DependentLength{}

pub trait UINodeLength2{}
impl UINodeLength2 for i32{}
impl UINodeLength2 for LengthUnit{}


#[derive(Clone)]
pub struct BoxDimensions<L1: UINodeLength1, L2: UINodeLength2>{
    pub width: L1,
    pub height: L1,   
    pub margin: [L2; 4], // top, right, bottom, left
    pub padding: [L2; 4], // top, right, bottom, left
}

impl BoxDimensions<i32, i32>{
    pub fn width_with_margin(&self)-> i32{
        self.width + self.margin[3] + self.margin[1] // margin_left + margin_right
    }
    pub fn height_with_margin(&self)-> i32{
        self.height + self.margin[0] + self.margin[2] // margin_top + margin_bottom
    }
}


#[derive(Clone)]
pub struct BoxModel<L1: UINodeLength1, L2: UINodeLength2>{
    pub dimensions: BoxDimensions<L1, L2>,
    pub h_alignment: HorizontalAlignment,
    pub v_alignment: VerticalAlignment,
    pub color: cgmath::Vector4<f32>, // color is extracted from the meta because it is not a part of the bind group
}

pub struct UINode<L1: UINodeLength1, L2: UINodeLength2>{ 
    pub box_model: BoxModel<L1, L2>,
    pub children: Children<L1, L2>, // assuming horizontal layout
    pub meta: UIRenderableMeta, // contains optional texture information
}

impl UINode<DependentLength, LengthUnit>{
    pub fn calculate_dimensions(self,
        parent_width: i32, 
        parent_height: i32,
        screen_width: u32,
        screen_height: u32,
    )-> UINode<i32, i32>{
        let box_model = self.box_model;
        let dimensions = &box_model.dimensions;
        fn convert_length_unit_to_i32(length_unit: &LengthUnit, parent_length: i32, screen_width: u32, screen_height: u32)-> i32{
            match length_unit{
                LengthUnit::Pixels(pixels) => *pixels,
                LengthUnit::RelativeScreenWidth(relative) => (screen_width as f32 * relative) as i32,
                LengthUnit::RelativeScreenHeight(relative) => (screen_height as f32 * relative) as i32,
                LengthUnit::RelativeParent(relative) => (parent_length as f32 * relative) as i32,
            }
        }
        fn convert_independent_children_to_i32(dependent_length: &ChildrenIndependent, parent_length: i32, screen_width: u32, screen_height: u32)-> i32{
            let ChildrenIndependent{preferred_length, min_length, max_length} = dependent_length;
            let new_preferred_length = convert_length_unit_to_i32(preferred_length, parent_length, screen_width, screen_height);
            let new_min_length = match min_length{
                Some(length) => Some(convert_length_unit_to_i32(length, parent_length, screen_width, screen_height)),
                None => None,
            };
            let new_max_length = match max_length{
                Some(length) => Some(convert_length_unit_to_i32(length, parent_length, screen_width, screen_height)),
                None => None,
            };
            if let (Some(new_min_length), Some(new_max_length)) = (new_min_length, new_max_length){
                if new_min_length > new_max_length{
                    panic!("min length is greater than max length");
                }
            }
            let mut result_length = new_preferred_length;
            if let Some(min_length) = new_min_length{
                if result_length < min_length{
                    result_length = min_length;
                }
            }
            if let Some(max_length) = new_max_length{
                if result_length > max_length{
                    result_length = max_length;
                }
            }
            result_length
        }
        // result
        fn convert_dependent_to_i32(length: &DependentLength, parent_length: i32, screen_width: u32, screen_height: u32)->(i32, bool){
            match length{
                DependentLength::ChildrenIndependent(children_independent) => {
                    let new_length = convert_independent_children_to_i32(
                        children_independent,
                        parent_length,
                        screen_width, 
                        screen_height);
                    (new_length, false)
                },
                DependentLength::FitChildren{default_length} => {
                    let default_width = convert_length_unit_to_i32(
                        default_length,
                        parent_length,
                        screen_width, 
                        screen_height);
                    (default_width, true)
                },
            }
        }
        let (mut new_width, need_update_width) = convert_dependent_to_i32(
            &dimensions.width,
            parent_width,
            screen_width,
            screen_height,
        );
        let (mut new_height, need_update_height) = convert_dependent_to_i32(
            &dimensions.height,
            parent_height,
            screen_width,
            screen_height,
        );

        let new_children: Children<i32, i32> = match self.children{
            Children::NoChildren => Children::NoChildren,
            Children::OneChild(child) => {
                let child = child.calculate_dimensions(new_width, new_height, screen_width, screen_height);
                Children::OneChild(Box::new(child))
            }
            Children::HorizontalLayout(children) => {
                let new_children = children.into_iter().map(|child| {
                    child.calculate_dimensions(new_width, new_height, screen_width, screen_height)
                }).collect::<Vec<_>>();
                Children::HorizontalLayout(new_children)
            }
            Children::VerticalLayout(children) => {
                let new_children = children.into_iter().map(|child| {
                    child.calculate_dimensions(new_width, new_height, screen_width, screen_height)
                }).collect::<Vec<_>>();
                Children::VerticalLayout(new_children)
            }
            Children::GridLayout(children) => {
                todo!()
            }
        };
        // update width or height based on new children
        let padding = [
            convert_length_unit_to_i32(&dimensions.padding[0], parent_height, screen_width, screen_height),
            convert_length_unit_to_i32(&dimensions.padding[1], parent_width, screen_width, screen_height),
            convert_length_unit_to_i32(&dimensions.padding[2], parent_height, screen_width, screen_height),
            convert_length_unit_to_i32(&dimensions.padding[3], parent_width, screen_width, screen_height),
        ];
        if need_update_width || need_update_height{
            match &new_children{
                Children::NoChildren => {
                    panic!("Specify fit_children while there is no children");
                }
                Children::OneChild(child) => {
                    let child_dimensions = &child.box_model.dimensions;
                    if need_update_width{
                        new_width = child_dimensions.width_with_margin() + padding[1] + padding[3]; // margin_left + margin_right
                    }
                    if need_update_height{
                        new_height = child_dimensions.height_with_margin() + padding[0] + padding[2]; // margin_top + margin_bottom
                    }
                }
                Children::HorizontalLayout(children) => {
                    if need_update_width{
                        let width_sum: i32 = children.iter().map(|child| {
                            let child_dimensions = &child.box_model.dimensions;
                            child_dimensions.width_with_margin() // margin_left + margin_right
                        }).sum();
                        new_width = width_sum + padding[1] + padding[3]; // padding_left + padding_right
                    }
                    if need_update_height{
                        let height_max = children.iter().map(|child| {
                            let child_dimensions = &child.box_model.dimensions;
                            child_dimensions.height_with_margin() // margin_top + margin_bottom
                        }).max().unwrap_or_else(||{
                            println!("Warning: no children found");
                            0
                        });
                        new_height = height_max + padding[0] + padding[2]; // padding_top + padding_bottom
                    }
                }
                Children::VerticalLayout(children) => {
                    if need_update_height{
                        let height_sum: i32 = children.iter().map(|child| {
                            let child_dimensions = &child.box_model.dimensions;
                            child_dimensions.height_with_margin() // margin_top + margin_bottom
                        }).sum();
                        new_height = height_sum + padding[0] + padding[2]; // padding_top + padding_bottom
                    }
                    if need_update_width{
                        let width_max: i32 = children.iter().map(|child| {
                            let child_dimensions = &child.box_model.dimensions;
                            child_dimensions.width_with_margin() // margin_left + margin_right
                        }).max().unwrap_or_else(||{
                            println!("Warning: no children found");
                            0
                        });
                        new_width = width_max + padding[1] + padding[3]; // padding_left + padding_right
                    }
                }
                Children::GridLayout(children) => {
                    todo!()
                }
            }
        }
        let new_margin = [
            convert_length_unit_to_i32(&dimensions.margin[0], parent_height, screen_width, screen_height),
            convert_length_unit_to_i32(&dimensions.margin[1], parent_width, screen_width, screen_height),
            convert_length_unit_to_i32(&dimensions.margin[2], parent_height, screen_width, screen_height),
            convert_length_unit_to_i32(&dimensions.margin[3], parent_width, screen_width, screen_height),
        ];
        let new_padding = [
            convert_length_unit_to_i32(&dimensions.padding[0], parent_height, screen_width, screen_height),
            convert_length_unit_to_i32(&dimensions.padding[1], parent_width, screen_width, screen_height),
            convert_length_unit_to_i32(&dimensions.padding[2], parent_height, screen_width, screen_height),
            convert_length_unit_to_i32(&dimensions.padding[3], parent_width, screen_width, screen_height),
        ];
        let new_dimensions = BoxDimensions::<i32, i32>{
            width: new_width,
            height: new_height,
            margin: new_margin,
            padding: new_padding,
        };
        let new_box_model = BoxModel{
            dimensions: new_dimensions,
            h_alignment: box_model.h_alignment,
            v_alignment: box_model.v_alignment,
            color: box_model.color,
        };
        UINode {
            box_model: new_box_model,
            children: new_children,
            meta: self.meta,
        }
    }
}

impl UINode<i32, i32>{
    pub fn to_ui_renderables(
        &self, 
        canvas_order: u32,
        element_order: u32, 
        screen_width: u32, 
        screen_height: u32, 
        pos_x: i32, 
        pos_y: i32
    ) -> HashMap<UIRenderableMeta, Vec<UIInstance>>{
        let mut ui_renderables = HashMap::new();
        // current pivot
        let dimensions = &self.box_model.dimensions;
        let pos_x = pos_x + dimensions.margin[3]; // margin_left
        let pos_y = pos_y + dimensions.margin[0]; // margin_top
        let normalized_x_start = (pos_x as f32 / screen_width as f32) * 2.0 - 1.0;
        let normalized_y_start = (pos_y as f32 / screen_height as f32) * 2.0 - 1.0;
        let normalized_width = (dimensions.width as f32 / screen_width as f32) * 2.0;
        let normalized_height = (dimensions.height as f32 / screen_height as f32) * 2.0;
        let normalized_x_end = normalized_x_start + normalized_width;
        let normalized_y_end = normalized_y_start + normalized_height;
        // flip y
        let normalized_y_start = - normalized_y_start;
        let normalized_y_end = - normalized_y_end;
        let ui_instance = UIInstance {
            location: [
                normalized_x_start,
                normalized_y_start,
                normalized_x_end,
                normalized_y_end,
            ],
            color: self.box_model.color,
            sort_order: SortOrder{
                canvas_order,
                element_order,
            },
            use_texture: self.meta.uses_texture(),
        };
        ui_renderables.entry(self.meta.clone()).or_insert_with(Vec::new).push(ui_instance);
        match self.children {
            Children::NoChildren => {}
            Children::OneChild(ref child) => {
                let child_renderables = child.to_ui_renderables(canvas_order, element_order + 1, screen_width, screen_height, pos_x + dimensions.padding[3], pos_y + dimensions.padding[0]);
                for (meta, instances) in child_renderables {
                    ui_renderables.entry(meta).or_insert_with(Vec::new).extend(instances);
                }
            }
            Children::HorizontalLayout(ref children) => {
                let mut pos_x = pos_x + dimensions.padding[3]; // margin_left
                let pos_y = pos_y + dimensions.padding[0]; // margin_top
                for child in children {
                    let child_renderables = child.to_ui_renderables(canvas_order, element_order + 1, screen_width, screen_height, pos_x, pos_y);
                    for (meta, instances) in child_renderables {
                        ui_renderables.entry(meta).or_insert_with(Vec::new).extend(instances);
                    }
                    let child_dimensions = &child.box_model.dimensions;
                    pos_x += child_dimensions.width_with_margin(); // margin_left + margin_right
                }
            }
            Children::VerticalLayout(ref children) => {
                let pos_x = pos_x + dimensions.padding[3]; // padding_left
                let mut pos_y = pos_y + dimensions.padding[0]; // padding_top
                for child in children {
                    let child_renderables = child.to_ui_renderables(canvas_order, element_order + 1, screen_width, screen_height, pos_x, pos_y);
                    for (meta, instances) in child_renderables {
                        ui_renderables.entry(meta).or_insert_with(Vec::new).extend(instances);
                    }
                    let child_dimensions = &child.box_model.dimensions;
                    pos_y += child_dimensions.height_with_margin(); // margin_top + margin_bottom
                }
            }
            Children::GridLayout(ref children) => {
                todo!()
            }
        }
        ui_renderables
    }
}

pub struct Button {}
