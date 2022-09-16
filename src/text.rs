use std::{collections::HashMap, ffi::OsStr};

use anyhow::*;

use crate::texture;


pub struct GlyphRect {
    pub tl: f32,
    pub bl: f32,
    pub br: f32,
    pub tr: f32,
}

impl GlyphRect {
    pub fn new(tl: f32, bl: f32, br: f32, tr: f32) -> Self {
        Self { tl, bl, br, tr }
    }

    pub fn as_arr(&self) -> [f32; 4] {
        [ self.tl, self.bl, self.br, self.tr ]
    }
}

#[derive(Clone)]
pub struct GlyphDesc {
    linear_stride: usize,
    h: i32,
    w: i32,
    pitch: i32, // row stride, add this to go down one row
    bearing_x: i32,
    bearing_y: i32,
    advance: i32, // in 1/64 pixels
}

pub struct LinearTextAtlas {
    sum_pitch: usize,
    max_y_max: usize,
    max_y_min: usize,
    descriptors: Vec<GlyphDesc>,
    bytes: Vec<u8>,
}

impl LinearTextAtlas {    
    fn create(face: &freetype::face::Face) -> Result<Self> {
        const COUNT: usize = 128;

        let mut descriptors = Vec::with_capacity(COUNT);
        let mut bytes = Vec::new();
        
        let mut sum_pitch = 0;
        let (mut max_y_max, mut max_y_min) = (0, 0);

        let mut stride = 0;
        for ch in 0..COUNT {
            face.load_char(ch, freetype::face::LoadFlag::RENDER)?;
            let glyph = face.glyph();
            let bitmap = glyph.bitmap();
            bytes.extend(bitmap.buffer());
            
            let desc = GlyphDesc {
                linear_stride: stride,
                h: bitmap.rows(),
                w: bitmap.width(),
                // TODO: what if pitch is negative
                // NOTE: do not support for now and produce garbage
                pitch: bitmap.pitch(),
                bearing_x: glyph.bitmap_left(),
                bearing_y: glyph.bitmap_top(),
                advance: glyph.advance().x,
            };
            sum_pitch += desc.pitch;
            max_y_max = max_y_max.max(desc.bearing_y);
            max_y_min = max_y_min.max(desc.h - desc.bearing_y);
            stride += (desc.h * desc.pitch) as usize;
            
            descriptors.push(desc);
        }

        Ok(Self {
            sum_pitch: sum_pitch as usize,
            max_y_max: max_y_max as usize,
            max_y_min: max_y_min as usize,
            descriptors,
            bytes,
        })
    }

    pub fn get_glyph_texture(&self, ch: usize) -> (&GlyphDesc, &[u8]) {
        let desc = &self.descriptors[ch];
        let stride = desc.linear_stride;
        let size = (desc.h * desc.pitch) as usize;

        (desc, &self.bytes[stride .. stride+size])
    }
}

pub struct TextAtlas {
    descriptors: Vec<GlyphDesc>,
    rects: Vec<GlyphRect>,
    bytes: Vec<u8>,
}

impl TextAtlas {
    pub fn create(linear_atlas: &LinearTextAtlas) -> Self {
        const COUNT: usize = 128;

        let fit_w = linear_atlas.sum_pitch;
        let fit_h = linear_atlas.max_y_max + linear_atlas.max_y_min;
        let zero = linear_atlas.max_y_max;

        let descriptors = linear_atlas.descriptors.clone();
        let mut rects = Vec::with_capacity(descriptors.len());
        let mut bytes = vec![0; fit_h * fit_w];

        // bytes[zero-bearing_y..zero-bearing_y+h, x0..x1] = linear_atlas.bytes[stride..stride+size].as_2d(h, pitch);

        let mut x_start = 0;
        for ch in 0..COUNT {
            let (desc, texture) = linear_atlas.get_glyph_texture(ch);
            
            let (tl, bl) = (
                zero - desc.bearing_y as usize,
                zero - desc.bearing_y as usize + desc.h as usize - 1,
            );
            let (br, tr) = (
                tl + desc.pitch as usize - 1,
                bl + desc.pitch as usize - 1,
            );

            for i in 0..desc.h as usize {
                // bytes[...] = texture[pitch*i .. pitch*(i+1)];
                // (
                //     zero - desc.bearing_y as usize + i .. zero - desc.bearing_y as usize + (i+1),
                //     x_start .. x_start + desc.pitch
                // );
                let offset_factor_2d = (tl + i) * fit_w;
                let offset = offset_factor_2d + x_start;
                bytes[offset .. offset + desc.pitch as usize].as_mut()
                    .clone_from_slice(&texture[desc.pitch as usize * i .. desc.pitch as usize * (i+1)]);
            }

            rects.push(GlyphRect {
                tl: tl as f32,
                bl: bl as f32,
                br: br as f32,
                tr: tr as f32,
            });

            x_start += desc.pitch as usize;
        }

        Self {
            descriptors,
            rects,
            bytes,
        }
    }
}

pub struct FontContainer {
    face: freetype::face::Face,
    atlas: LinearTextAtlas,
}

impl FontContainer {
    pub fn new(
        library: freetype::Library,
        font_path: impl AsRef<OsStr>,
        face_index: isize,
    ) -> Result<Self> {
        let face = library.new_face(font_path, face_index)?;
        let atlas = LinearTextAtlas::create(&face)?;
        Ok(Self {
            face,
            atlas,
        })
    }

    pub fn get_glyph_texture(&self, ch: usize) -> (&GlyphDesc, &[u8]) {
        self.atlas.get_glyph_texture(ch)
    }
}

pub struct TextMap {
    map: HashMap<String, FontContainer>,
}