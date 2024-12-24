use ultraviolet::Mat4;
use ultraviolet::Similarity3;
use ultraviolet::Vec3;
use ultraviolet::Vec4;

use crate::component::Camera;
use crate::component::Texture;
use crate::component::Transform;

pub use self::renderer::RenderItem;
pub use self::renderer::Renderer;
pub use self::renderer::RendererEvent;
pub use self::renderer_manager::RendererManager;
pub use self::sprite::Sprite;

mod renderer;
mod renderer_manager;
mod sprite;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniform {
    pub model_transform: [[f32; 4]; 4],
    pub camera_view: [[f32; 4]; 4],
}

impl Uniform {
    pub fn compute(transform: &Transform, camera: &Camera, texture: &Texture) -> Self {
        let mut model_transform = Similarity3::identity();
        model_transform.prepend_scaling(transform.scale);

        if let Some((x, y)) = texture.origin {
            model_transform.append_translation(Vec3 {
                x: -x as f32,
                y: -y as f32,
                z: 0.0,
            });
        }

        model_transform.append_translation(Vec3 {
            x: transform.x * transform.scale,
            y: transform.y * transform.scale,
            z: transform.z,
        });

        // TODO rotation?

        let camera_view = ultraviolet::projection::lh_ydown::orthographic_wgpu_dx(
            camera.left,
            camera.right,
            camera.bottom,
            camera.top,
            camera.near,
            camera.far,
        );

        Self {
            model_transform: create_matrix4(&model_transform.into_homogeneous_matrix()),
            camera_view: create_matrix4(&camera_view),
        }
    }
}

fn create_matrix4(t: &Mat4) -> [[f32; 4]; 4] {
    [
        create_matrix(&t.cols[0]),
        create_matrix(&t.cols[1]),
        create_matrix(&t.cols[2]),
        create_matrix(&t.cols[3]),
    ]
}

fn create_matrix(t: &Vec4) -> [f32; 4] {
    [t.x, t.y, t.z, t.w]
}
