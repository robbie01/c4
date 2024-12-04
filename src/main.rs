use state::State;
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::{ControlFlow, EventLoop}, window::Window};

mod camera;
mod state;
mod board;

#[derive(Debug, Default)]
struct App {
    state: Option<State>
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let win = event_loop.create_window(Window::default_attributes()).unwrap();
        self.state = Some(State::new(win));
    }

    fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            _window_id: winit::window::WindowId,
            event: winit::event::WindowEvent,
        ) {
        
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::Resized(sz) => {
                let state = self.state.as_mut().unwrap();
                state.resize(sz);
            },
            WindowEvent::RedrawRequested => {
                let state = self.state.as_mut().unwrap();

                state.render();

                state.win().request_redraw();
            },
            _ => {}
        }
    }

    fn exiting(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.state = None;
        println!("abc");
    }
}

fn main() -> anyhow::Result<()> {
    let ev = EventLoop::new()?;
    ev.set_control_flow(ControlFlow::Poll);
    ev.run_app(&mut App::default())?;

    Ok(())
}
