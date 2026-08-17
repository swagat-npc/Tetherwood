use glam::Vec2;
use std::collections::HashMap;

use crate::engine::entity::{
    Collider, Direction, Entity, EntityId, Rect, aabb_overlap, point_in_rect,
};
use crate::engine::texture::{TextureId, TextureStore};

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

impl Scene {
    pub fn player(&self) -> &Entity {
        &self.entities[self.player_index]
    }

    pub fn player_mut(&mut self) -> &mut Entity {
        &mut self.entities[self.player_index]
    }

    /// Checks whether a proposed world-space collider (center + half-size)
    /// would overlap any wall or any other entity's collider. `skip_index`
    /// excludes the entity performing the check, so it never collides
    /// with its own collider.
    fn collider_blocked(
        &self,
        world_center: Vec2,
        half_size: Vec2,
        skip_index: usize,
    ) -> Option<Rect> {
        for wall in &self.walls {
            if aabb_overlap(
                world_center,
                half_size,
                wall.rect.center,
                wall.rect.half_size,
            ) {
                return Some(Rect {
                    center: wall.rect.center,
                    half_size: wall.rect.half_size,
                });
            }
        }

        for (i, entity) in self.entities.iter().enumerate() {
            if i == skip_index {
                continue;
            }
            if let Some(collider) = &entity.collider {
                let other_center = entity.position + collider.rect.center;
                if aabb_overlap(
                    world_center,
                    half_size,
                    other_center,
                    collider.rect.half_size,
                ) {
                    return Some(Rect {
                        center: other_center,
                        half_size: collider.rect.half_size,
                    });
                }
            }
        }

        None
    }

    pub fn check_triggers(&mut self, show_debug_info: bool) -> Option<(SceneId, WarpId)> {
        let player_center = self.player().collider_center();

        // Clear recently_used for any trigger the player has now left and
        // re-arms it for the next time it's actually walked into again.
        for trigger in self.triggers.iter_mut() {
            if trigger.recently_used && !point_in_rect(player_center, &trigger.rect) {
                trigger.recently_used = false;
            }
        }

        for trigger in &self.triggers {
            if trigger.recently_used {
                continue;
            }
            let TriggerKind::Warp {
                warp_id,
                target_scene,
                target_warp_id,
                ..
            } = trigger.kind
            else {
                continue; // not a warp trigger — check_triggers only resolves warps
            };
            if point_in_rect(player_center, &trigger.rect) {
                if show_debug_info {
                    println!(
                        "{:?}:{} -> {:?}:{}",
                        self.id, warp_id.0, target_scene, target_warp_id.0
                    );
                }
                return Some((target_scene, target_warp_id));
            }
        }
        None
    }

    /// Checks whether the player is currently in range of, and correctly
    /// facing, an Interact trigger. Called from the interact-button press
    /// handler — correctness only matters at the instant of the press.
    pub fn try_interact(&self) -> Option<InteractResult> {
        let player = self.player();

        for trigger in &self.triggers {
            if !trigger.active {
                continue;
            }
            match &trigger.kind {
                TriggerKind::Dialogue {
                    id,
                    required_facing,
                    consumes_entity,
                    ..
                } => {
                    if point_in_rect(player.collider_center(), &trigger.rect)
                        && required_facing.contains(&player.facing)
                    {
                        return Some(InteractResult::Dialogue(*id, *consumes_entity));
                    }
                }
                TriggerKind::Toggle {
                    target_entity,
                    required_facing,
                    ..
                } => {
                    if point_in_rect(player.collider_center(), &trigger.rect)
                        && required_facing.contains(&player.facing)
                    {
                        return Some(InteractResult::Toggle(*target_entity));
                    }
                }
                TriggerKind::Warp { .. } => continue,
            }
        }
        None
    }

    /// Shows/hides every interact prompt icon based on proximity alone —
    /// no facing check, since the icon is a "something's here" cue,
    /// distinct from the facing-gated action. Call unconditionally, every
    /// frame. Multiple triggers can share one prompt_entity (e.g. two
    /// approach sides for one object); visibility is true if the player
    /// is in range of *any* of them, not just whichever was checked last.
    pub fn update_interact_prompts(&mut self) {
        let player_position = self.player().collider_center();
        let mut visible: HashMap<EntityId, (bool, TextureId)> = HashMap::new();

        for trigger in &self.triggers {
            if !trigger.active {
                continue;
            }
            if let TriggerKind::Dialogue {
                prompt_entity,
                prompt_texture,
                ..
            } = trigger.kind
            {
                let (Some(prompt_entity), Some(prompt_texture)) = (prompt_entity, prompt_texture)
                else {
                    continue;
                };
                let in_range = point_in_rect(player_position, &trigger.rect);
                let entry = visible
                    .entry(prompt_entity)
                    .or_insert((false, prompt_texture));
                entry.0 = entry.0 || in_range;
            }
        }

        for (entity_id, (is_visible, texture)) in visible {
            self.entities[entity_id.0].texture_id = if is_visible { Some(texture) } else { None };
        }
    }

    /// Marks the trigger matching `warp_id` as just-arrived-at (recently_used),
    /// and returns its position — the caller spawns the player there. Runs
    /// every time a warp is used, so position is always freshly set
    /// rather than left over from wherever the player was on a previous visit.
    pub fn activate_warp(&mut self, warp_id: WarpId) -> Option<Vec2> {
        for trigger in self.triggers.iter_mut() {
            let TriggerKind::Warp {
                warp_id: this_warp_id,
                spawn_offset,
                ..
            } = trigger.kind
            else {
                continue;
            };
            if this_warp_id == warp_id {
                trigger.recently_used = true;
                return Some(trigger.rect.center + spawn_offset);
            }
        }
        None
    }

    /// Attempts to move the player by `delta`. Resolves collisions
    /// sequentially, per axis — x first, then y from the (possibly
    /// already-updated) x — which is what produces sliding along a
    /// wall or furniture edge on diagonal movement, per the M3
    /// collision design.
    pub fn try_move_player(&mut self, delta: Vec2) {
        let idx = self.player_index;

        // `let ... else` — new syntax: if the pattern on the left
        // doesn't match, the `else` block must diverge (here, `return`)
        // rather than continue with some fallback value. Used here
        // because a player with no collider has nothing to check
        // against; we just move it and stop.
        let Some(collider) = self.entities[idx].collider.as_ref() else {
            self.entities[idx].position += delta;
            return;
        };
        let (offset, half_size) = (collider.rect.center, collider.rect.half_size);

        // Recover the original, un-normalized direction and full speed
        // magnitude, so a blocked axis can hand its "unused" speed budget
        // to the other axis instead of leaving it diagonal-reduced.
        let direction = delta.normalize_or_zero();
        let full_speed = delta.length();

        // First pass: check whether each axis's *original, diagonal-sized*
        // proposal would be blocked - using the same starting position for
        // both checks, since neither axis has actually moved yet.
        let start = self.entities[idx].position;

        let x_probe = Vec2::new(start.x + delta.x, start.y) + offset;
        let x_blocked_initially = self.collider_blocked(x_probe, half_size, idx).is_some();

        let y_probe = Vec2::new(start.x, start.y + delta.y) + offset;
        let y_blocked_initially = self.collider_blocked(y_probe, half_size, idx).is_some();

        // Each axis's actual delta: full, undiminished speed in its own
        // direction if the *other* axis was blocked (nothing left to share
        // the diagonal split with) - otherwise the original diagonal delta.
        let x_delta = if y_blocked_initially && direction.x != 0.0 {
            direction.x.signum() * full_speed
        } else {
            delta.x
        };
        let y_delta = if x_blocked_initially && direction.y != 0.0 {
            direction.y.signum() * full_speed
        } else {
            delta.y
        };

        // X axis
        let proposed_x = start.x + x_delta;
        let world_center = Vec2::new(proposed_x, start.y) + offset;
        let x_blocked = self.collider_blocked(world_center, half_size, idx);

        match x_blocked {
            Some(obstacle) => {
                // The player's collider edge lands exactly on the obstacle's
                // edge - computed from geometry alone, never from delta.x, so
                // it's the same fixed answer every frame regardless of speed.
                let target_collider_center_x = if x_delta > 0.0 {
                    obstacle.center.x - obstacle.half_size.x - half_size.x
                } else {
                    obstacle.center.x + obstacle.half_size.x + half_size.x
                };
                self.entities[idx].position.x = target_collider_center_x - offset.x;
            }
            None => {
                self.entities[idx].position.x = proposed_x;
            }
        }

        // Y axis, from whatever x just resulted from the check above.
        let proposed_y = self.entities[idx].position.y + y_delta;
        let world_center = Vec2::new(self.entities[idx].position.x, proposed_y) + offset;

        match self.collider_blocked(world_center, half_size, idx) {
            Some(obstacle) => {
                let target_collider_center_y = if delta.y > 0.0 {
                    obstacle.center.y - obstacle.half_size.y - half_size.y
                } else {
                    obstacle.center.y + obstacle.half_size.y + half_size.y
                };
                self.entities[idx].position.y = target_collider_center_y - offset.y;
            }
            None => {
                self.entities[idx].position.y = proposed_y;
            }
        }
    }

    /// Flips a Toggle trigger's target entity between its open/closed
    /// texture and collider. State is inferred from which texture the
    /// entity currently has (ADR-037's Option-as-state pattern), not
    /// tracked separately.
    pub fn toggle_entity(&mut self, target_entity: EntityId) {
        let Some(trigger) = self.triggers.iter().find(|t| {
            matches!(t.kind, TriggerKind::Toggle { target_entity: te, .. } if te == target_entity)
        }) else {
            return;
        };
        let TriggerKind::Toggle {
            closed_texture,
            open_texture,
            closed_collider,
            ..
        } = trigger.kind
        else {
            return;
        };

        let entity = &mut self.entities[target_entity.0];
        if entity.texture_id == Some(closed_texture) {
            entity.texture_id = Some(open_texture);
            entity.collider = None;
        } else {
            entity.texture_id = Some(closed_texture);
            entity.collider = Some(Collider {
                rect: closed_collider,
            });
        }
    }

    // Scene method, mirroring toggle_entity's search pattern
    pub fn consume_entity(&mut self, entity_id: EntityId) {
        self.entities[entity_id.0].texture_id = None;
        self.entities[entity_id.0].collider = None;
        for trigger in self.triggers.iter_mut() {
            if let TriggerKind::Dialogue {
                consumes_entity: Some(id),
                prompt_entity,
                ..
            } = trigger.kind
            {
                if id == entity_id {
                    trigger.active = false;
                    if let Some(prompt_entity) = prompt_entity {
                        self.entities[prompt_entity.0].texture_id = None;
                    }
                }
            }
        }
    }
}
