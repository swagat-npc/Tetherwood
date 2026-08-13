use glam::Vec2;

/// Good Neighbors font, hand-arranged (in Aseprite) into a uniform
/// 10x9 grid — no external metadata file needed, since every glyph
/// occupies an identical cell and position is pure arithmetic from
/// character order, not derived from a packed/variable-width atlas.
pub const GLYPH_SIZE: Vec2 = Vec2::new(9.0, 15.0);
pub const ATLAS_SIZE: Vec2 = Vec2::new(100.0, 144.0);
/// Distance from one cell's origin to the next — glyph size plus the
/// 1px gap on all sides. Distinct from GLYPH_SIZE: the glyph itself
/// doesn't fill the full pitch, narrow characters (e.g. '1') just
/// leave their cell's remainder blank, same as any monospace font.
pub const GLYPH_PITCH: Vec2 = Vec2::new(10.0, 16.0);

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
}

/// Plain &str convenience wrapper for callers
/// (F3, FPS counter, mouse position) that don't need color.
pub fn layout_text(text: &str, origin: Vec2) -> Vec<PositionedGlyph> {
    let colored: Vec<(char, [f32; 4])> = text.chars().map(|c| (c, [1.0, 1.0, 1.0, 1.0])).collect();
    layout_colored_text(&colored, origin)
}

/// Lays out a string starting at `origin` (top-left of the first
/// character), advancing left to right by GLYPH_PITCH.x per
/// character. Takes pre-colored characters instead of a plain
/// &str — used for dialogue's per-span coloring.
pub fn layout_colored_text(chars: &[(char, [f32; 4])], origin: Vec2) -> Vec<PositionedGlyph> {
    let mut glyphs = Vec::new();
    let mut cursor = origin;

    for &(c, color) in chars {
        if c == ' ' {
            cursor.x += GLYPH_PITCH.x;
            continue;
        }
        match glyph_cell(c) {
            Some(cell) => {
                glyphs.push(PositionedGlyph {
                    cell,
                    position: cursor,
                    color,
                });
                cursor.x += GLYPH_PITCH.x;
            }
            None => {
                // TODO: add font name to something like environment variables to be used
                // for warning. Environment variables also has the potential of being used
                // for making the entity properties' default values configurable through engine GUI
                log::warn!(
                    "no glyph for character {c:?} - in <CURRENTLY_USED_FONT> font — skipped"
                );
                cursor.x += GLYPH_PITCH.x; // still advance, so later characters don't overlap the gap
            }
        }
    }
    glyphs
}
