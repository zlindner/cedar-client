use std::path::Path;

use glium::{
    backend::glutin::SimpleWindowBuilder,
    glutin::surface::WindowSurface,
    implement_vertex,
    index::{NoIndices, PrimitiveType},
    texture::{RawImage2d, Texture2d},
    uniform,
    winit::{
        application::ApplicationHandler,
        event::WindowEvent,
        event_loop::{ActiveEventLoop, EventLoop},
        window::{Window, WindowId},
    },
    Display, Program, Surface, VertexBuffer,
};
use nx_pkg4::file::NxFile;

#[derive(Default)]
struct App {
    window: Option<Window>,
    display: Option<Display<WindowSurface>>,
    program: Option<Program>,
    ui_nx: Option<NxFile>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (window, display) = SimpleWindowBuilder::new()
            .with_title("CedarMS")
            .with_inner_size(2560, 1600)
            .build(event_loop);

        let vertex_shader_src = r#"
            #version 410 core

            in vec2 position;
            in vec2 tex_coords;
            in vec4 colour;

            out vec2 v_tex_coords;
            out vec4 v_colour;

            uniform vec2 screen_size;
            uniform int y_offset;

            void main(void)
            {
                float x = -1.0 + position.x * 2.0 / screen_size.x;
			    float y = 1.0 - (position.y + y_offset) * 2.0 / screen_size.y;

                gl_Position = vec4(position.x, position.y, 0.0, 1.0);
                
                v_tex_coords = tex_coords;
                v_colour = colour;
            }
        "#;

        let fragment_shader_src = r#"
            #version 410 core

		    in vec2 v_tex_coords;
            in vec4 v_colour;
            
            out vec4 colour;

            uniform sampler2D tex;

            void main(void)
            {
                colour = texture(tex, v_tex_coords);
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

        self.ui_nx = Some(NxFile::open(Path::new("nx/UI.nx")).unwrap());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let display = self.display.as_ref().unwrap();

                let mut target = display.draw();
                target.clear_color(1.0, 1.0, 1.0, 1.0);

                let root = self.ui_nx.as_ref().unwrap().root();
                let basic = root.get("Basic.img").unwrap();
                let cursor = basic.get("Cursor").unwrap();
                let cursor_0 = cursor.get("0").unwrap();
                let cursor_0_0 = cursor_0.get("0").unwrap();

                let bitmap = cursor_0_0.bitmap().unwrap().unwrap();

                let image = RawImage2d::from_raw_rgba_reversed(
                    &bitmap.data,
                    (bitmap.width.into(), bitmap.height.into()),
                );

                let texture = Texture2d::new(display, image).unwrap();

                #[derive(Copy, Clone)]
                struct Vertex {
                    position: [f32; 2],
                    tex_coords: [f32; 2],
                    colour: [f32; 4],
                }

                implement_vertex!(Vertex, position, tex_coords, colour);

                let shape = vec![
                    Vertex {
                        position: [-0.5, -0.5],
                        tex_coords: [0.0, 0.0],
                        colour: [0.0, 0.0, 0.0, 1.0],
                    },
                    Vertex {
                        position: [0.5, -0.5],
                        tex_coords: [1.0, 0.0],
                        colour: [0.0, 0.0, 0.0, 1.0],
                    },
                    Vertex {
                        position: [0.5, 0.5],
                        tex_coords: [1.0, 1.0],
                        colour: [0.0, 0.0, 0.0, 1.0],
                    },
                    Vertex {
                        position: [0.5, 0.5],
                        tex_coords: [1.0, 1.0],
                        colour: [0.0, 0.0, 0.0, 1.0],
                    },
                    Vertex {
                        position: [-0.5, 0.5],
                        tex_coords: [0.0, 1.0],
                        colour: [0.0, 0.0, 0.0, 1.0],
                    },
                    Vertex {
                        position: [-0.5, -0.5],
                        tex_coords: [0.0, 0.0],
                        colour: [0.0, 0.0, 0.0, 1.0],
                    },
                ];
                let indices = NoIndices(PrimitiveType::TrianglesList);
                let vertex_buffer = VertexBuffer::new(display, &shape).unwrap();

                let uniforms = uniform! {
                    screen_size: [800.0_f32, 600.0_f32],
                    y_offset: 0,
                    tex: &texture,
                };

                target
                    .draw(
                        &vertex_buffer,
                        &indices,
                        self.program.as_ref().unwrap(),
                        &uniforms,
                        &Default::default(),
                    )
                    .unwrap();

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
