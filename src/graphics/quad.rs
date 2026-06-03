use std::ops::Range;

use crate::{graphics::Vertex, resource::FontCharacter};

const INDICES: &[u16] = &[0, 1, 3, 3, 1, 2];

#[derive(Clone)]
pub struct TexturedQuad {
    pub vertex_buffer: Vec<u8>,
    pub index_buffer: Vec<u8>,
    pub index_buffer_range: Range<u32>,
}

impl TexturedQuad {
    pub fn full_image(width: u32, height: u32) -> Self {
        let width = width as f32;
        let height = height as f32;

        Self::new(
            [0.0, 0.0],
            [width, height],
            [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]],
        )
    }

    pub fn glyph(source: SourceRect, image_width: u32, image_height: u32) -> Self {
        let image_width = image_width as f32;
        let image_height = image_height as f32;

        Self::new(
            [0.0, 0.0],
            [source.width, source.height],
            [
                [source.x.0 / image_width, source.y.0 / image_height],
                [source.x.0 / image_width, source.y.1 / image_height],
                [source.x.1 / image_width, source.y.1 / image_height],
                [source.x.1 / image_width, source.y.0 / image_height],
            ],
        )
    }

    fn new(origin: [f32; 2], size: [f32; 2], tex_coords: [[f32; 2]; 4]) -> Self {
        let [x, y] = origin;
        let [width, height] = size;
        let vertices = [
            Vertex {
                position: [x, y, 0.0],
                tex_coords: tex_coords[0],
            },
            Vertex {
                position: [x, y + height, 0.0],
                tex_coords: tex_coords[1],
            },
            Vertex {
                position: [x + width, y + height, 0.0],
                tex_coords: tex_coords[2],
            },
            Vertex {
                position: [x + width, y, 0.0],
                tex_coords: tex_coords[3],
            },
        ];

        Self {
            vertex_buffer: bytemuck::cast_slice(&vertices).to_vec(),
            index_buffer: bytemuck::cast_slice(INDICES).to_vec(),
            index_buffer_range: 0..INDICES.len() as u32,
        }
    }
}

pub struct SourceRect {
    pub x: (f32, f32),
    pub y: (f32, f32),
    pub width: f32,
    pub height: f32,
}

impl From<&FontCharacter> for SourceRect {
    fn from(character: &FontCharacter) -> Self {
        Self {
            x: character.x,
            y: character.y,
            width: character.width,
            height: character.height,
        }
    }
}
