use std::{fmt, ops::Range};

use nx_pkg4::{Node, NxError, NxNode};

use crate::graphics::Vertex;

use super::Renderable;

const INDICES: &[u16] = &[0, 1, 3, 3, 1, 2];

#[derive(Clone)]
pub struct Texture {
    pub path: String,

    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,

    /// The value of the `origin` child node.
    pub origin: Option<(i32, i32)>,

    /// The value of the `z` child node.
    layer: Option<i64>,

    pub vertex_buffer: Vec<u8>,
    pub index_buffer: Vec<u8>,
    pub index_buffer_range: Range<u32>,
}

// TODO: this can probably be something provided by nx-pkg4
impl Texture {
    pub fn load(path: &str, node: NxNode) -> Result<Option<Self>, NxError> {
        let origin = match node.get("origin") {
            Some(child) => child.vector()?,
            None => None,
        };

        let layer = match node.get("z") {
            Some(child) => child.integer()?,
            None => None,
        };

        let bitmap = match node.bitmap()? {
            Some(bitmap) => bitmap,
            None => {
                log::warn!("{} isn't a bitmap", path);
                return Ok(None);
            }
        };

        let width = bitmap.width.into();
        let height = bitmap.height.into();
        let vertex_buffer = Self::get_vertex_buffer(width, height);

        Ok(Some(Self {
            path: path.to_string(),
            width: bitmap.width.into(),
            height: bitmap.height.into(),
            data: bitmap.data,
            origin,
            layer,
            vertex_buffer,
            index_buffer: bytemuck::cast_slice(INDICES).to_vec(),
            index_buffer_range: 0..INDICES.len() as u32,
        }))
    }

    fn get_vertex_buffer(width: u32, height: u32) -> Vec<u8> {
        let width = width as f32;
        let height = height as f32;

        let vertices = [
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
        ];

        bytemuck::cast_slice(&vertices).to_vec()
    }
}

/// Manually implementing Debug for Texture, replacing data with an empty slice since it can
/// contain hundreds of elements and isn't useful to log.
impl fmt::Debug for Texture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        #[derive(Debug)]
        #[allow(unused)]
        struct Texture<'a> {
            path: &'a str,
            width: &'a u32,
            height: &'a u32,
            data: [u8; 0],
            origin: &'a Option<(i32, i32)>,
            layer: &'a Option<i64>,
            vertex_buffer: [u8; 0],
            index_buffer: [u8; 0],
            index_buffer_range: &'a Range<u32>,
        }

        let Self {
            path,
            width,
            height,
            data: _,
            origin,
            layer,
            vertex_buffer: _,
            index_buffer: _,
            index_buffer_range,
        } = self;

        fmt::Debug::fmt(
            &Texture {
                path,
                width,
                height,
                data: [],
                origin,
                layer,
                vertex_buffer: [],
                index_buffer: [],
                index_buffer_range,
            },
            f,
        )
    }
}

impl Renderable for Texture {
    fn create_render_pipeline(
        device: &wgpu::Device,
        transform_bind_group_layout: &wgpu::BindGroupLayout,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        config: &wgpu::SurfaceConfiguration,
    ) -> wgpu::RenderPipeline {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("texture render pipeline layout"),
                bind_group_layouts: &[transform_bind_group_layout, texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/texture.wgsl"));

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("texture render pipeline"),
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
}
