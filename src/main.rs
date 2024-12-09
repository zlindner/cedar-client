use glium::{
    backend::glutin::SimpleWindowBuilder,
    glutin::surface::WindowSurface,
    winit::{
        application::ApplicationHandler,
        event::WindowEvent,
        event_loop::{ActiveEventLoop, EventLoop},
        window::{Window, WindowId},
    },
    Display, Program, Surface,
};

#[derive(Default)]
struct App {
    window: Option<Window>,
    display: Option<Display<WindowSurface>>,
    program: Option<Program>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (window, display) = SimpleWindowBuilder::new()
            .with_title("CedarMS")
            .with_inner_size(2560, 1600)
            .build(event_loop);

        let vertex_shader_src = r#"
            #version 410 core

            in vec4 coord;
            in vec4 color;

            out vec2 texpos;
            out vec4 colormod;

            uniform vec2 screensize;
            uniform int yoffset;

            void main(void)
            {
                float x = -1.0 + coord.x * 2.0 / screensize.x;
                float y = 1.0 - (coord.y + float(yoffset)) * 2.0 / screensize.y;
                gl_Position = vec4(x, y, 0.0, 1.0);
                texpos = coord.zw;
                colormod = color;
            }
        "#;

        let fragment_shader_src = r#"
            #version 410 core

		    in vec2 texpos;
            in vec4 colormod;

            uniform sampler2D tex;
            uniform vec2 atlassize;
            uniform int fontregion;

            out vec4 FragColor;

            void main(void)
            {
                if (texpos.y == 0)
                {
                    FragColor = colormod;
                }
                else if (texpos.y <= float(fontregion))
                {
                    FragColor = vec4(1, 1, 1, texture(tex, texpos / atlassize).r) * colormod;
                }
                else
                {
                    FragColor = texture(tex, texpos / atlassize) * colormod;
                }
            }
        "#;

        self.window = Some(window);
        self.display = Some(display);
        self.program = Some(
            Program::from_source(
                self.display.as_ref().unwrap(),
                vertex_shader_src,
                fragment_shader_src,
                None,
            )
            .unwrap(),
        );
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
