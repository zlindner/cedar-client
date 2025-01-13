use std::{collections::HashMap, fs::File, io::Read, path::Path};

use ab_glyph::{point, FontVec, Glyph, PxScale, ScaleFont};
use image::DynamicImage;

use crate::component::Colour;

/// The set of supported characters.
const CHARACTERS: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890!@#$%^&*()_-+=<,>.?/:;'{[}]|\\\"";

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct FontDescriptor {
    name: String,
    size: u8,
    colour: Colour,
}

impl FontDescriptor {
    pub fn new(name: &str, size: u8, colour: Colour) -> Self {
        Self {
            name: name.to_string(),
            size,
            colour,
        }
    }
}

impl Default for FontDescriptor {
    fn default() -> Self {
        Self {
            name: "Arial".to_string(),
            size: 13,
            colour: Colour::white(),
        }
    }
}

pub struct Font {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub min_y: f32,
    pub characters: HashMap<char, FontCharacter>,
}

impl Font {
    pub fn load(descriptor: FontDescriptor) -> Self {
        let path = format!("assets/fonts/{}.ttf", descriptor.name);
        let mut file = File::open(Path::new(&path)).expect("font should exist in assets/fonts");
        let mut font_bytes = Vec::new();
        file.read_to_end(&mut font_bytes).unwrap();

        let font = FontVec::try_from_vec(font_bytes).unwrap();
        let font = ab_glyph::Font::as_scaled(&font, PxScale::from(descriptor.size as f32));

        let mut glyphs: Vec<Glyph> = Vec::new();
        let mut caret = point(0.0, font.ascent());

        for (i, char) in CHARACTERS.chars().enumerate() {
            let mut glyph = font.scaled_glyph(char);

            if i != 0 {
                let prev = glyphs.get(i - 1).unwrap();
                caret.x += font.kern(prev.id, glyph.id);
            }

            glyph.position = caret;
            caret.x += font.h_advance(glyph.id);

            glyphs.push(glyph);
        }

        let glyphs_width = {
            let min_x = glyphs.first().unwrap().position.x;
            let last_glyph = glyphs.last().unwrap();
            let max_x = last_glyph.position.x + font.h_advance(last_glyph.id);
            (max_x - min_x).ceil() as u32
        };
        let glyphs_height = font.height().ceil() as u32;
        let mut min_y = 99999.;

        // TODO: should be able to (eventually) get rid of image crate dependency and create our
        // own simple image buffer.
        let mut image = DynamicImage::new_rgba8(glyphs_width + 40, glyphs_height + 40).to_rgba8();
        let mut characters = HashMap::new();

        for (pos, glyph) in glyphs.drain(0..glyphs.len()).enumerate() {
            if let Some(outlined) = font.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();
                outlined.draw(|x, y, v| {
                    let px = image.get_pixel_mut(x + bounds.min.x as u32, y + bounds.min.y as u32);

                    *px = image::Rgba([
                        descriptor.colour.red,
                        descriptor.colour.green,
                        descriptor.colour.blue,
                        px.0[3].saturating_add((v * descriptor.colour.alpha as f32) as u8),
                    ]);
                });

                if min_y > bounds.min.y {
                    min_y = bounds.min.y;
                }

                characters.insert(
                    CHARACTERS.chars().nth(pos).unwrap(),
                    FontCharacter::new((bounds.min.x, bounds.max.x), (bounds.min.y, bounds.max.y)),
                );
            }
        }

        Self {
            data: image.to_vec(),
            width: glyphs_width + 40,
            height: glyphs_height + 40,
            min_y,
            characters,
        }
    }

    pub fn compute_vertical_offset(&self, current_pos_y: f32) -> f32 {
        if current_pos_y > self.min_y {
            return current_pos_y - self.min_y;
        }

        0.0
    }
}

pub struct FontCharacter {
    pub x: (f32, f32),
    pub y: (f32, f32),
    pub width: f32,
    pub height: f32,
}

impl FontCharacter {
    pub fn new(x: (f32, f32), y: (f32, f32)) -> Self {
        Self {
            x,
            y,
            width: x.1 - x.0,
            height: y.1 - y.0,
        }
    }
}
