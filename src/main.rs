use learn_wgpu3::app::App;
use learn_wgpu3::cache::{CacheValue, get_font};
use learn_wgpu3::ui::{Char, ToUINode};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

fn main() {
    let font = get_font("assets/times.ttf".to_string());
    let font = match font.as_ref() {
        CacheValue::Font(font) => font,
        _ => panic!("Invalid cache value"),
    };
    let character = Char {
        font: &font,
        scale: 1024.0,
        character: 'g',
    };
    let ui_node = character.to_ui_node();

    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    let _ = event_loop.run_app(&mut app);
}
