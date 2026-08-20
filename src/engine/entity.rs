use crate::engine::renderer::texture::TextureId;
use glam::Vec2;

/// Indexes into a Scene's entities Vec (ADR-025). Stays valid for the
/// scene's whole lifetime — entities are only ever appended, never
/// removed or reordered (ADR-037), same guarantee TextureId already
/// relies on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub usize);

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
    /// Whether this entity is active and should be rendered/updated.
    pub active: bool,
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

    pub fn world_collider(&self) -> Option<Rect> {
        self.collider.as_ref().map(|c| Rect {
            center: self.position + c.rect.center,
            half_size: c.rect.half_size,
        })
    }

    pub fn deactivate(&mut self) {
        self.active = false;
        self.collider = None;
        self.texture_id = None;
    }

    pub fn match_facing_direction(&self, facing: &'static [Direction]) -> bool {
        match self.facing {
            Direction::Up => facing.contains(&Direction::Down),
            Direction::Down => facing.contains(&Direction::Up),
            Direction::Left => facing.contains(&Direction::Right),
            Direction::Right => facing.contains(&Direction::Left),
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

/// A solid region that blocks movement — walls, entity colliders.
/// Wraps Rect, paired with Trigger: Collider blocks, Trigger fires.
pub struct Collider {
    pub rect: Rect,
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

/// True if `player_facing`, from `player_center`, is oriented toward
/// the rect described by `target_center`/`target_half_size` — not
/// just its exact center point. Checks two things: the target is on
/// the correct side (delta sign on the facing axis), and the player
/// is actually within the target's extent on the *other* axis — the
/// second check is what stops a player standing flush against one
/// side of a wide target from "facing toward" it while looking along
/// a perpendicular direction (e.g. flush against a villager's right
/// edge, facing up, without the villager being above them at all).
/// Discovered as a real bug during Beat 2/M5's NPC interact work,
/// where a single-trigger, any-side-approachable NPC first exposed
/// this — a plain center-to-center direction check isn't enough once
/// a target is wider than a point.
pub fn is_facing_toward(
    player_center: Vec2,
    target_center: Vec2,
    target_half_size: Vec2,
    player_facing: Direction,
) -> bool {
    let delta = target_center - player_center;

    // Target boundaries
    let target_left_edge = target_center.x - target_half_size.x;
    let target_right_edge = target_center.x + target_half_size.x;
    let target_top_edge = target_center.y - target_half_size.y;
    let target_bottom_edge = target_center.y + target_half_size.y;

    let x_in_bounds = player_center.x >= target_left_edge && player_center.x <= target_right_edge;
    let y_in_bounds = player_center.y >= target_top_edge && player_center.y <= target_bottom_edge;

    match player_facing {
        Direction::Up => delta.y < 0.0 && x_in_bounds,
        Direction::Down => delta.y > 0.0 && x_in_bounds,
        Direction::Left => delta.x < 0.0 && y_in_bounds,
        Direction::Right => delta.x > 0.0 && y_in_bounds,
    }
}
