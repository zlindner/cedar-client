use std::ops::Range;

use wgpu::include_wgsl;

use super::Vertex;

const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

#[derive(Clone)]
pub struct Sprite {
    pub texture_path: String,
    vertices: Option<[Vertex; 4]>,
}

impl Sprite {
    pub fn new(texture_path: String) -> Self {
        Self {
            texture_path,
            vertices: None,
        }
    }

    fn compute_vertices(&self) -> [Vertex; 4] {
        [
            Vertex {
                position: [-1.0, -1.0, 0.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [1.0, -1.0, 0.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [1.0, 1.0, 0.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [-1.0, 1.0, 0.0],
                tex_coords: [0.0, 0.0],
            },
        ]
    }
}

pub trait Renderable {
    fn create_render_pipeline(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        config: &wgpu::SurfaceConfiguration,
    ) -> wgpu::RenderPipeline;

    fn get_vertex_buffer(&mut self) -> &[u8];

    fn get_index_buffer(&self) -> &[u8];

    fn index_buffer_range(&self) -> Range<u32>;
}

impl Renderable for Sprite {
    fn create_render_pipeline(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        config: &wgpu::SurfaceConfiguration,
    ) -> wgpu::RenderPipeline {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("bitmap render pipeline layout"),
                bind_group_layouts: &[bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader = device.create_shader_module(include_wgsl!("shaders/bitmap.wgsl"));

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

    fn get_vertex_buffer(&mut self) -> &[u8] {
        if self.vertices.is_none() {
            self.vertices = Some(self.compute_vertices());
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
