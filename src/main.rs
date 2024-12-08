use glium::{
    backend::glutin::SimpleWindowBuilder,
    glutin::surface::WindowSurface,
    winit::{
        application::ApplicationHandler,
        event::WindowEvent,
        event_loop::{ActiveEventLoop, EventLoop},
        window::{Window, WindowId},
    },
    Display, Surface,
};

#[derive(Default)]
struct App {
    window: Option<Window>,
    display: Option<Display<WindowSurface>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (window, display) = SimpleWindowBuilder::new()
            .with_title("CedarMS")
            .with_inner_size(2560, 1600)
            .build(event_loop);

        self.window = Some(window);
        self.display = Some(display);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let display = self.display.as_ref().unwrap();
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 1.0, 1.0);
                target.finish().unwrap();

                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
