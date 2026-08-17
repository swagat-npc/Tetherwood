mod mechanics;

use crate::engine::entity::{
    Collider, Direction, Entity, EntityId, Rect, aabb_overlap, point_in_rect,
};
use crate::engine::renderer::texture::{TextureId, TextureStore};
use glam::Vec2;

/// Per-scene camera behavior (ADR-041). Static holds its own fixed
/// anchor point; Follow has no stored data — it reads the player's
/// current position fresh every frame, in the render path, not here.
#[derive(Debug, Clone, Copy)]
pub enum CameraMode {
    Static(Vec2),
    Follow,
}

pub enum InteractResult {
    Dialogue(&'static str, Option<EntityId>),
    Toggle(EntityId),
}

pub struct Background {
    pub texture: TextureId,
    pub position: Vec2,
    pub size: Vec2,
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
    /// False once this trigger's effect has been permanently consumed
    /// (e.g. the necklace's dialogue finished and removed it) —
    /// checked before any detection logic runs. Distinct from
    /// recently_used, which is transient (re-arms on leaving the
    /// rect); this is permanent for the scene's remaining lifetime.
    pub active: bool,
}

/// Identifies a single warp point within one scene, and doubles as a
/// human-readable debug label — both jobs from one authored value, no
/// separate name registry to keep in sync (ADR-051). Uniqueness only
/// needs to hold within one scene, since a warp is always addressed
/// as (SceneId, WarpId) together.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WarpId(pub &'static str);

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
        /// Entity this dialogue's *last* line should consume (ADR-037's
        /// texture-clear pattern) once it closes — e.g. the necklace
        /// removing itself after being picked up. None for dialogue that
        /// doesn't remove anything (the bed's lore-drop).
        consumes_entity: Option<EntityId>,
    },
    Toggle {
        target_entity: EntityId,
        closed_texture: TextureId,
        open_texture: TextureId,
        closed_collider: Rect,
        required_facing: &'static [Direction],
    },
}

/// Identifies a scene by name, independent of whether that scene is
/// currently loaded. Used by warps (Trigger::Warp) to name a
/// destination scene that may not exist in memory yet (ADR-048: scenes
/// are lazily constructed on entry, not kept resident).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneId {
    Home,
    Outside,
}

pub struct Scene {
    pub id: SceneId,
    pub background: Vec<Background>,
    pub walls: Vec<Collider>,
    pub triggers: Vec<Trigger>,
    pub entities: Vec<Entity>,
    pub texture_store: TextureStore,
    pub player_index: usize,
    pub camera_mode: CameraMode,
}
