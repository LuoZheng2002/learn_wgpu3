use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    dpi::LogicalPosition,
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

use crate::{input_context::InputContext, render_context::RenderContext, state::State};

#[derive(Default)]
pub struct App {
    pub window: Option<Arc<Window>>,
    pub render_context: Option<RenderContext>,
    pub state: State,
    pub input_context: InputContext,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = WindowAttributes::default()
            .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
            .with_position(LogicalPosition::new(0, 0));
        let window = event_loop.create_window(attributes).unwrap();
        let window = Arc::new(window);
        self.render_context = Some(RenderContext::new(window.clone()));
        self.window = Some(window);
        self.state.init();
    }
    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        self.input_context.handle_device_event(&event);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        self.input_context.handle_window_event(&event);
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap().request_redraw();
                self.state
                    .update(&mut self.input_context, &self.render_context.as_ref().unwrap().size);
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
