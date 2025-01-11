use std::{collections::HashMap, fs::File, io::Read, path::Path, sync::LazyLock};

use ab_glyph::{point, Font, FontVec, Glyph, PxScale, ScaleFont};
use image::DynamicImage;
use nx_pkg4::{Node, NxFile};

use crate::graphics::Texture;

static NX_FILES: LazyLock<HashMap<String, NxFile>> = LazyLock::new(|| {
    let mut nx_files = HashMap::new();
    let paths = std::fs::read_dir("assets/nx").expect("nx folder should exist");

    for path in paths {
        let file_name = path.unwrap().file_name().into_string().unwrap();
        let nx_path = format!("assets/nx/{}", file_name);
        nx_files.insert(file_name, NxFile::open(Path::new(&nx_path)).unwrap());
    }

    nx_files
});

static FONTS: LazyLock<HashMap<String, FontData>> = LazyLock::new(|| {
    let mut fonts = HashMap::new();

    // TODO fonts should be keyed by a FontKey, containing font name, size, colour.
    fonts.insert("default".to_string(), FontData::load(24.0, (0, 0, 0)));
    fonts
});

/// The set of supported characters.
const CHARACTERS: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890!@#$%^&*()_-+=<,>.?/:;'{[}]|\\\"";

pub struct AssetManager;

impl AssetManager {
    pub fn get_texture(path: &str) -> Option<Texture> {
        log::info!("Getting texture for {}", path);
        let (file_name, path) = path.split_at(path.find("/").unwrap());

        let file = match NX_FILES.get(file_name) {
            Some(file) => file,
            None => {
                log::warn!("{} isn't open", file_name);
                return None;
            }
        };

        let root = file.root();

        // Remove the leading slash from path.
        let node = match root.get(&path[1..path.len()]) {
            Some(node) => node,
            None => {
                log::error!("Texture not found {}", path);
                return None;
            }
        };

        match Texture::load(path, node) {
            Ok(texture) => texture,
            Err(e) => {
                log::error!("Error getting texture {}: {}", path, e);
                return None;
            }
        }
    }

    pub fn get_texture_rgba(path: &str) -> Option<Texture> {
        let mut texture = match Self::get_texture(path) {
            Some(texture) => texture,
            None => return None,
        };

        for pixel in texture.data.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }

        Some(texture)
    }

    pub fn get_font(key: &str) -> Option<&FontData> {
        FONTS.get(key)
    }
}

pub struct FontData {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub min_y: f32,
    pub characters: HashMap<char, FontCharacter>,
}

impl FontData {
    pub fn load(font_size: f32, font_colour: (u8, u8, u8)) -> Self {
        let mut file = File::open(Path::new("assets/fonts/Arial.ttf")).unwrap();
        let mut font_bytes = Vec::new();
        file.read_to_end(&mut font_bytes).unwrap();

        let font = FontVec::try_from_vec(font_bytes).unwrap();
        let font = font.as_scaled(PxScale::from(font_size));

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
        let mut image = DynamicImage::new_rgba8(glyphs_width + 40, glyphs_height + 40).to_rgba8();
        let mut characters = HashMap::new();

        for (pos, glyph) in glyphs.drain(0..glyphs.len()).enumerate() {
            if let Some(outlined) = font.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();
                outlined.draw(|x, y, v| {
                    let px = image.get_pixel_mut(x + bounds.min.x as u32, y + bounds.min.y as u32);

                    *px = image::Rgba([
                        font_colour.0,
                        font_colour.1,
                        font_colour.2,
                        px.0[3].saturating_add((v * 255.0) as u8),
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
