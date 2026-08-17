use crate::engine::text;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
    tint: [f32; 4],
}

impl Vertex {
    pub(super) fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3, // Position
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2, // UV coordinates
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3 + 2]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4, // Tint color
                },
            ],
        }
    }
}

pub struct SolidRect {
    pub position: glam::Vec2,
    pub size: glam::Vec2,
    pub fill_color: [f32; 4],
    pub border_color: [f32; 4],
    pub border_thickness_px: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct SolidVertex {
    position: [f32; 3],
    /// 0..1 position within this specific rect — same role tex_coords
    /// played before (which corner of the unit quad this is), just
    /// renamed since there's no texture being sampled here. The
    /// fragment shader still needs this to know "how close to an
    /// edge am I" for the border test.
    local_uv: [f32; 2],
    fill_color: [f32; 4],
    border_color: [f32; 4],
    /// Per-axis thickness in local_uv space — same calculation
    /// ADR-043 already does (pixels / rect_width, pixels / rect_height),
    /// just baked into each vertex now instead of a per-draw uniform.
    border_thickness: [f32; 2],
}

impl SolidVertex {
    pub(super) fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SolidVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3, // position
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2, // local_uv
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3 + 2]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4, // fill_color
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3 + 2 + 4]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4, // border_color
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3 + 2 + 4 + 4]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x2, // border_thickness
                },
            ],
        }
    }
}

pub(super) const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.0, 0.0],
        tex_coords: [0.0, 0.0],
        tint: [1.0, 1.0, 1.0, 1.0],
    },
    Vertex {
        position: [0.0, 1.0, 0.0],
        tex_coords: [0.0, 1.0],
        tint: [1.0, 1.0, 1.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
        tex_coords: [1.0, 1.0],
        tint: [1.0, 1.0, 1.0, 1.0],
    },
    Vertex {
        position: [1.0, 0.0, 0.0],
        tex_coords: [1.0, 0.0],
        tint: [1.0, 1.0, 1.0, 1.0],
    },
];

pub(super) const INDICES: &[u16] = &[0, 1, 2, 0, 3, 2];

/// Builds the model matrix for a sprite: scale a unit quad to `size`,
/// then translate so `position` lands at the sprite's center (ADR-033).
pub(super) fn model_matrix(position: glam::Vec2, size: glam::Vec2) -> glam::Mat4 {
    let half_size = size / 2.0;
    let scale = glam::Mat4::from_scale(size.extend(1.0));
    let translate = glam::Mat4::from_translation((position - half_size).extend(0.0));
    translate * scale
}

/// Builds one vertex+index buffer for an entire string — 4 vertices
/// and 6 indices per glyph, with each glyph's final screen position
/// and atlas UV baked directly into its vertices. Unlike the earlier
/// per-glyph-uniform approach, no per-draw remapping is needed: the
/// vertex data already is the answer, so the whole string draws in
/// one draw call instead of one per glyph.
pub(super) fn build_text_mesh(glyphs: &[text::PositionedGlyph]) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::with_capacity(glyphs.len() * 4);
    let mut indices = Vec::with_capacity(glyphs.len() * 6);

    for glyph in glyphs {
        let (uv_min, uv_max) = text::glyph_uv(glyph.cell.0, glyph.cell.1);

        let top_left = glyph.position;
        let bottom_right = glyph.position + text::GLYPH_SIZE * glyph.scale;

        let base = vertices.len() as u16;
        vertices.push(Vertex {
            position: [top_left.x, top_left.y, 0.0],
            tex_coords: [uv_min.x, uv_min.y],
            tint: glyph.color,
        });
        vertices.push(Vertex {
            position: [top_left.x, bottom_right.y, 0.0],
            tex_coords: [uv_min.x, uv_max.y],
            tint: glyph.color,
        });
        vertices.push(Vertex {
            position: [bottom_right.x, bottom_right.y, 0.0],
            tex_coords: [uv_max.x, uv_max.y],
            tint: glyph.color,
        });
        vertices.push(Vertex {
            position: [bottom_right.x, top_left.y, 0.0],
            tex_coords: [uv_max.x, uv_min.y],
            tint: glyph.color,
        });

        // Same 0,1,2,0,3,2 winding as the existing static VERTICES/
        // INDICES pair, just offset per glyph via `base`.
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 3, base + 2]);
    }

    (vertices, indices)
}

pub(super) fn build_solid_rect_mesh(rects: &[SolidRect]) -> (Vec<SolidVertex>, Vec<u16>) {
    let mut vertices = Vec::with_capacity(rects.len() * 4);
    let mut indices = Vec::with_capacity(rects.len() * 6);
    for rect in rects {
        let half_size = rect.size / 2.0;
        let top_left = rect.position - half_size;
        let bottom_right = rect.position + half_size;
        let thickness = [
            rect.border_thickness_px / rect.size.x,
            rect.border_thickness_px / rect.size.y,
        ];

        let base = vertices.len() as u16;
        vertices.push(SolidVertex {
            position: [top_left.x, top_left.y, 0.0],
            local_uv: [0.0, 0.0],
            fill_color: rect.fill_color,
            border_color: rect.border_color,
            border_thickness: thickness,
        });
        vertices.push(SolidVertex {
            position: [top_left.x, bottom_right.y, 0.0],
            local_uv: [0.0, 1.0],
            fill_color: rect.fill_color,
            border_color: rect.border_color,
            border_thickness: thickness,
        });
        vertices.push(SolidVertex {
            position: [bottom_right.x, bottom_right.y, 0.0],
            local_uv: [1.0, 1.0],
            fill_color: rect.fill_color,
            border_color: rect.border_color,
            border_thickness: thickness,
        });
        vertices.push(SolidVertex {
            position: [bottom_right.x, top_left.y, 0.0],
            local_uv: [1.0, 0.0],
            fill_color: rect.fill_color,
            border_color: rect.border_color,
            border_thickness: thickness,
        });
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 3, base + 2]);
    }
    (vertices, indices)
}
