use anyhow::Result;

use crate::engine::entity::{Entity, Rect, TextureId, aabb_overlap};
use crate::engine::texture::TextureStore;

pub struct Scene {
    pub background: TextureId,
    pub background_position: glam::Vec2,
    pub background_size: glam::Vec2,
    pub walls: Vec<Rect>,
    pub entities: Vec<Entity>,
    pub texture_store: TextureStore,
    pub player_index: usize,
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
        world_center: glam::Vec2,
        half_size: glam::Vec2,
        skip_index: usize,
    ) -> bool {
        for wall in &self.walls {
            if aabb_overlap(world_center, half_size, wall.offset, wall.half_size) {
                return true;
            }
        }

        for (i, entity) in self.entities.iter().enumerate() {
            if i == skip_index {
                continue;
            }
            if let Some(collider) = &entity.collider {
                let other_center = entity.position + collider.offset;
                if aabb_overlap(world_center, half_size, other_center, collider.half_size) {
                    return true;
                }
            }
        }

        false
    }

    /// Attempts to move the player by `delta`. Resolves collisions
    /// sequentially, per axis — x first, then y from the (possibly
    /// already-updated) x — which is what produces sliding along a
    /// wall or furniture edge on diagonal movement, per the M3
    /// collision design.
    pub fn try_move_player(&mut self, delta: glam::Vec2) {
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
        let (offset, half_size) = (collider.offset, collider.half_size);

        // X axis
        let proposed = glam::Vec2::new(
            self.entities[idx].position.x + delta.x,
            self.entities[idx].position.y,
        );
        let world_center = proposed + offset;
        if !self.collider_blocked(world_center, half_size, idx) {
            self.entities[idx].position.x = proposed.x;
        }

        // Y axis, from whatever x just resulted from the check above.
        let proposed = glam::Vec2::new(
            self.entities[idx].position.x,
            self.entities[idx].position.y + delta.y,
        );
        let world_center = proposed + offset;
        if !self.collider_blocked(world_center, half_size, idx) {
            self.entities[idx].position.y = proposed.y;
        }
    }

    /// Beat 1's bedroom: a sealed rectangular room (no working exit yet —
    /// the door is a solid placeholder until M4 builds scene transitions).
    /// All sizes are at multiplying_factor = 1.0 — pure layout/collision
    /// verification content, not final visual scale (see M3 session notes).
    pub fn new_bedroom(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        multiplying_factor: f32,
    ) -> Result<Self> {
        let mut texture_store = TextureStore::new();

        let background = texture_store.load(device, queue, "assets/bedroom.png")?;
        let background_position = glam::Vec2::new(64.0, 64.0) * multiplying_factor;
        let background_size = glam::Vec2::new(128.0, 128.0) * multiplying_factor;

        let walls = vec![
            Rect {
                offset: glam::Vec2::new(64.0, -4.0) * multiplying_factor,
                half_size: glam::Vec2::new(64.0, 8.0) * multiplying_factor,
            }, // north
            Rect {
                offset: glam::Vec2::new(64.0, 132.0) * multiplying_factor,
                half_size: glam::Vec2::new(64.0, 8.0) * multiplying_factor,
            }, // south
            Rect {
                offset: glam::Vec2::new(-8.0, 64.0) * multiplying_factor,
                half_size: glam::Vec2::new(8.0, 64.0) * multiplying_factor,
            }, // west
            Rect {
                offset: glam::Vec2::new(132.0, 64.0) * multiplying_factor,
                half_size: glam::Vec2::new(8.0, 64.0) * multiplying_factor,
            }, // east
        ];

        let mut entities: Vec<Entity> = Vec::new();

        let wardrobe_tex = texture_store.load(device, queue, "assets/wardrobe.png")?;
        entities.push(Entity {
            position: glam::Vec2::new(16.0, 12.0) * multiplying_factor,
            size: glam::Vec2::new(24.0, 40.0) * multiplying_factor,
            collider: Some(Rect {
                offset: glam::Vec2::new(0.0, 0.0) * multiplying_factor,
                half_size: glam::Vec2::new(12.0, 20.0) * multiplying_factor,
            }),
            texture_id: Some(wardrobe_tex),
        });

        let bed_tex = texture_store.load(device, queue, "assets/bed.png")?;
        entities.push(Entity {
            position: glam::Vec2::new(16.0, 48.0) * multiplying_factor,
            size: glam::Vec2::new(32.0, 64.0) * multiplying_factor,
            collider: Some(Rect {
                offset: glam::Vec2::new(0.0, 20.0) * multiplying_factor,
                half_size: glam::Vec2::new(16.0, 4.0) * multiplying_factor,
            }),
            texture_id: Some(bed_tex),
        });

        entities.push(Entity {
            position: glam::Vec2::new(112.0, 48.0) * multiplying_factor,
            size: glam::Vec2::new(32.0, 64.0) * multiplying_factor,
            collider: Some(Rect {
                offset: glam::Vec2::new(0.0, 20.0) * multiplying_factor,
                half_size: glam::Vec2::new(16.0, 4.0) * multiplying_factor,
            }),
            texture_id: Some(bed_tex),
        });

        let nightstand_tex = texture_store.load(device, queue, "assets/nightstand.png")?;
        entities.push(Entity {
            position: glam::Vec2::new(64.0, 44.0) * multiplying_factor,
            size: glam::Vec2::new(25.0, 16.0) * multiplying_factor,
            collider: Some(Rect {
                offset: glam::Vec2::new(0.0, 6.0) * multiplying_factor,
                half_size: glam::Vec2::new(12.5, 2.0) * multiplying_factor,
            }),
            texture_id: Some(nightstand_tex),
        });

        let door_tex = texture_store.load(device, queue, "assets/door.png")?;
        entities.push(Entity {
            position: glam::Vec2::new(64.0, 128.0) * multiplying_factor,
            size: glam::Vec2::new(32.0, 16.0) * multiplying_factor,
            collider: Some(Rect {
                offset: glam::Vec2::new(0.0, 0.0) * multiplying_factor,
                half_size: glam::Vec2::new(32.0, 16.0) * multiplying_factor,
            }),
            texture_id: Some(door_tex),
        });

        let player_tex = texture_store.load(device, queue, "assets/player.png")?;
        entities.push(Entity {
            position: glam::Vec2::new(64.0, 87.5) * multiplying_factor,
            size: glam::Vec2::new(14.0, 24.0) * multiplying_factor,
            collider: Some(Rect {
                offset: glam::Vec2::new(0.0, 6.0) * multiplying_factor,
                half_size: glam::Vec2::new(7.0, 6.0) * multiplying_factor,
            }),
            texture_id: Some(player_tex),
        });
        let player_index = entities.len() - 1;

        Ok(Scene {
            background,
            background_position,
            background_size,
            walls,
            entities,
            texture_store,
            player_index,
        })
    }
}
