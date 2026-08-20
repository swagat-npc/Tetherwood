use super::{
    Collider, Entity, EntityId, InteractResult, Rect, Scene, SceneId, TriggerKind, WarpId,
    aabb_overlap, point_in_rect,
};
use crate::engine::entity::{Direction, is_facing_toward};
use crate::engine::grid;
use crate::engine::renderer::texture::{TextureId, TextureStore};
use crate::engine::scene::{CameraMode, WallId};
use glam::Vec2;
use std::collections::HashMap;

impl Scene {
    pub fn new(
        id: SceneId,
        texture_store: TextureStore,
        orthographic_camera_mode: CameraMode,
        isometric_camera_mode: CameraMode,
        is_isometric: bool,
        multiplying_factor: f32,
    ) -> Self {
        let current_camera_mode = Self::resolve_camera_mode(
            orthographic_camera_mode,
            isometric_camera_mode,
            is_isometric,
        );
        let cell_size = grid::CELL_SIZE * multiplying_factor;
        Self {
            id,
            static_grid: grid::SpatialGrid::new(cell_size),
            dynamic_grid: grid::SpatialGrid::new(cell_size),
            background: Vec::new(),
            walls: Vec::new(),
            triggers: Vec::new(),
            entities: Vec::new(),
            texture_store,
            player_index: 0, // placeholder, set actual value when building the scene
            orthographic_camera_mode,
            isometric_camera_mode,
            current_camera_mode,
        }
    }

    pub fn player(&self) -> &Entity {
        &self.entities[self.player_index]
    }

    pub fn player_mut(&mut self) -> &mut Entity {
        &mut self.entities[self.player_index]
    }

    fn resolve_camera_mode(
        orthographic: CameraMode,
        isometric: CameraMode,
        is_isometric: bool,
    ) -> CameraMode {
        if is_isometric {
            isometric
        } else {
            orthographic
        }
    }

    pub fn camera_mode(&self) -> CameraMode {
        self.current_camera_mode
    }

    pub fn sync_camera_mode(&mut self, is_isometric: bool) {
        self.current_camera_mode = Self::resolve_camera_mode(
            self.orthographic_camera_mode,
            self.isometric_camera_mode,
            is_isometric,
        );
    }

    pub fn static_grid(&self) -> &grid::SpatialGrid {
        &self.static_grid
    }

    pub fn build_static_grid(&mut self, multiplying_factor: f32) {
        let cell_size = grid::CELL_SIZE * multiplying_factor;
        self.static_grid = grid::SpatialGrid::new(cell_size);

        for (i, wall) in self.walls.iter().enumerate() {
            self.static_grid
                .insert(&wall.rect, grid::CollisionHandle::Wall(WallId(i)));
        }

        for (i, entity) in self.entities.iter().enumerate() {
            if i == self.player_index {
                continue;
            }
            if let Some(rect) = entity.world_collider() {
                self.static_grid
                    .insert(&rect, grid::CollisionHandle::Entity(EntityId(i)));
            }
        }
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
        let mut candidates = self
            .static_grid
            .collision_handles_around_position(world_center, 1);
        candidates.extend(
            self.dynamic_grid
                .collision_handles_around_position(world_center, 1),
        );

        for handle in candidates {
            let candidate_rect = match handle {
                grid::CollisionHandle::Wall(wall_id) => self.walls[wall_id.0].rect,
                grid::CollisionHandle::Entity(entity_id) => {
                    if entity_id.0 == skip_index {
                        continue;
                    }
                    match self.entities[entity_id.0].world_collider() {
                        Some(rect) => rect,
                        None => continue,
                    }
                }
            };

            if aabb_overlap(
                world_center,
                half_size,
                candidate_rect.center,
                candidate_rect.half_size,
            ) {
                return Some(candidate_rect);
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
        for trigger in &self.triggers {
            if !trigger.active {
                continue;
            }
            match &trigger.kind {
                TriggerKind::Dialogue {
                    id,
                    facing: entity_facing,
                    consumes_entity,
                    sets_flag,
                    ..
                } => {
                    if self.player_flush_with(&trigger.rect, entity_facing) {
                        return Some(InteractResult::Dialogue(*id, *consumes_entity, *sets_flag));
                    }
                }
                TriggerKind::Toggle {
                    target_entity,
                    facing: entity_facing,
                    ..
                } => {
                    if self.player_near(&trigger.rect, entity_facing) {
                        return Some(InteractResult::Toggle(*target_entity));
                    }
                }
                TriggerKind::Warp { .. } => continue,
            }
        }
        None
    }

    /// Flush-contact interact check (Dialogue triggers): the player's own
    /// collider must genuinely overlap the trigger's rect, not just have
    /// its center point inside it - correct for interactables meant to
    /// be approached and touched (an NPC, a pickup), where a loose
    /// "somewhere nearby" check would let interaction fire from a spot
    /// that doesn't visually read as "next to it."
    fn player_flush_with(&self, trigger_rect: &Rect, entity_facing: &'static [Direction]) -> bool {
        let player = self.player();
        let Some(player_collider) = &player.collider else {
            return false;
        };
        aabb_overlap(
            player.collider_center(),
            player_collider.rect.half_size,
            trigger_rect.center,
            trigger_rect.half_size,
        ) && is_facing_toward(
            player.collider_center(),
            trigger_rect.center,
            trigger_rect.half_size,
            player.facing,
        ) && player.match_facing_direction(entity_facing)
    }

    /// Vicinity interact check (Toggle triggers): the player's center
    /// just needs to be inside the trigger's rect - correct for things
    /// like a door, where the interactable object's own collider can
    /// disappear (an open door has none), so there's no "flush" geometry
    /// left to test against; a wider zone the player can approach from
    /// nearby is the intended behavior instead.
    fn player_near(&self, trigger_rect: &Rect, entity_facing: &'static [Direction]) -> bool {
        let player = self.player();
        point_in_rect(player.collider_center(), trigger_rect)
            && is_facing_toward(
                player.collider_center(),
                trigger_rect.center,
                trigger_rect.half_size,
                player.facing,
            )
            && player.match_facing_direction(entity_facing)
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

    fn rebuild_dynamic_grid(&mut self, multiplying_factor: f32) {
        let cell_size = grid::CELL_SIZE * multiplying_factor;
        self.dynamic_grid = grid::SpatialGrid::new(cell_size);
        if let Some(rect) = self.player().world_collider() {
            self.dynamic_grid.insert(
                &rect,
                grid::CollisionHandle::Entity(EntityId(self.player_index)),
            );
        }
    }

    /// Attempts to move the player by `delta`. Resolves collisions
    /// sequentially, per axis — x first, then y from the (possibly
    /// already-updated) x — which is what produces sliding along a
    /// wall or furniture edge on diagonal movement, per the M3
    /// collision design.
    pub fn try_move_player(
        &mut self,
        delta: Vec2,
        multiplying_factor: f32,
        enable_player_collision: bool,
    ) {
        // Rebuild the dynamic grid to account for entity movement before collision detection
        self.rebuild_dynamic_grid(multiplying_factor);

        let idx = self.player_index;

        if !enable_player_collision {
            self.entities[idx].position += delta; // move freely, skip all collision resolution
            return;
        }

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
        self.entities[entity_id.0].deactivate();
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
                        self.entities[prompt_entity.0].deactivate();
                    }
                }
            }
        }
    }
}
