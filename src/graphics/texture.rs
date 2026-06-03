use std::{fmt, sync::Arc};

use crate::{
    graphics::{ImageAsset, SourceRect, TexturedQuad, Vertex},
    resource::FontCharacter,
};

use super::Renderable;

#[derive(Clone)]
pub struct Texture {
    pub image: Arc<ImageAsset>,
    pub quad: TexturedQuad,
}

impl Texture {
    pub fn from_image(image: Arc<ImageAsset>) -> Self {
        let quad = TexturedQuad::full_image(image.width, image.height);
        Self { image, quad }
    }

    pub fn from_image_asset(image: ImageAsset) -> Self {
        Self::from_image(Arc::new(image))
    }

    pub fn font(character: &FontCharacter, image: Arc<ImageAsset>) -> Self {
        let quad = TexturedQuad::glyph(SourceRect::from(character), image.width, image.height);

        Self { image, quad }
    }
}

/// Manually implementing Debug for Texture, replacing pixel and buffer data with empty slices
/// since they can contain hundreds of elements and aren't useful to log.
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
        }

        fmt::Debug::fmt(
            &Texture {
                path: &self.image.path,
                width: &self.image.width,
                height: &self.image.height,
                data: [],
                origin: &self.image.origin,
                layer: &self.image.layer,
                vertex_buffer: [],
                index_buffer: [],
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
