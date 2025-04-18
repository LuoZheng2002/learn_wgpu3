// length options: fixed, depends on siblings, depends on parent, depends on children

// decide parents first, then siblings, then children

// a span wrapping several padded characters

// padding, margin, children
// width, height,

use std::{collections::HashMap, default, i32, ops::Bound};

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

impl Children<BoundedLength, DependentLength>{
    pub fn calculate_dimensions(
            self,
            parent_width: i32,
            parent_height: i32,
            screen_width: u32,
            screen_height: u32,
        )->Children<i32, i32>{
        match self{
            Children::NoChildren => Children::NoChildren,
            Children::OneChild(child) => {
                let child = child.calculate_dimensions(parent_width, parent_height, screen_width, screen_height);
                Children::OneChild(Box::new(child))
            }
            Children::HorizontalLayout(children) => {
                let new_children = children.into_iter().map(|child| {
                    child.calculate_dimensions(parent_width, parent_height, screen_width, screen_height)
                }).collect::<Vec<_>>();
                Children::HorizontalLayout(new_children)
            }
            Children::VerticalLayout(children) => {
                let new_children = children.into_iter().map(|child| {
                    child.calculate_dimensions(parent_width, parent_height, screen_width, screen_height)
                }).collect::<Vec<_>>();
                Children::VerticalLayout(new_children)
            }
            Children::GridLayout(children) => {
                todo!()
            }
        }
    }
}


#[derive(Clone)]
pub enum CanvasPadding{
    Pixels(i32),
    RelativeScreenWidth(f32),
    RelativeScreenHeight(f32),
}

#[derive(Clone)]
pub enum DependentLength{
    Pixels(i32),
    RelativeScreenWidth(f32),
    RelativeScreenHeight(f32),
    RelativeParent(f32),
}
impl DependentLength{
    pub fn zero()-> Self{
        Self::Pixels(0)
    }
}

#[derive(Clone)]
pub struct BoundedLength{
    pub preferred_length: DependentLength,
    pub min_length: Option<DependentLength>,
    pub max_length: Option<DependentLength>,
}

impl BoundedLength{
    pub fn zero()-> Self{
        Self { 
            preferred_length: DependentLength::zero(), 
            min_length: Some(DependentLength::zero()),
            max_length: Some(DependentLength::zero()),
        }
    }
    pub fn fixed_dependent(length: DependentLength)-> Self{
        Self {
                preferred_length: length.clone(),
                min_length: Some(length.clone()),
                max_length: Some(length),
        }
    }
    pub fn fixed_pixels(length: i32)-> Self{
        Self::fixed_dependent(DependentLength::Pixels(length))
    }
}

pub trait UINodeLength1{}
impl UINodeLength1 for i32{}
impl UINodeLength1 for BoundedLength{}

pub trait UINodeLength2{}
impl UINodeLength2 for i32{}
impl UINodeLength2 for DependentLength{}


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
    pub uniform_division: bool,
    pub color: cgmath::Vector4<f32>, // color is extracted from the meta because it is not a part of the bind group
}

pub struct UINode<L1: UINodeLength1, L2: UINodeLength2>{ 
    pub box_model: BoxModel<L1, L2>,
    pub children: Children<L1, L2>, // assuming horizontal layout
    pub meta: UIRenderableMeta, // contains optional texture information
}

impl UINode<BoundedLength, DependentLength>{
    pub fn calculate_dimensions(self,
        parent_width: i32, 
        parent_height: i32,
        screen_width: u32,
        screen_height: u32,
    )-> UINode<i32, i32>{
        let box_model = self.box_model;
        let dimensions = &box_model.dimensions;
        fn convert_dependent_length_to_i32(length_unit: &DependentLength, parent_length: i32, screen_width: u32, screen_height: u32)-> i32{
            match length_unit{
                DependentLength::Pixels(pixels) => *pixels,
                DependentLength::RelativeScreenWidth(relative) => (screen_width as f32 * relative) as i32,
                DependentLength::RelativeScreenHeight(relative) => (screen_height as f32 * relative) as i32,
                DependentLength::RelativeParent(relative) => (parent_length as f32 * relative) as i32,
            }
        }
        fn convert_bounded_length_to_i32(length: &BoundedLength, parent_length: i32, screen_width: u32, screen_height: u32)-> i32{
            let BoundedLength{preferred_length, min_length, max_length} = length;
            let preferred_length = convert_dependent_length_to_i32(preferred_length, parent_length, screen_width, screen_height);
            let min_length = match min_length{
                Some(length) => convert_dependent_length_to_i32(length, parent_length, screen_width, screen_height),
                None => i32::MIN,
            };
            let max_length = match max_length{
                Some(length) => convert_dependent_length_to_i32(length, parent_length, screen_width, screen_height),
                None => i32::MAX,
            };
            if min_length > max_length{
                panic!("min length is greater than max length");
            }
            // clamp preferred length to min and max
            preferred_length.clamp(min_length, max_length)
        }
        // result
        
        let width = convert_bounded_length_to_i32(
            &dimensions.width,
            parent_width,
            screen_width,
            screen_height,
        );
        let height = convert_bounded_length_to_i32(
            &dimensions.height,
            parent_height,
            screen_width,
            screen_height,
        );

        let children: Children<i32, i32> = self.children.calculate_dimensions(
            width,
            height,
            screen_width,
            screen_height,
        );

        let margin = [
            convert_dependent_length_to_i32(&dimensions.margin[0], parent_height, screen_width, screen_height),
            convert_dependent_length_to_i32(&dimensions.margin[1], parent_width, screen_width, screen_height),
            convert_dependent_length_to_i32(&dimensions.margin[2], parent_height, screen_width, screen_height),
            convert_dependent_length_to_i32(&dimensions.margin[3], parent_width, screen_width, screen_height),
        ];
        let padding = [
            convert_dependent_length_to_i32(&dimensions.padding[0], parent_height, screen_width, screen_height),
            convert_dependent_length_to_i32(&dimensions.padding[1], parent_width, screen_width, screen_height),
            convert_dependent_length_to_i32(&dimensions.padding[2], parent_height, screen_width, screen_height),
            convert_dependent_length_to_i32(&dimensions.padding[3], parent_width, screen_width, screen_height),
        ];
        let dimensions = BoxDimensions::<i32, i32>{
            width,
            height,
            margin,
            padding,
        };
        let box_model = BoxModel{
            dimensions,
            h_alignment: box_model.h_alignment,
            v_alignment: box_model.v_alignment,
            color: box_model.color,
            uniform_division: box_model.uniform_division,
        };
        UINode {
            box_model,
            children,
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
