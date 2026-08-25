use crate::engine::renderer::texture::Texture;
use anyhow::{Context, Result};
use glam::Vec2;
use std::collections::HashMap;

/// Good Neighbors font, hand-arranged (in Aseprite) into a uniform
/// 10x9 grid — no external metadata file needed, since every glyph
/// occupies an identical cell and position is pure arithmetic from
/// character order, not derived from a packed/variable-width atlas.
pub const GLYPH_SIZE: Vec2 = Vec2::new(9.0, 15.0);
/// Distance from one cell's origin to the next — glyph size plus the
/// 1px gap on all sides. Distinct from GLYPH_SIZE: the glyph itself
/// doesn't fill the full pitch, narrow characters (e.g. '1') just
/// leave their cell's remainder blank, same as any monospace font.
pub const GLYPH_PITCH: Vec2 = Vec2::new(10.0, 16.0);
pub const ATLAS_SIZE: Vec2 = Vec2::new(100.0, 144.0);
/// Multiplies GLYPH_SIZE/GLYPH_PITCH for on-screen rendering — the
/// atlas itself stays authored at native 9x15 pixels (nothing here
/// changes), this just scales how large a glyph quad ends up drawn.
pub const DIALOGUE_TEXT_SCALE: f32 = 2.0;
pub const DEBUG_TEXT_SCALE: f32 = 1.25;
pub const DEBUG_TEXT_PADDING: f32 = 10.0;

/// Maps a character to its (column, row) cell in the atlas grid.
/// Explicit table, not computed from character code, since the grid's
/// ordering doesn't follow any contiguous range (digits, then
/// uppercase, then a mixed symbol set per row). Returns None for any
/// character not present in this font.
pub fn glyph_cell(c: char) -> Option<(u32, u32)> {
    Some(match c {
        '0' => (0, 0),
        '1' => (1, 0),
        '2' => (2, 0),
        '3' => (3, 0),
        '4' => (4, 0),
        '5' => (5, 0),
        '6' => (6, 0),
        '7' => (7, 0),
        '8' => (8, 0),
        '9' => (9, 0),

        'A' => (0, 1),
        'B' => (1, 1),
        'C' => (2, 1),
        'D' => (3, 1),
        'E' => (4, 1),
        'F' => (5, 1),
        'G' => (6, 1),
        'H' => (7, 1),
        'I' => (8, 1),
        'J' => (9, 1),

        'K' => (0, 2),
        'L' => (1, 2),
        'M' => (2, 2),
        'N' => (3, 2),
        'O' => (4, 2),
        'P' => (5, 2),
        'Q' => (6, 2),
        'R' => (7, 2),
        'S' => (8, 2),
        'T' => (9, 2),

        'U' => (0, 3),
        'V' => (1, 3),
        'W' => (2, 3),
        'X' => (3, 3),
        'Y' => (4, 3),
        'Z' => (5, 3),
        '?' => (6, 3),
        '@' => (7, 3),
        '=' => (8, 3),
        '-' => (9, 3),

        'a' => (0, 4),
        'b' => (1, 4),
        'c' => (2, 4),
        'd' => (3, 4),
        'e' => (4, 4),
        'f' => (5, 4),
        'g' => (6, 4),
        'h' => (7, 4),
        'i' => (8, 4),
        'j' => (9, 4),

        'k' => (0, 5),
        'l' => (1, 5),
        'm' => (2, 5),
        'n' => (3, 5),
        'o' => (4, 5),
        'p' => (5, 5),
        'q' => (6, 5),
        'r' => (7, 5),
        's' => (8, 5),
        't' => (9, 5),

        'u' => (0, 6),
        'v' => (1, 6),
        'w' => (2, 6),
        'x' => (3, 6),
        'y' => (4, 6),
        'z' => (5, 6),
        '+' => (6, 6),
        ',' => (7, 6),
        '.' => (8, 6),
        '_' => (9, 6),

        '!' => (0, 7),
        '"' => (1, 7),
        '#' => (2, 7),
        '$' => (3, 7),
        '%' => (4, 7),
        '&' => (5, 7),
        '\'' => (6, 7),
        '(' => (7, 7),
        ')' => (8, 7),
        '*' => (9, 7),

        '\\' => (0, 8),
        '/' => (1, 8),
        ':' => (2, 8),
        ';' => (3, 8),
        '<' => (4, 8),
        '>' => (5, 8),
        '©' => (6, 8),
        '▼' => (7, 8),

        _ => return None,
    })
}

/// Top-left pixel of a glyph's cell within the atlas texture.
pub fn glyph_atlas_position(column: u32, row: u32) -> Vec2 {
    Vec2::new(column as f32 * GLYPH_PITCH.x, row as f32 * GLYPH_PITCH.y)
}

/// UV-space (0.0..1.0) top-left and bottom-right corners for a glyph's
/// cell, ready to feed into the existing textured-quad vertex layout
/// (same tex_coords shape Entity/Background sprites already use, just
/// a sub-rectangle of the atlas instead of the full 0..1 range).
pub fn glyph_uv(column: u32, row: u32) -> (Vec2, Vec2) {
    let pixel_pos = glyph_atlas_position(column, row);
    let uv_min = pixel_pos / ATLAS_SIZE;
    let uv_max = (pixel_pos + GLYPH_SIZE) / ATLAS_SIZE;
    (uv_min, uv_max)
}

/// One glyph, ready to hand to the renderer: which atlas cell to
/// sample, and where on screen (world-space, same units as
/// Entity::position) its cell's top-left corner lands.
pub struct PositionedGlyph {
    pub cell: (u32, u32),
    pub position: Vec2,
    pub color: [f32; 4],
    pub scale: f32,
}

pub struct PositionedTTFGlyph {
    pub uv_min: Vec2,
    pub uv_max: Vec2,
    pub position: Vec2, // top-left of the glyph's quad, unlike center-based positioning used everywhere else
    pub size: Vec2,
    pub color: [f32; 4],
}

pub struct TTFGlyph {
    pub uv_min: Vec2,
    pub uv_max: Vec2,
    pub advance: f32, // how far the cursor advances after this glyph
    pub offset: Vec2, // glyph's bitmap top-left offset from the cursor position
    pub size: Vec2,   // glyph's bitmap size
}

pub struct TextBounds {
    pub min: Vec2,
    pub max: Vec2,
}

impl TextBounds {
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }
}

pub fn layout_ttf_text(
    text: &str,
    font: &HashMap<char, TTFGlyph>,
    origin: Vec2,
    scale: f32,
    color: [f32; 4],
) -> (Vec<PositionedTTFGlyph>, TextBounds) {
    let mut glyphs = Vec::new();
    let mut cursor = origin;
    let mut min = origin;
    let mut max = origin;

    for c in text.chars() {
        if let Some(g) = font.get(&c) {
            let position = cursor + g.offset * scale;
            let size = g.size * scale;
            let bottom_right = position + size;

            min = min.min(position);
            max = max.max(bottom_right);

            glyphs.push(PositionedTTFGlyph {
                uv_min: g.uv_min,
                uv_max: g.uv_max,
                position,
                size,
                color,
            });
            cursor.x += g.advance * scale;
        } else {
            cursor.x += scale * 8.0; // rough fallback for missing/space glyphs
        }
    }

    // Width still needs to reflect the final cursor position, not just
    // the last real glyph's box - trailing fallback/space characters
    // advance width without producing a glyph to compare against.
    max.x = max.x.max(cursor.x);

    (glyphs, TextBounds { min, max })
}

pub fn build_ttf_atlas(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    font_path: &str,
    px_size: f32,
) -> Result<(Texture, HashMap<char, TTFGlyph>)> {
    let font_bytes =
        std::fs::read(font_path).with_context(|| format!("failed to read ttf: {font_path}"))?;
    let font = fontdue::Font::from_bytes(font_bytes, fontdue::FontSettings::default())
        .expect("failed to parse ttf");

    let chars: Vec<char> = (32u8..=126u8).map(|b| b as char).collect(); // printable ASCII

    // Rasterize every glyph up front, so we know each one's real size
    // before deciding atlas layout.
    let rasterized: Vec<(char, fontdue::Metrics, Vec<u8>)> = chars
        .iter()
        .map(|&c| {
            let (metrics, bitmap) = font.rasterize(c, px_size);
            (c, metrics, bitmap)
        })
        .collect();

    const ATLAS_WIDTH: u32 = 256; // fixed width, rows grow downward as needed
    const PADDING: u32 = 1; // gap between glyphs, avoids sampling bleed at small scales

    let mut glyphs = HashMap::new();

    let mut cursor_x: u32 = 0;
    let mut cursor_y: u32 = 0;
    let mut row_height: u32 = 0;

    // First pass: compute final atlas_height by simulating the packing,
    // so the pixel buffer can be allocated once at the right size
    // rather than resized/copied row by row.
    for (_, metrics, _) in &rasterized {
        let w = metrics.width as u32;
        let h = metrics.height as u32;
        if cursor_x + w + PADDING > ATLAS_WIDTH {
            cursor_x = 0;
            cursor_y += row_height + PADDING;
            row_height = 0;
        }
        cursor_x += w + PADDING;
        row_height = row_height.max(h);
    }
    let atlas_height = cursor_y + row_height + PADDING;
    let mut atlas_rgba = vec![0u8; (ATLAS_WIDTH * atlas_height * 4) as usize];

    // Second pass: actually place each glyph's pixels and record its UV rect.
    cursor_x = 0;
    cursor_y = 0;
    row_height = 0;
    for (c, metrics, bitmap) in &rasterized {
        let w = metrics.width as u32;
        let h = metrics.height as u32;
        if cursor_x + w + PADDING > ATLAS_WIDTH {
            cursor_x = 0;
            cursor_y += row_height + PADDING;
            row_height = 0;
        }

        for y in 0..h {
            for x in 0..w {
                let coverage = bitmap[(y * w + x) as usize];
                let atlas_px = ((cursor_y + y) * ATLAS_WIDTH + (cursor_x + x)) as usize * 4;
                atlas_rgba[atlas_px..atlas_px + 4].copy_from_slice(&[255, 255, 255, coverage]);
            }
        }

        let uv_min = Vec2::new(cursor_x as f32, cursor_y as f32)
            / Vec2::new(ATLAS_WIDTH as f32, atlas_height as f32);
        let uv_max = Vec2::new((cursor_x + w) as f32, (cursor_y + h) as f32)
            / Vec2::new(ATLAS_WIDTH as f32, atlas_height as f32);

        glyphs.insert(
            *c,
            TTFGlyph {
                uv_min,
                uv_max,
                advance: metrics.advance_width,
                offset: Vec2::new(metrics.xmin as f32, -metrics.ymin as f32 - h as f32), // see note below
                size: Vec2::new(w as f32, h as f32),
            },
        );

        cursor_x += w + PADDING;
        row_height = row_height.max(h);
    }

    let image_buffer = image::RgbaImage::from_raw(ATLAS_WIDTH, atlas_height, atlas_rgba)
        .expect("atlas buffer size mismatch");
    let dynamic_image = image::DynamicImage::ImageRgba8(image_buffer);
    let texture = Texture::from_image(device, queue, &dynamic_image, Some("ttf atlas"))?;

    Ok((texture, glyphs))
}

pub fn combined_glyph_info(glyphs: &[PositionedGlyph], padding: f32) -> (Vec2, Vec2) {
    if glyphs.is_empty() {
        return (Vec2::ZERO, Vec2::ZERO);
    }

    let scaled_size = GLYPH_SIZE * glyphs.first().map(|g| g.scale).unwrap_or(1.0);

    // Center Position of all the glyphs combined
    let min_x = glyphs
        .iter()
        .map(|g| g.position.x)
        .fold(f32::INFINITY, f32::min);
    let min_y = glyphs
        .iter()
        .map(|g| g.position.y)
        .fold(f32::INFINITY, f32::min);
    let max_x = glyphs
        .iter()
        .map(|g| g.position.x + scaled_size.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let max_y = glyphs
        .iter()
        .map(|g| g.position.y + scaled_size.y)
        .fold(f32::NEG_INFINITY, f32::max);

    let combined_center = Vec2::new(min_x + (max_x - min_x) / 2.0, min_y + (max_y - min_y) / 2.0);
    let combined_size = Vec2::new(max_x - min_x + padding * 2.0, max_y - min_y + padding * 2.0);

    (combined_center, combined_size)
}

pub fn centered_text_origin(text: &str, screen_center_x: f32, y: f32, scale: f32) -> Vec2 {
    let total_width = text.chars().count() as f32 * GLYPH_PITCH.x * scale;
    Vec2::new(screen_center_x - total_width / 2.0, y)
}

/// Convenience wrapper for default text layout (white, no color) with a custom scale.
pub fn layout_text_scaled(text: &str, origin: Vec2, scale: f32) -> Vec<PositionedGlyph> {
    let colored: Vec<(char, [f32; 4])> = text.chars().map(|c| (c, [1.0, 1.0, 1.0, 1.0])).collect();
    layout_colored_text_scaled(&colored, origin, scale)
}

/// Lays out a string starting at `origin` (top-left of the first
/// character), advancing left to right by GLYPH_PITCH.x per
/// character. Takes pre-colored characters instead of a plain
/// &str — used for dialogue's per-span coloring and a custom scale.
pub fn layout_colored_text_scaled(
    chars: &[(char, [f32; 4])],
    origin: Vec2,
    scale: f32,
) -> Vec<PositionedGlyph> {
    let mut glyphs = Vec::new();
    let mut cursor = origin;

    for &(c, color) in chars {
        if c == ' ' {
            cursor.x += GLYPH_PITCH.x * scale;
            continue;
        }
        match glyph_cell(c) {
            Some(cell) => {
                glyphs.push(PositionedGlyph {
                    cell,
                    position: cursor,
                    color,
                    scale,
                });
                cursor.x += GLYPH_PITCH.x * scale;
            }
            None => {
                // TODO: add font name to something like environment variables to be used
                // for warning. Environment variables also has the potential of being used
                // for making the entity properties' default values configurable through engine GUI
                log::warn!(
                    "no glyph for character {c:?} - in <CURRENTLY_USED_FONT> font — skipped"
                );
                cursor.x += GLYPH_PITCH.x * scale; // still advance, so later characters don't overlap the gap
            }
        }
    }
    glyphs
}

/// Splits a line's spans into multiple visual lines, breaking at word
/// boundaries so no line exceeds max_width. Each returned line is
/// itself a Vec<(char, [f32;4])> — ready to hand straight to
/// layout_colored_text_scaled, one call per line, with the caller
/// choosing each line's y position. Doesn't touch layout_colored_text_scaled
/// itself, so every existing caller (debug text, toasts) is unaffected.
pub fn wrap_colored_text(
    chars: &[(char, [f32; 4])],
    max_width: f32,
    scale: f32,
) -> Vec<Vec<(char, [f32; 4])>> {
    let char_width = GLYPH_PITCH.x * scale;
    let mut lines: Vec<Vec<(char, [f32; 4])>> = vec![Vec::new()];
    let mut current_word: Vec<(char, [f32; 4])> = Vec::new();
    let mut current_line_width = 0.0;
    let mut current_word_width = 0.0;

    let flush_word = |lines: &mut Vec<Vec<(char, [f32; 4])>>,
                      current_word: &mut Vec<(char, [f32; 4])>,
                      current_line_width: &mut f32,
                      current_word_width: &mut f32| {
        if current_word.is_empty() {
            return;
        }
        if *current_line_width + *current_word_width > max_width && *current_line_width > 0.0 {
            lines.push(Vec::new());
            *current_line_width = 0.0;
        }
        lines.last_mut().unwrap().append(current_word);
        *current_line_width += *current_word_width;
        *current_word_width = 0.0;
    };

    for &(c, color) in chars {
        if c == ' ' {
            flush_word(
                &mut lines,
                &mut current_word,
                &mut current_line_width,
                &mut current_word_width,
            );
            lines.last_mut().unwrap().push((' ', color));
            current_line_width += char_width;
        } else {
            current_word.push((c, color));
            current_word_width += char_width;
        }
    }
    flush_word(
        &mut lines,
        &mut current_word,
        &mut current_line_width,
        &mut current_word_width,
    );

    lines
}
