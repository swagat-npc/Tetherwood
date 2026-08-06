use glam::Vec2;

/// Unique identifier for a texture stored in a TextureStore.
/// Simple newtype wrapper around a usize index, made distinct to avoid accidental mixup.
#[derive(Clone, Copy)]
pub struct TextureId(pub usize);

pub struct Entity {
    /// World-space center, in pixels (ADR-033).
    pub position: Vec2,
    /// Sprite dimensions in pixels, also drives the y-sort baseline.
    pub size: Vec2,
    /// Solid area, relative to `position`, in pixels. If `None`, the entity is non-solid and can be walked through.
    pub collider: Option<Rect>,
    /// Which texture to render for this entity, if any. If `None`, the entity is invisible.
    pub texture_id: Option<TextureId>,
}

/// Axis-aligned rectangle (AABB): center offset + half-extents.
/// Used for collision detection and other spatial queries.
pub struct Rect {
    /// Center offset from the entity's position, where box's center sits relative to the entity's position.
    /// For walls (which have no entity), this is used directly as a world-space center instead.
    pub offset: Vec2,
    /// Half-size of the rectangle, for overlap detection.
    pub half_size: Vec2,
}

/// Returns true if two axis-aligned rectangles — each given as a
/// world-space center and half-extents — overlap. Per axis, the rects
/// overlap only if their centers are closer together than the sum of
/// their half-sizes; failing on either axis alone proves separation
/// (derived and hand-traced during M3 design).
pub fn aabb_overlap(center_a: Vec2, half_a: Vec2, center_b: Vec2, half_b: Vec2) -> bool {
    (center_a.x - center_b.x).abs() < half_a.x + half_b.x
        && (center_a.y - center_b.y).abs() < half_a.y + half_b.y
}
