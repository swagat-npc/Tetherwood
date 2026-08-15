use anyhow::Result;
use glam::Vec2;
use std::collections::HashMap;

use crate::engine::entity::{
    Background, Collider, Direction, Entity, Rect, Trigger, TriggerKind, aabb_overlap,
    point_in_rect,
};
use crate::engine::ids::{EntityId, SceneId, WarpId};
use crate::engine::texture::TextureStore;

/// Per-scene camera behavior (ADR-041). Static holds its own fixed
/// anchor point; Follow has no stored data — it reads the player's
/// current position fresh every frame, in the render path, not here.
#[derive(Debug, Clone, Copy)]
pub enum CameraMode {
    Static(Vec2),
    Follow,
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
    pub fn try_interact(&self) -> Option<&'static str> {
        let player = self.player();
        let player_center = player.collider_center();

        for trigger in &self.triggers {
            let TriggerKind::Interact {
                id,
                required_facing,
                ..
            } = trigger.kind
            else {
                continue;
            };
            if !point_in_rect(player_center, &trigger.rect) {
                continue;
            }
            if required_facing.contains(&player.facing) {
                return Some(id);
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
        let mut visible: HashMap<EntityId, (bool, crate::engine::ids::TextureId)> = HashMap::new();

        for trigger in &self.triggers {
            if let TriggerKind::Interact {
                prompt_entity,
                prompt_texture,
                ..
            } = trigger.kind
            {
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

    // TODO: sliding along the wall decreases speed

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

        // X axis
        let proposed = Vec2::new(
            self.entities[idx].position.x + delta.x,
            self.entities[idx].position.y,
        );
        let world_center = proposed + offset;

        match self.collider_blocked(world_center, half_size, idx) {
            Some(obstacle) => {
                // The player's collider edge lands exactly on the obstacle's
                // edge — computed from geometry alone, never from delta.x, so
                // it's the same fixed answer every frame regardless of speed.
                let target_collider_center_x = if delta.x > 0.0 {
                    obstacle.center.x - obstacle.half_size.x - half_size.x
                } else {
                    obstacle.center.x + obstacle.half_size.x + half_size.x
                };
                self.entities[idx].position.x = target_collider_center_x - offset.x;
            }
            None => {
                self.entities[idx].position.x = proposed.x;
            }
        }

        // Y axis, from whatever x just resulted from the check above.
        let proposed = Vec2::new(
            self.entities[idx].position.x,
            self.entities[idx].position.y + delta.y,
        );
        let world_center = proposed + offset;

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
                self.entities[idx].position.y = proposed.y;
            }
        }
    }

    /// Beat 1's home: player's bedroom with a working south-door warp to
    /// the outside scene (SceneId::Outside, WarpId("door")). Sizes scale
    /// via multiplying_factor (ADR-042).
    pub fn new_home(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        multiplying_factor: f32,
    ) -> Result<Self> {
        let mut texture_store = TextureStore::new();

        // Create Background
        let mut background: Vec<Background> = Vec::new();

        let floor = texture_store.load(device, queue, "assets/bedroom.png")?;
        let floor_position = Vec2::new(64.0, 64.0) * multiplying_factor;
        let floor_size = Vec2::new(128.0, 128.0) * multiplying_factor;
        background.push(Background {
            texture: floor,
            position: floor_position,
            size: floor_size,
        });

        // Room half-extents, derived directly from the background — not
        // separately hand-tuned, so a resized background can never silently
        // desync from wall length again.
        let room_half_width = floor_size.x / 2.0;
        let room_half_height = floor_size.y / 2.0;

        // The one genuinely independent value: walls are always 8px thick
        // (half-extent on their thin axis), regardless of room size.
        let wall_thickness = 8.0 * multiplying_factor;

        let left_edge = floor_position.x - room_half_width;
        let right_edge = floor_position.x + room_half_width;
        let top_edge = floor_position.y - room_half_height;
        let bottom_edge = floor_position.y + room_half_height;

        // Door gap: matches door_tex's existing width/position (32px wide,
        // centered on the room's x-center) — derived from the door entity's
        // own dimensions, not a separately hand-picked number.
        let door_half_width = 16.0 * multiplying_factor;
        let door_center_x = floor_position.x;

        let south_gap_start = door_center_x - door_half_width;
        let south_gap_end = door_center_x + door_half_width;

        let south_west_half_width = (south_gap_start - left_edge) / 2.0;
        let south_west_center_x = left_edge + south_west_half_width;

        let south_east_half_width = (right_edge - south_gap_end) / 2.0;
        let south_east_center_x = south_gap_end + south_east_half_width;

        let door = texture_store.load(device, queue, "assets/door.aseprite")?;
        let door_position = Vec2::new(floor_position.x, bottom_edge);
        let door_size = Vec2::new(32.0, 16.0) * multiplying_factor;
        background.push(Background {
            texture: door,
            position: door_position,
            size: door_size,
        });

        // Create Walls
        let walls = vec![
            Collider {
                rect: Rect {
                    center: Vec2::new(floor_position.x, top_edge - wall_thickness),
                    half_size: Vec2::new(room_half_width, wall_thickness),
                },
            }, // north
            Collider {
                rect: Rect {
                    center: Vec2::new(south_west_center_x, bottom_edge + wall_thickness),
                    half_size: Vec2::new(south_west_half_width, wall_thickness),
                },
            }, // south-west (of the door)
            Collider {
                rect: Rect {
                    center: Vec2::new(south_east_center_x, bottom_edge + wall_thickness),
                    half_size: Vec2::new(south_east_half_width, wall_thickness),
                },
            }, // south-east (of the door)
            Collider {
                rect: Rect {
                    center: Vec2::new(left_edge - wall_thickness, floor_position.y),
                    half_size: Vec2::new(wall_thickness, room_half_height),
                },
            }, // west
            Collider {
                rect: Rect {
                    center: Vec2::new(right_edge + wall_thickness, floor_position.y),
                    half_size: Vec2::new(wall_thickness, room_half_height),
                },
            }, // east
        ];

        // Create Triggers
        // Trigger sits fully past the wall's outer edge — the player must
        // walk all the way through the doorway gap and beyond the threshold
        // before their center overlaps this, not just step into the gap.
        let door_trigger_depth = wall_thickness; // how far past the wall the trigger extends
        let door_trigger_center_y = bottom_edge + 2.0 * wall_thickness;

        let mut triggers: Vec<Trigger> = Vec::new();

        triggers.push(Trigger {
            rect: Rect {
                center: Vec2::new(door_center_x, door_trigger_center_y),
                half_size: Vec2::new(door_half_width, door_trigger_depth),
            },
            recently_used: false,
            kind: TriggerKind::Warp {
                warp_id: WarpId("door"),
                target_scene: SceneId::Outside,
                target_warp_id: WarpId("door"),
                spawn_offset: Vec2::new(0.0, -20.0 * multiplying_factor), // up, into the room
            },
        });

        // Create Entities
        let mut entities: Vec<Entity> = Vec::new();

        let wardrobe_tex = texture_store.load(device, queue, "assets/wardrobe.aseprite")?;
        entities.push(Entity {
            position: Vec2::new(16.0, 12.0) * multiplying_factor,
            size: Vec2::new(24.0, 40.0) * multiplying_factor,
            collider: Some(Collider {
                rect: Rect {
                    center: Vec2::new(0.0, 0.0) * multiplying_factor,
                    half_size: Vec2::new(12.0, 20.0) * multiplying_factor,
                },
            }),
            texture_id: Some(wardrobe_tex),
            facing: Direction::Down,
        });

        let bed_tex = texture_store.load(device, queue, "assets/bed.aseprite")?;

        let bed_collider = Rect {
            center: Vec2::new(0.0, 5.0) * multiplying_factor,
            half_size: Vec2::new(16.0, 22.0) * multiplying_factor,
        };
        entities.push(Entity {
            position: Vec2::new(16.0, 48.0) * multiplying_factor,
            size: Vec2::new(32.0, 64.0) * multiplying_factor,
            collider: Some(Collider {
                rect: Rect {
                    center: bed_collider.center,
                    half_size: bed_collider.half_size,
                },
            }),
            texture_id: Some(bed_tex),
            facing: Direction::Down,
        });

        entities.push(Entity {
            position: Vec2::new(112.0, 48.0) * multiplying_factor,
            size: Vec2::new(32.0, 64.0) * multiplying_factor,
            collider: Some(Collider {
                rect: Rect {
                    center: bed_collider.center,
                    half_size: bed_collider.half_size,
                },
            }),
            texture_id: Some(bed_tex),
            facing: Direction::Down,
        });

        let bed_prompt_tex = texture_store.load(device, queue, "assets/prompt.aseprite")?;
        entities.push(Entity {
            position: Vec2::new(94.0, 25.0) * multiplying_factor,
            size: Vec2::new(8.0, 8.0) * multiplying_factor,
            collider: None,
            texture_id: Some(bed_prompt_tex),
            facing: Direction::Down,
        });
        let bed_prompt = EntityId(entities.len() - 1);

        triggers.push(Trigger {
            rect: Rect {
                center: Vec2::new(94.0, 40.0) * multiplying_factor,
                half_size: Vec2::new(7.0, 8.0) * multiplying_factor,
            },
            recently_used: false,
            kind: TriggerKind::Interact {
                id: "bed_examine",
                prompt_entity: bed_prompt,
                prompt_texture: bed_prompt_tex,
                required_facing: &[Direction::Right],
            },
        });

        let nightstand_tex = texture_store.load(device, queue, "assets/nightstand.aseprite")?;
        entities.push(Entity {
            position: Vec2::new(64.0, 44.0) * multiplying_factor,
            size: Vec2::new(25.0, 16.0) * multiplying_factor,
            collider: Some(Collider {
                rect: Rect {
                    center: Vec2::new(0.0, 4.0) * multiplying_factor,
                    half_size: Vec2::new(12.5, 4.0) * multiplying_factor,
                },
            }),
            texture_id: Some(nightstand_tex),
            facing: Direction::Down,
        });

        let player_tex = texture_store.load(device, queue, "assets/player.aseprite")?;
        entities.push(Entity {
            position: Vec2::new(64.0, 87.5) * multiplying_factor,
            size: Vec2::new(14.0, 24.0) * multiplying_factor,
            collider: Some(Collider {
                rect: Rect {
                    center: Vec2::new(0.0, 6.0) * multiplying_factor,
                    half_size: Vec2::new(7.0, 6.0) * multiplying_factor,
                },
            }),
            texture_id: Some(player_tex),
            facing: Direction::Down,
        });
        let player_index = entities.len() - 1;

        let necklace_prompt_tex = texture_store.load(device, queue, "assets/prompt.aseprite")?;
        entities.push(Entity {
            position: Vec2::new(112.0, -5.0) * multiplying_factor,
            size: Vec2::new(8.0, 8.0) * multiplying_factor,
            collider: None,
            texture_id: Some(necklace_prompt_tex),
            facing: Direction::Down,
        });
        let necklace_prompt = EntityId(entities.len() - 1);

        let necklace_tex = texture_store.load(device, queue, "assets/necklace.aseprite")?;
        entities.push(Entity {
            position: Vec2::new(112.0, 10.0) * multiplying_factor,
            size: Vec2::new(20.0, 20.0) * multiplying_factor,
            collider: Some(Collider {
                rect: Rect {
                    center: Vec2::new(0.0, 4.0) * multiplying_factor,
                    half_size: Vec2::new(4.0, 8.0) * multiplying_factor,
                },
            }),
            texture_id: Some(necklace_tex),
            facing: Direction::Down,
        });

        triggers.push(Trigger {
            rect: Rect {
                center: Vec2::new(112.0, 10.0) * multiplying_factor,
                half_size: Vec2::new(12.0, 12.0) * multiplying_factor,
            },
            recently_used: false,
            kind: TriggerKind::Interact {
                id: "necklace_examine",
                prompt_entity: necklace_prompt,
                prompt_texture: necklace_prompt_tex,
                required_facing: &[Direction::Right],
            },
        });

        Ok(Scene {
            id: SceneId::Home,
            background,
            walls,
            triggers,
            entities,
            texture_store,
            player_index,
            camera_mode: CameraMode::Static(floor_position),
        })
    }

    pub fn new_outside(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        multiplying_factor: f32,
    ) -> Result<Self> {
        let mut texture_store = TextureStore::new();

        // Create Background
        let mut background = Vec::new();
        let village = texture_store.load(device, queue, "assets/ai_outside.png")?;
        let village_position = Vec2::new(64.0, 64.0) * multiplying_factor;
        let village_size = Vec2::new(128.0, 128.0) * multiplying_factor;
        background.push(Background {
            texture: village,
            position: village_position,
            size: village_size,
        });

        // Create Walls
        let wall_thickness = 8.0 * multiplying_factor;
        let village_half_width = village_size.x * 0.5;
        let village_half_height = village_size.y * 0.5;

        let left_edge = village_position.x - village_half_width;
        let right_edge = village_position.x + village_half_width;
        let top_edge = village_position.y - village_half_height;
        let bottom_edge = village_position.y + village_half_height;

        let walls = vec![
            Collider {
                rect: Rect {
                    center: Vec2::new(village_position.x, top_edge - wall_thickness),
                    half_size: Vec2::new(village_half_width, wall_thickness),
                },
            }, // north
            Collider {
                rect: Rect {
                    center: Vec2::new(village_position.x, bottom_edge + wall_thickness),
                    half_size: Vec2::new(village_half_width, wall_thickness),
                },
            }, // south
            Collider {
                rect: Rect {
                    center: Vec2::new(left_edge - wall_thickness, village_position.y),
                    half_size: Vec2::new(wall_thickness, village_half_height),
                },
            }, // west
            Collider {
                rect: Rect {
                    center: Vec2::new(right_edge + wall_thickness, village_position.y),
                    half_size: Vec2::new(wall_thickness, village_half_height),
                },
            }, // east
        ];

        // Create Triggers
        let mut triggers = Vec::new();

        let door_position = Vec2::new(64.0, 64.0) * multiplying_factor;
        let door_size = Vec2::new(16.0, 24.0) * multiplying_factor;

        triggers.push(Trigger {
            rect: Rect {
                center: Vec2::new(door_position.x, door_position.y),
                half_size: Vec2::new(door_size.x * 0.5, door_size.y * 0.5),
            },
            recently_used: false,
            kind: TriggerKind::Warp {
                warp_id: WarpId("door"),
                target_scene: SceneId::Home,
                target_warp_id: WarpId("door"),
                spawn_offset: Vec2::new(0.0, 20.0 * multiplying_factor), // down, into the patio
            },
        });

        // Create Entities
        let mut entities = Vec::new();

        let player_tex = texture_store.load(device, queue, "assets/player.aseprite")?;
        entities.push(Entity {
            position: Vec2::new(64.0, 87.5) * multiplying_factor,
            size: Vec2::new(14.0, 24.0) * multiplying_factor,
            collider: Some(Collider {
                rect: Rect {
                    center: Vec2::new(0.0, 6.0) * multiplying_factor,
                    half_size: Vec2::new(7.0, 6.0) * multiplying_factor,
                },
            }),
            texture_id: Some(player_tex),
            facing: Direction::Down,
        });
        let player_index = entities.len() - 1;

        Ok(Scene {
            id: SceneId::Outside,
            background,
            walls,
            triggers,
            entities,
            texture_store,
            player_index,
            camera_mode: CameraMode::Follow,
        })
    }
}
