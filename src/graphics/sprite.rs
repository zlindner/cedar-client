use std::ops::Range;

use nx_pkg4::NxBitmap;
use wgpu::include_wgsl;

use super::Vertex;

const INDICES: &[u16] = &[0, 1, 3, 3, 1, 2];

#[derive(Clone)]
pub struct Sprite {
    pub bitmap_path: String,
    vertices: Option<[Vertex; 4]>,
}

impl Sprite {
    pub fn new(bitmap_path: &str) -> Self {
        Self {
            bitmap_path: bitmap_path.to_string(),
            vertices: None,
        }
    }

    fn compute_vertices(&self, bitmap: &NxBitmap) -> [Vertex; 4] {
        let width = bitmap.width.into();
        let height = bitmap.height.into();

        [
            Vertex {
                position: [0.0, 0.0, 0.0],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: [0.0, height, 0.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [width, height, 0.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [width, 0.0, 0.0],
                tex_coords: [1.0, 0.0],
            },
        ]
    }
}

pub trait Renderable {
    fn create_render_pipeline(
        device: &wgpu::Device,
        transform_bind_group_layout: &wgpu::BindGroupLayout,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        config: &wgpu::SurfaceConfiguration,
    ) -> wgpu::RenderPipeline;

    fn get_vertex_buffer(&mut self, bitmap: &NxBitmap) -> &[u8];

    fn get_index_buffer(&self) -> &[u8];

    fn index_buffer_range(&self) -> Range<u32>;
}

impl Renderable for Sprite {
    fn create_render_pipeline(
        device: &wgpu::Device,
        transform_bind_group_layout: &wgpu::BindGroupLayout,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        config: &wgpu::SurfaceConfiguration,
    ) -> wgpu::RenderPipeline {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("bitmap render pipeline layout"),
                bind_group_layouts: &[transform_bind_group_layout, texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader = device.create_shader_module(include_wgsl!("shaders/sprite.wgsl"));

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("bitmap render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        })
    }

    fn get_vertex_buffer(&mut self, bitmap: &NxBitmap) -> &[u8] {
        if self.vertices.is_none() {
            self.vertices = Some(self.compute_vertices(bitmap));
        }

        bytemuck::cast_slice(self.vertices.as_ref().unwrap())
    }

    fn get_index_buffer(&self) -> &[u8] {
        bytemuck::cast_slice(INDICES)
    }

    fn index_buffer_range(&self) -> Range<u32> {
        0..INDICES.len() as u32
    }
}
