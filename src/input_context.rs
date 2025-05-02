use std::{collections::HashMap, result};
use winit::{
    event::{DeviceEvent, ElementState, MouseButton, WindowEvent},
    keyboard::{Key, KeyCode, NamedKey, PhysicalKey},
};
#[derive(Default)]
pub struct InputContext {
    key_states: HashMap<KeyCode, bool>,
    key_down_flags: HashMap<KeyCode, bool>,
    key_up_flags: HashMap<KeyCode, bool>,
    current_key_down: Option<KeyCode>,
    mouse_left: bool,
    mouse_left_pressed_flag: bool,
    mouse_left_released_flag: bool,
    mouse_right: bool,
    mouse_right_pressed_flag: bool,
    mouse_right_released_flag: bool,
    cursor_position: Option<(f64, f64)>,
    device_mouse_delta_accumulated: (f64, f64),
    pressed_str: Option<String>,
}

impl InputContext {
    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        // self.current_key_down = None;
        match event {
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                let physical_key = event.physical_key;
                if let PhysicalKey::Code(key_code) = physical_key {
                    println!("Key event: {:?}", key_code);
                    match event.state {
                        ElementState::Pressed => {
                            self.current_key_down = Some(key_code);
                            let prev_state = self.key_states.insert(key_code, true);
                            let prev_pressed = prev_state.unwrap_or(false);
                            if !prev_pressed {
                                self.key_down_flags.insert(key_code, true);
                            }
                            self.key_up_flags.insert(key_code, false);
                        }
                        ElementState::Released => {
                            let prev_state = self.key_states.insert(key_code, false);
                            let prev_pressed = prev_state.unwrap_or(false);
                            if prev_pressed {
                                self.key_up_flags.insert(key_code, true);
                            }
                            self.key_down_flags.insert(key_code, false);
                        }
                    }
                }
                self.pressed_str = event.text.as_ref().map(|c| c.to_string());
            }
            WindowEvent::MouseInput { state, button, .. } => {
                fn handle_mouse_pressed(
                    mouse: &mut bool,
                    mouse_pressed_flag: &mut bool,
                    mouse_released_flag: &mut bool,
                ) {
                    if !*mouse {
                        *mouse_pressed_flag = true;
                    }
                    *mouse = true;
                    *mouse_released_flag = false;
                }
                fn handle_mouse_released(
                    mouse: &mut bool,
                    mouse_pressed_flag: &mut bool,
                    mouse_released_flag: &mut bool,
                ) {
                    if *mouse {
                        *mouse_released_flag = true;
                    }
                    *mouse = false;
                    *mouse_pressed_flag = false;
                }
                match button {
                    MouseButton::Left => match state {
                        ElementState::Pressed => {
                            handle_mouse_pressed(
                                &mut self.mouse_left,
                                &mut self.mouse_left_pressed_flag,
                                &mut self.mouse_left_released_flag,
                            );
                        }
                        ElementState::Released => {
                            handle_mouse_released(
                                &mut self.mouse_left,
                                &mut self.mouse_left_pressed_flag,
                                &mut self.mouse_left_released_flag,
                            );
                        }
                    },
                    MouseButton::Right => match state {
                        ElementState::Pressed => {
                            handle_mouse_pressed(
                                &mut self.mouse_right,
                                &mut self.mouse_right_pressed_flag,
                                &mut self.mouse_right_released_flag,
                            );
                        }
                        ElementState::Released => {
                            handle_mouse_released(
                                &mut self.mouse_right,
                                &mut self.mouse_right_pressed_flag,
                                &mut self.mouse_right_released_flag,
                            );
                        }
                    },
                    _ => {}
                }
            }
            WindowEvent::CursorMoved { device_id: _, position }=> {
                self.cursor_position = Some((position.x, position.y));
            }
            _ => {}
        }
    }

    pub fn handle_device_event(&mut self, event: &DeviceEvent) {
        // handle device events here
        match event {
            DeviceEvent::MouseMotion { delta } => {
                // println!("Received mouse motion: {:?}", delta);
                let old = self.device_mouse_delta_accumulated;
                self.device_mouse_delta_accumulated = (delta.0 + old.0, delta.1 + old.1);
            }
            _ => {}
        }
    }

    pub fn get_key_down(&mut self, key: KeyCode) -> bool {
        self.key_down_flags.insert(key, false).unwrap_or(false)
    }

    pub fn get_key(&mut self, key: KeyCode) -> bool {
        self.key_states.entry(key).or_insert(false).clone()
    }
    pub fn get_current_key_down(&mut self) -> Option<KeyCode> {
        let result = self.current_key_down.clone();
        self.current_key_down = None;
        result
    }

    pub fn get_key_up(&mut self, key: KeyCode) -> bool {
        self.key_up_flags.insert(key, false).unwrap_or(false)
    }

    pub fn mouse_left_down(&mut self) -> bool {
        let result = self.mouse_left_pressed_flag;
        self.mouse_left_pressed_flag = false;
        result
    }

    pub fn mouse_left(&mut self) -> bool {
        self.mouse_left
    }

    pub fn mouse_left_up(&mut self) -> bool {
        let result = self.mouse_left_released_flag;
        self.mouse_left_released_flag = false;
        result
    }

    pub fn mouse_right_down(&mut self) -> bool {
        let result = self.mouse_right_pressed_flag;
        self.mouse_right_pressed_flag = false;
        result
    }

    pub fn mouse_right(&mut self) -> bool {
        self.mouse_right
    }

    pub fn mouse_right_up(&mut self) -> bool {
        let result = self.mouse_right_released_flag;
        self.mouse_right_released_flag = false;
        result
    }

    pub fn mouse_position(&self) -> Option<(f64, f64)> {
        self.cursor_position
    }
    pub fn device_mouse_delta_accumulated(&mut self) -> (f64, f64) {
        self.device_mouse_delta_accumulated
    }
    pub fn get_pressed_str(&mut self) -> Option<String> {
        let result = self.pressed_str.clone();
        self.pressed_str = None;
        result
    }
}
