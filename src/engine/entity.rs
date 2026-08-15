use glam::Vec2;

use crate::engine::ids::{EntityId, SceneId, TextureId, WarpId};

pub struct Entity {
    /// World-space center, in pixels (ADR-033).
    pub position: Vec2,
    /// Sprite dimensions in pixels, also drives the y-sort baseline.
    pub size: Vec2,
    /// Solid area, relative to `position`, in pixels. If `None`, the entity is non-solid and can be walked through.
    pub collider: Option<Collider>,
    /// Which texture to render for this entity, if any. If `None`, the entity is invisible.
    pub texture_id: Option<TextureId>,
    /// Last non-idle movement direction. Defaults to Down at construction; updated only when movement is nonzero.
    pub facing: Direction,
}

impl Entity {
    /// World-space center of this entity's collider, if it has one —
    /// falls back to the entity's own sprite-center position
    /// otherwise. Distinct from `position` (ADR-033's sprite-center
    /// convention): a collider is frequently offset from its sprite's
    /// visual center (e.g. feet/hips vs. head), and anything checking
    /// "is this entity here" for physical/trigger purposes should use
    /// this, not the raw sprite position.
    pub fn collider_center(&self) -> Vec2 {
        match &self.collider {
            Some(collider) => self.position + collider.rect.center,
            None => self.position,
        }
    }
}

/// Axis-aligned rectangle (AABB): center offset + half-extents.
/// Used for collision detection and other spatial queries.
#[derive(Debug, Clone, Copy)]
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

/// What a trigger does when the player's center enters it. Two
/// variants: Warp (scene transition) and Interact (proximity +
/// facing + button). A single-variant enum was sufficient until
/// Interact arrived as the second real consumer (ADR-046).
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
    Dialogue {
        /// Content identifier, resolved by game::dialogue::line_for.
        id: &'static str,
        /// The floating prompt-icon entity, shown/hidden by proximity
        /// alone. Multiple triggers may share one prompt_entity (e.g.
        /// an object reachable from two sides) — see
        /// Scene::update_interact_prompts for how that's resolved.
        prompt_entity: Option<EntityId>,
        prompt_texture: Option<TextureId>,
        /// Facing direction(s) that make this specific trigger's rect
        /// valid. A slice because one rect can accept more than one
        /// facing (e.g. a straight-on approach from either side of a
        /// symmetric object) — but each rect still only knows about
        /// facings correct for *that* rect's position; an object
        /// reachable from multiple distinct sides needs one Trigger
        /// per side, not one Trigger with every direction listed.
        required_facing: &'static [Direction],
    },
    Toggle {
        target_entity: EntityId,
        closed_texture: TextureId,
        open_texture: TextureId,
        closed_collider: Rect,
        required_facing: &'static [Direction],
    },
}

/// Four-directional facing — matches how pixel-art directional sprites
/// actually work (separate up/down/left/right frame sets, not
/// continuous angles). Diagonal movement still picks one dominant
/// cardinal direction, standard for GBA/SNES-era games.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Picks the dominant axis of a movement vector. Diagonal input
    /// has no corresponding sprite direction, so whichever axis is
    /// larger wins. Returns None for zero movement — caller keeps
    /// whatever facing was already set (facing persists while idle).
    pub fn from_movement(movement: Vec2) -> Option<Direction> {
        if movement == Vec2::ZERO {
            return None;
        }
        Some(if movement.x.abs() > movement.y.abs() {
            if movement.x > 0.0 {
                Direction::Right
            } else {
                Direction::Left
            }
        } else if movement.y > 0.0 {
            Direction::Down
        } else {
            Direction::Up
        })
    }
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
