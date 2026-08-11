use glam::Vec2;

use crate::engine::ids::{SceneId, TextureId, WarpId};

pub struct Entity {
    /// World-space center, in pixels (ADR-033).
    pub position: Vec2,
    /// Sprite dimensions in pixels, also drives the y-sort baseline.
    pub size: Vec2,
    /// Solid area, relative to `position`, in pixels. If `None`, the entity is non-solid and can be walked through.
    pub collider: Option<Collider>,
    /// Which texture to render for this entity, if any. If `None`, the entity is invisible.
    pub texture_id: Option<TextureId>,
}

/// Axis-aligned rectangle (AABB): center offset + half-extents.
/// Used for collision detection and other spatial queries.
pub struct Rect {
    /// Center offset from the entity's position, where box's center sits relative to the entity's position.
    /// For walls (which have no entity), this is used directly as a world-space center instead.
    pub center: Vec2,
    /// Half-size of the rectangle, for overlap detection.
    pub half_size: Vec2,
}

pub struct Background {
    pub texture: TextureId,
    pub position: Vec2,
    pub size: Vec2,
}

/// A solid region that blocks movement — walls, entity colliders.
/// Wraps Rect, paired with Trigger: Collider blocks, Trigger fires.
pub struct Collider {
    pub rect: Rect,
}

/// A non-solid region that fires an effect on overlap, distinct from
/// walls/colliders (which block movement). Reuses Rect for geometry —
/// only the meaning differs (ADR-046). rect is world-space, matching
/// the walls convention (ADR-038), not entity-relative.
pub struct Trigger {
    pub rect: Rect,
    pub kind: TriggerKind,
    /// True immediately after the player arrives via this trigger;
    /// suppresses re-firing until the player's center leaves rect.
    /// Transient play-session state, not content (parallel to
    /// Entity's Option fields, ADR-037). (ADR-051)
    pub recently_used: bool,
}

/// What a trigger does when the player's center enters it. Single
/// variant for now, deliberately — dispatch shape is decided once a
/// second kind exists to compare against (ADR-046).
pub enum TriggerKind {
    Warp {
        /// This trigger's own identity — how a warp from elsewhere
        /// finds *this* trigger inside the destination scene's
        /// Vec<Trigger>. (ADR-051)
        warp_id: WarpId,
        target_scene: SceneId,
        target_warp_id: WarpId,
        /// Offset from this trigger's rect.center where an arriving
        /// player actually spawns — kept separate from the trigger's
        /// detection geometry (rect), since "how big is the doorway
        /// overlap zone" and "how far into the room do you land" are
        /// independently tunable. Direction is scene-specific: a door
        /// in a south wall wants a different offset than one in a
        /// north wall.
        spawn_offset: Vec2,
    },
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

/// True if a point falls within a rect's bounds — used for triggers
/// (doors, zones): "stand inside it, something happens." Distinct from
/// aabb_overlap, used for solid colliders: "touch it, you're blocked."
pub fn point_in_rect(point: Vec2, rect: &Rect) -> bool {
    let min = rect.center - rect.half_size;
    let max = rect.center + rect.half_size;
    point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
}
