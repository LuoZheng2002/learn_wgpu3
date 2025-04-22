use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::{render_context::RenderContext, state::State};

#[derive(Default)]
pub struct App {
    pub window: Option<Arc<Window>>,
    pub render_context: Option<RenderContext>,
    pub state: State,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();
        let window = Arc::new(window);
        self.render_context = Some(RenderContext::new(window.clone()));
        self.window = Some(window);
        self.state.init();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap().request_redraw();
                self.state
                    .update(&self.render_context.as_ref().unwrap().size);
                match self
                    .render_context
                    .as_mut()
                    .unwrap()
                    .render(&mut self.state)
                {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let new_size = self.render_context.as_ref().unwrap().size;
                        self.render_context.as_mut().unwrap().resize(new_size);
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory | wgpu::SurfaceError::Other) => {
                        log::error!("OutOfMemory");
                        event_loop.exit();
                    }

                    // This happens when the a frame takes too long to present
                    Err(wgpu::SurfaceError::Timeout) => {
                        log::warn!("Surface timeout")
                    }
                }
            }
            WindowEvent::Resized(new_size) => {
                self.render_context.as_mut().unwrap().resize(new_size);
            }
            _ => (),
        }
    }
}
