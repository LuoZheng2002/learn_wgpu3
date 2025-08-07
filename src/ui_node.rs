// length options: fixed, depends on parent, depends on children

// decide parents first, then siblings, then children

// a span wrapping several padded characters

// padding, margin, children
// width, height,

use std::{any::TypeId, collections::HashMap, i32, sync::{Arc, Mutex, RwLock, Weak}, vec};

use either::Either;
use lazy_static::lazy_static;
use winit::keyboard::KeyCode;

use crate::{my_texture::MyTexture, state, ui::UICell, ui_renderable::TextureMeta};

// handle event -> state change -> render

// each ui is a rectangular region, with its own texture and a cached texture that includes its children

// each ui must have: the calculated size and position (from last frame), the target relative size, position, margin, padding (when they change, the ui is invalidated)
// each ui must have: a callback function that changes the ui's target relative size

// ui object -> ui node (that handles events but does not calculate positions and sizes, invalidates ui parameters / texture if necessary) -> 

// the cached texture of a ui is invalidated if its size, texture, or children count, or children's cached texture, or the array layout is invalidated

// ui node needs a pointer to the optional cached texture
// if it is none, it will be created by rendering its own texture, and childrens' cached textures which depends on its size, childrens' sizes, childrens' positions, and childrens' clip boxes (relative)

// the ui can be updated either from code or from events
// we want to directly call the methods of a ui, and mark it as invalidated
// we want to make its parents to be invalidated too, which is a separate pass
// if a component is not invalidated, then its size and positions are also not invalidated, and all its children can be ignored, meaning we don't need to calculate their positions and sizes

// every ui needs to implement ToUINode
// StructuredChildren can be reduced to horizontal and vertical layouts

// ui node needs to include: optional cached texture, texture meta, relative box dimensions, cached absolute box dimensions, structured children, 
// event handler, 
// basically each ui element stores a mutable ui node, 


/// ui node pass: ui to ui node -> 
/// handle event (without invalidating self) -> 
/// invalidate nodes because of texture meta ->
/// calculate all components' dimensions (translate dependencies to pointers -> turn into virtual boxes (with boxs' width and height to be expressions) -> flatten -> solve iteratively) ->
/// propagate invalidation from children to parents
///  -> calculate dimensions (for invalidated components) -> into render instruction
pub trait ToUINode {
    fn to_ui_node(
        &self,
    ) -> UINode;
    fn update_and_to_instruction(
        &self,        
        screen_width: u32,
        screen_height: u32,
        event: &UINodeEvent,
    )->UIRenderInstruction{
        let ui_node = self.to_ui_node();
        let ui_node = ui_node.calculate_dimensions(screen_width, screen_height, screen_width, screen_height);
        let ui_node = ui_node.flatten_children(
            0, 0, 
            screen_width, 
            screen_height, 
            HorizontalAlignment::Left, 
        VerticalAlignment::Top);
        let ui_node = ui_node.to_unified();
        ui_node.handle_event(event);
        let ui_node = self.to_ui_node();
        let ui_node = ui_node.calculate_dimensions(screen_width, screen_height, screen_width, screen_height);
        let ui_node = ui_node.flatten_children(
            0, 0, 
            screen_width, 
            screen_height, 
            HorizontalAlignment::Left, 
        VerticalAlignment::Top);
        let ui_node = ui_node.to_unified();
        ui_node.to_ui_render_instruction(screen_width, screen_height)
    }
}


#[derive(Clone, Copy)]
pub enum HorizontalAlignment {
    Left,
    Center,
    Right,
}
#[derive(Clone, Copy)]
pub enum VerticalAlignment {
    Top,
    Center,
    Bottom,
}

#[derive(Clone, Copy)]
pub enum ChildrenLayout{
    Horizontal,
    Vertical,
}

pub struct StructuredChildren{   
    h_alignment: HorizontalAlignment,
    v_alignment: VerticalAlignment,
    uniform_division: bool,
    layout: ChildrenLayout,
    children: Vec<UINode>,
}

// todo: not sure if putting alignment here is a good idea

/// it means that the UINode that owns this struct is actually the content, so it has alignment information for calculating
/// its position with respect to the parent (cell)
pub struct ChildrenAreDummyCells {
    cells: Vec<DummyCell>,
} //an ui node that owns the "Cells" struct is actually the content
// for each UINode<u32, u32, Content> we need to add position information
// a cell has a fixed size and position, but the content doesn't

// for a content to be rendered inside a cell, we need to know:
// cell width and height
// content width and height
// alignment

pub struct ChildIsContent {
    cell_rel_pos_x: u32, // top left corner relative to parent
    cell_rel_pos_y: u32,
    cell_width: u32,
    cell_height: u32,
    content: UINode<BoxDimensionsWithGlobal, ChildrenAreDummyCells>,
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
        }
    }
}

#[derive(Clone, Copy)]
pub enum DependentLength {
    Pixels(u32),
    RelativeScreenWidth(f32),
    RelativeScreenHeight(f32),
    RelativeParentWidth(f32),
    RelativeParentHeight(f32),
}
impl DependentLength {
    pub fn zero() -> Self {
        Self::Pixels(0)
    }
}

#[derive(Clone)]
pub struct BoundedLength {
    pub preferred_length: DependentLength,
    pub min_length: Option<DependentLength>,
    pub max_length: Option<DependentLength>,
}

impl BoundedLength {
    pub fn zero() -> Self {
        Self {
            preferred_length: DependentLength::zero(),
            min_length: Some(DependentLength::zero()),
            max_length: Some(DependentLength::zero()),
        }
    }
    pub fn fixed_dependent(length: DependentLength) -> Self {
        Self {
            preferred_length: length.clone(),
            min_length: Some(length.clone()),
            max_length: Some(length),
        }
    }
    pub fn fixed_pixels(length: u32) -> Self {
        Self::fixed_dependent(DependentLength::Pixels(length))
    }
}

pub trait UINodeLength1 {}
impl UINodeLength1 for u32 {}
impl UINodeLength1 for BoundedLength {}

pub trait UINodeLength2 {}
impl UINodeLength2 for u32 {}
impl UINodeLength2 for DependentLength {}

pub trait UIChildren<B: BoxDimensions> {}

impl<B: BoxDimensions> UIChildren<B> for StructuredChildren<B> {}
impl UIChildren<BoxDimensionsWithGlobal> for ChildrenAreDummyCells {}
impl UIChildren<BoxDimensionsWithGlobal> for ChildIsContent {}
impl UIChildren<BoxDimensionsWithGlobal> for UnifiedChildren {}

#[derive(Clone)]
pub struct BoxDimensionsRelative {
    pub width: BoundedLength,
    pub height: BoundedLength,
    pub margin: [DependentLength; 4],  // top, right, bottom, left
    pub padding: [DependentLength; 4], // top, right, bottom, left
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
pub enum ComponentIdentifier{
    Default{
        id: u64,
        name: String,
    },
    Char{
        character: char,
        font_path: String,
        show_cursor: bool,
    },
    DummyChar{
        show_cursor: bool,
    }
}

impl ComponentIdentifier{
    pub fn to_string(&self) -> String {
        match self {
            ComponentIdentifier::Default{id, name} => format!("Component: {}: {}", id, name),
            ComponentIdentifier::Char{character, font_path, show_cursor} => format!("Char: {}: {}, show_cursor: {}", character, font_path, show_cursor),
            ComponentIdentifier::DummyChar{show_cursor} => format!("DummyChar: show_cursor: {}", show_cursor),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum UIIdentifier {
    Component(ComponentIdentifier),
    Cell{parent: ComponentIdentifier, index: u64},
}
impl UIIdentifier {
    pub fn to_string(&self) -> String {
        match self {
            UIIdentifier::Component(id) => format!("Component: {}", id.to_string()),
            UIIdentifier::Cell{parent, index} => format!("Cell: {}[{}]", parent.to_string(), index),
        }
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

// pub struct UINode<B: BoxDimensions, C: UIChildren<B>> {
//     pub box_dimensions: B,
//     pub children: C,       // assuming horizontal layout
//     pub texture_meta: TextureMeta, // contains optional texture information
//     pub identifier: UIIdentifier,
//     pub render_version: u64,
//     pub event_handler: Option<Box<dyn Fn(&UINodeEventProcessed)->bool>>,
//     pub render_state_changed_handler: Option<Box<dyn Fn()>>,
// }

pub enum Expression{
    None,
    Constant(i32),
    Sum2(Weak<RwLock<Expression>>, Weak<RwLock<Expression>>),
    Sum3(Weak<RwLock<Expression>>, Weak<RwLock<Expression>>, Weak<RwLock<Expression>>),
    Diff2(Weak<RwLock<Expression>>, Weak<RwLock<Expression>>),
    Diff3(Weak<RwLock<Expression>>, Weak<RwLock<Expression>>, Weak<RwLock<Expression>>),
    Mul(Weak<RwLock<Expression>>, f32),
}

// need a smart expression type that references other expressions and record expressions that referece self
// if one expression changes, it will notify all the expressions that reference it
// if expressions corresponding to a node's width and height are changed, the node will be invalidated

// an expression can be changed if the expression itself changes or its value changes
pub struct CachedDimensions{
    pub width: u32,
    pub height: u32,
    pub rel_x: i32,
    pub rel_y: i32,
    pub global_x: i32,
    pub global_y: i32,
    pub bound_x: i32,
    pub bound_y: i32,
    pub bound_width: u32,
    pub bound_height: u32,
    pub rel_x_inside_box: i32,
    pub rel_y_inside_box: i32,
    // previous record of width and height for determining if the size has changed
    pub prev_width: u32,
    pub prev_height: u32,
}

pub struct DependentDimensions{
    pub width: DependentLength,
    pub height: DependentLength,
    pub margin: [i32; 4],
    pub padding: [i32; 4],
}
// new problem: previously use margin to implement the offset of scroll view
// hopefully if the right and bottom margin are set to 0, it will not affect the size of the box

pub struct UINodeEssentials{
    pub cached_dimensions: CachedDimensions,
    pub dependent_dimensions: DependentDimensions,
    pub cached_texture: Option<MyTexture>,
    pub texture_meta: TextureMeta,
    // previous texture meta for determining if the texture has changed
    pub prev_texture_meta: TextureMeta,
}
// event handler can change: texture meta (button), dimensions (draggable split bar, but with constraints)

// we may need extra ui's state to handle events



// the event handler needs to capture both UINodeEssentials and a mutable state of the UI.


// uinode itself is disposable
// everything that is mutable should be in the RwLock
pub struct UINode{
    pub essentials: Weak<RwLock<UINodeEssentials>>, // it has to modify the dimensions of the UI
    pub children: StructuredChildren,
    pub event_handler: Option<Weak<dyn Fn(&UINodeEvent)>>,
}

pub struct VirtualBox{
    pub children: Vec<VirtualBox>,
    pub box_rel_x: Arc<RwLock<Expression>>, // this can be determined
    pub box_rel_y: Arc<RwLock<Expression>>,
    pub box_width: Arc<RwLock<Expression>>,
    pub box_height: Arc<RwLock<Expression>>,
    // pub horizontal_alignment: HorizontalAlignment,
    // pub vertical_alignment: VerticalAlignment,
}
pub struct UINodeWithBoundedChildren{
    pub essentials: Weak<RwLock<UINodeEssentials>>,
    pub children: Vec<VirtualBox>,
    pub event_handler: Option<Weak<dyn Fn(&UINodeEvent)>>,
}

impl UINode{
    pub fn handle_event(&self, event: &UINodeEvent){
        if let Some(handler) = self.event_handler.as_ref(){
            let handler = handler.upgrade().unwrap();
            handler(event);
        }
        for child in self.children.children.iter(){
            child.handle_event(event);
        }
    }

    pub fn invalidate_by_texture_meta(&self){

    }

    /// if child is invalidated, parent must be invalidated because the texture changes
    /// but what about the child's siblings? -> it depends on whether the dimensions of the child is changed
    /// if a component's size changes, it is invalidated
    /// so a component is invalidated if its size changes, its texture meta changes, or its children changes
    /// so we need to calculate the size of the children first 
    /// early stop criteria: if previous size == new size
    /// maybe separate texture invalidation and size invalidation?
    /// a component needs to redraw if its size changes, or its texture meta changes, or its children changes
    /// easy solution for size changes: recalculate all the components' sizes and positions
    /// and if size changes between frames, invalidate the texture
    /// this function does not take account of invalidation caused by size changes
    pub fn propagate_invalidation_from_children(&self)->bool{
        let mut invalidate_self = false;
        let children = &self.children.children;
        for child in children.iter(){
            if child.propagate_invalidation_from_children(){
                invalidate_self = true;
            }
        }
        let essentials = self.essentials.upgrade().unwrap();
        let mut essentials = essentials.write().unwrap();
        if invalidate_self{
            essentials.cached_texture = None; // invalidate the texture
        }
        essentials.cached_texture.is_none()
    }
    /// references: dependent dimensions
    /// produces: cached dimensions in expression form
    pub fn translate_dependencies_to_pointers(&self){

    }
    /// references: dependent dimensions
    /// produces: width, height (referencing dependent width and height), rel_x_inside_box, rel_y_inside_box (referecing margin)
    /// 
    pub fn calculate_dimensions(&self){

    }
    pub fn add_bounding_box(
        self,
        box_rel_x: i32,
        box_rel_y: i32,
        box_width: u32,
        box_height: u32,
        horizontal_alignment: HorizontalAlignment,
        vertical_alignment: VerticalAlignment,
    )-> VirtualBox{


        VirtualBox { 
            ui_node: (), 
            box_rel_x,
            box_rel_y,
            box_width,
            box_height, 
            horizontal_alignment, 
            vertical_alignment 
        }
    }
}


// impl UINode<BoxDimensionsRelative, StructuredChildren<BoxDimensionsRelative>> {
//     pub fn calculate_dimensions(
//         self,
//         parent_width: u32,
//         parent_height: u32,
//         screen_width: u32,
//         screen_height: u32,
//     ) -> UINode<BoxDimensionsAbsolute, StructuredChildren<BoxDimensionsAbsolute>> {
//         let dimensions = &self.box_dimensions;
//         fn convert_dependent_length_to_u32(
//             length_unit: &DependentLength,
//             parent_width: u32,
//             parent_height: u32,
//             screen_width: u32,
//             screen_height: u32,
//         ) -> u32 {
//             match length_unit {
//                 DependentLength::Pixels(pixels) => *pixels,
//                 DependentLength::RelativeScreenWidth(relative) => {
//                     (screen_width as f32 * relative) as u32
//                 }
//                 DependentLength::RelativeScreenHeight(relative) => {
//                     (screen_height as f32 * relative) as u32
//                 }
//                 DependentLength::RelativeParentWidth(relative) => {
//                     (parent_width as f32 * relative) as u32
//                 }
//                 DependentLength::RelativeParentHeight(relative) => {
//                     (parent_height as f32 * relative) as u32
//                 }
//             }
//         }
//         fn convert_bounded_length_to_u32(
//             length: &BoundedLength,
//             parent_width: u32,
//             parent_height: u32,
//             screen_width: u32,
//             screen_height: u32,
//         ) -> u32 {
//             let BoundedLength {
//                 preferred_length,
//                 min_length,
//                 max_length,
//             } = length;
//             let preferred_length = convert_dependent_length_to_u32(
//                 preferred_length,
//                 parent_width,
//                 parent_height,
//                 screen_width,
//                 screen_height,
//             );
//             let min_length = match min_length {
//                 Some(length) => convert_dependent_length_to_u32(
//                     length,
//                     parent_width,
//                     parent_height,
//                     screen_width,
//                     screen_height,
//                 ),
//                 None => u32::MIN,
//             };
//             let max_length = match max_length {
//                 Some(length) => convert_dependent_length_to_u32(
//                     length,
//                     parent_width,
//                     parent_height,
//                     screen_width,
//                     screen_height,
//                 ),
//                 None => u32::MAX,
//             };
//             if min_length > max_length {
//                 panic!("min length is greater than max length");
//             }
//             // clamp preferred length to min and max
//             preferred_length.clamp(min_length, max_length)
//         }
//         // result

//         let width = convert_bounded_length_to_u32(
//             &dimensions.width,
//             parent_width,
//             parent_height,
//             screen_width,
//             screen_height,
//         );
//         let height = convert_bounded_length_to_u32(
//             &dimensions.height,
//             parent_width,
//             parent_height,
//             screen_width,
//             screen_height,
//         );

//         let children: StructuredChildren<BoxDimensionsAbsolute> = self
//             .children
//             .calculate_dimensions(width, height, screen_width, screen_height);

//         let margin = [
//             convert_dependent_length_to_u32(
//                 &dimensions.margin[0],
//                 parent_width,
//                 parent_height,
//                 screen_width,
//                 screen_height,
//             ),
//             convert_dependent_length_to_u32(
//                 &dimensions.margin[1],
//                 parent_width,
//                 parent_height,
//                 screen_width,
//                 screen_height,
//             ),
//             convert_dependent_length_to_u32(
//                 &dimensions.margin[2],
//                 parent_width,
//                 parent_height,
//                 screen_width,
//                 screen_height,
//             ),
//             convert_dependent_length_to_u32(
//                 &dimensions.margin[3],
//                 parent_width,
//                 parent_height,
//                 screen_width,
//                 screen_height,
//             ),
//         ];
//         let padding = [
//             convert_dependent_length_to_u32(
//                 &dimensions.padding[0],
//                 parent_width,
//                 parent_height,
//                 screen_width,
//                 screen_height,
//             ),
//             convert_dependent_length_to_u32(
//                 &dimensions.padding[1],
//                 parent_width,
//                 parent_height,
//                 screen_width,
//                 screen_height,
//             ),
//             convert_dependent_length_to_u32(
//                 &dimensions.padding[2],
//                 parent_width,
//                 parent_height,
//                 screen_width,
//                 screen_height,
//             ),
//             convert_dependent_length_to_u32(
//                 &dimensions.padding[3],
//                 parent_width,
//                 parent_height,
//                 screen_width,
//                 screen_height,
//             ),
//         ];
//         let box_dimensions = BoxDimensionsAbsolute {
//             width,
//             height,
//             margin,
//             padding,
//         };
//         UINode {
//             box_dimensions,
//             children,
//             texture_meta: self.texture_meta,
//             identifier: self.identifier,
//             render_version: self.render_version,
//             event_handler: self.event_handler,
//             render_state_changed_handler: self.render_state_changed_handler,
//         }
//     }
// }

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

// a dummy cell that is not a uinode, and contains information about the cell's position and size, and the uinode inside it

pub struct DummyCell{
    pub rel_pos_x: u32, // top left corner relative to parent
    pub rel_pos_y: u32,
    pub global_pos_x: u32,
    pub global_pos_y: u32,
    pub width: u32,
    pub height: u32,
    pub h_alignment: HorizontalAlignment,
    pub v_alignment: VerticalAlignment,
    pub content: UINode<BoxDimensionsWithGlobal, ChildrenAreDummyCells>,
}


impl UINode<BoxDimensionsAbsolute, StructuredChildren<BoxDimensionsAbsolute>> {
    fn wrap_node_with_cell(
        self,
        cell_width: u32,
        cell_height: u32,
        cell_rel_x: u32,
        cell_rel_y: u32,
        parent_global_x: u32,
        parent_global_y: u32,
        h_alignment: HorizontalAlignment, // the alignment of the content with respect to the cell
        v_alignment: VerticalAlignment,
        // parent_id: ComponentIdentifier,
        // cell_index: u64,
        // parent_version: u64,
    ) -> DummyCell {
        let cell_global_x = parent_global_x + cell_rel_x;
        let cell_global_y = parent_global_y + cell_rel_y;
        DummyCell { 
            rel_pos_x: cell_rel_x, 
            rel_pos_y: cell_rel_y, 
            global_pos_x: cell_global_x, 
            global_pos_y: cell_global_y, 
            width: cell_width,
            height: cell_height,
            h_alignment,
            v_alignment,
            content: self.flatten_children(
                cell_global_x,
                cell_global_y,
                cell_width,
                cell_height,
                h_alignment,
                v_alignment,
            ),
        }
    }
    fn get_cell_lengths_and_positions_tangent_dir(
        total_length: u32,
        children_lengths: Vec<u32>,
        parent_padding: u32,
        uniform_division: bool,
        alignment: Either<HorizontalAlignment, VerticalAlignment>,
    ) -> Vec<(u32, u32)> {
        if uniform_division {
            let num_children = children_lengths.len();
            let num_children = usize::min(num_children, 1);
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

    fn children_array_to_dummy_cells(){
        // to do
        todo!()
    }
    pub fn flatten_children(
        self,
        cell_global_x: u32, // assume it does not take into account parent's padding
        cell_global_y: u32,
        cell_width: u32,
        cell_height: u32,
        h_alignment: HorizontalAlignment, // if the outermost node is a canvas that covers the entire screen, it does not matter
        v_alignment: VerticalAlignment,
    ) -> UINode<BoxDimensionsWithGlobal, ChildrenAreDummyCells> {
        let UINode {
            box_dimensions,
            children,
            texture_meta: meta,
            identifier,
            render_version: version,
            event_handler,
            render_state_changed_handler: state_changed_handler
        } = self;
        let parent_id = match &identifier {
            UIIdentifier::Component(id) => id,
            _ => unreachable!(),
        };
        let width_difference = cell_width as i32 - box_dimensions.width_with_margin() as i32;
        let width_difference = i32::max(width_difference, 0) as u32;
        let height_difference = cell_height as i32 - box_dimensions.height_with_margin() as i32;
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
        let self_global_x = cell_global_x + self_rel_x;
        let self_global_y = cell_global_y + self_rel_y;
        let children: ChildrenAreDummyCells = match children {
            StructuredChildren::NoChildren => {
                ChildrenAreDummyCells {
                    cells: vec![],
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
                    h_alignment,
                    v_alignment,
                );
                ChildrenAreDummyCells {
                    cells: vec![cell],
                }
            }
            StructuredChildren::HorizontalLayout {
                h_alignment,
                v_alignment,
                uniform_division,
                children,
            } => {
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
                        .enumerate()
                        .map(
                            |(i,(child, ((cell_width, cell_pos_x), (cell_height, cell_pos_y))))| {
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
                    ChildrenAreDummyCells {
                        cells,
                    }
            }
            StructuredChildren::VerticalLayout {
                h_alignment,
                v_alignment,
                uniform_division,
                children,
            } => {
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
                        .enumerate()
                        .map(
                            |(i, (child, ((cell_width, cell_pos_x), (cell_height, cell_pos_y))))| {
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
                    ChildrenAreDummyCells {
                        cells,
                    }
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
            texture_meta: meta,
            identifier,
            render_version: version,
            event_handler,
            render_state_changed_handler: state_changed_handler,
        }
    }
}

impl UINode<BoxDimensionsWithGlobal, ChildrenAreDummyCells> {
    pub fn to_unified(self) -> UINode<BoxDimensionsWithGlobal, UnifiedChildren> {
        let UINode {
            box_dimensions,
            children,
            texture_meta: meta,
            identifier,
            render_version: version,
            event_handler,
            render_state_changed_handler: state_changed_handler,
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
            texture_meta: meta,
            identifier,
            render_version: version,
            event_handler,
            render_state_changed_handler: state_changed_handler,
        }
    }
}
impl UINode<BoxDimensionsWithGlobal, ChildIsContent> {
    pub fn to_unified(self) -> UINode<BoxDimensionsWithGlobal, UnifiedChildren> {
        let UINode {
            box_dimensions,
            children,
            texture_meta: meta,
            identifier,
            render_version: version,
            event_handler,
            render_state_changed_handler: state_changed_handler,
        } = self;
        let children = vec![children.content.to_unified()];
        let children = UnifiedChildren { children };
        UINode {
            box_dimensions,
            children,
            texture_meta: meta,
            identifier,
            render_version: version,
            event_handler,
            render_state_changed_handler: state_changed_handler,
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
            texture_meta: meta,
            identifier,
            render_version: version,
            event_handler: _,
            render_state_changed_handler: _,
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
            texture_meta: meta,
            identifier,
            render_version: version,
            event_handler: _,
            render_state_changed_handler: _,
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
    fn process_event(&self, event: &UINodeEvent) -> UINodeEventProcessed {
        let box_dimensions = &self.box_dimensions;
        let mouse_hover_left_half = 
            event.mouse_x >= box_dimensions.global_pos_x
            && event.mouse_x < box_dimensions.global_pos_x + box_dimensions.width/2
            && event.mouse_y >= box_dimensions.global_pos_y
            && event.mouse_y < box_dimensions.global_pos_y + box_dimensions.height;
        let mouse_hover_right_half =
            event.mouse_x >= box_dimensions.global_pos_x + box_dimensions.width/2
            && event.mouse_x < box_dimensions.global_pos_x + box_dimensions.width
            && event.mouse_y >= box_dimensions.global_pos_y
            && event.mouse_y < box_dimensions.global_pos_y + box_dimensions.height;
        let mouse_hover = mouse_hover_left_half || mouse_hover_right_half;
        let left_clicked_left_half = mouse_hover_left_half && event.mouse_left_down;
        let left_clicked_right_half = mouse_hover_right_half && event.mouse_left_down;
        let left_clicked_inside = mouse_hover && event.mouse_left_down;
        let right_clicked_inside = mouse_hover && event.mouse_right_down;
        let lose_focus = !mouse_hover && (event.mouse_left_down || event.mouse_right_down);
        let left_released = event.mouse_left_up;
        let right_released = event.mouse_right_up;
        let pressed_str = event.pressed_str.clone();
        if let Some(keycode) = event.key_down {
            println!("key in processed pressed: {:?}", keycode);
        }
        UINodeEventProcessed { 
            left_clicked_inside, 
            left_released, 
            right_clicked_inside, 
            right_released, 
            mouse_hover, 
            lose_focus, 
            key_down: event.key_down, 
            cursor_blink: event.cursor_blink, 
            left_clicked_left_half, 
            left_clicked_right_half,
            pressed_str,
        }
    }
    /// the return value specifies whether the current UI element and its parent have a state change
    pub fn handle_event(&self, event: &UINodeEvent)->bool{
        
        let event_processed = self.process_event(event);
        let mut state_changed = false;
        if let Some(event_handler) = &self.event_handler {
            state_changed  = event_handler(&event_processed) || state_changed;
        }        
        for child in self.children.children.iter() {
            state_changed = child.handle_event(event) || state_changed;
        }
        if state_changed{
            if let Some(state_changed_handler) = &self.render_state_changed_handler {
                state_changed_handler();
            }
        }
        state_changed
    }
}


pub struct UINodeEvent{
    pub mouse_x: u32,
    pub mouse_y: u32,
    pub mouse_left: bool,
    pub mouse_right: bool,
    pub mouse_left_down: bool, // the frame that the button changes from up to down
    pub mouse_right_down: bool,
    pub mouse_left_up: bool, // the frame that the button changes from down to up
    pub mouse_right_up: bool,
    pub key_down: Option<KeyCode>, // the frame that the key changes from up to down
    pub cursor_blink: bool, // the frame that the cursor blinks
    pub pressed_str: Option<String>,
}

pub struct UINodeEventProcessed{
    pub left_clicked_inside: bool, // whether the mouse left button changes from up to down inside the element
    pub left_released: bool, // whether the mouse left button changes from down to up
    pub right_clicked_inside: bool, // whether the mouse right button changes from up to down inside the element
    pub right_released: bool, // whether the mouse right button changes from down to up
    pub mouse_hover: bool, // whether the mouse is inside the element
    pub lose_focus: bool, // whether the mouse is left clicked / right clicked outside the element
    pub key_down: Option<KeyCode>, // the key that is pressed down
    pub cursor_blink: bool, // the frame that the cursor blinks
    pub left_clicked_left_half: bool,
    pub left_clicked_right_half: bool,
    pub pressed_str: Option<String>, // the string that is pressed
}
