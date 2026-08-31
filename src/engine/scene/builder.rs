use super::{Direction, Entity, EntityId, Rect, Scene, Trigger, TriggerKind};
use crate::engine::entity::Collider;
use crate::engine::grid::CELL_SIZE;
use crate::engine::renderer::tile;
use crate::engine::scene::TriggerId;
use anyhow::Result;
use glam::Vec2;

pub enum FootprintAnchor {
    BottomCenter, // walking characters - player, future NPCs
    BottomLeft,   // placed objects - furniture, blocks
}

pub struct EntitySpec {
    pub position: Vec2,
    pub render_size: Vec2,
    pub base_size: Vec2,
    pub collider_offset: Vec2,
    pub collider_size: Vec2,
    pub name: &'static str,
    pub facing: Direction,
    pub anchor: FootprintAnchor,
}

impl EntitySpec {
    pub fn new(
        position: Vec2,
        render_size: Vec2,
        base_size: Vec2,
        collider_offset: Vec2,
        collider_size: Vec2,
        name: &'static str,
        facing: Direction,
    ) -> Self {
        Self {
            position,
            render_size,
            base_size,
            collider_offset,
            collider_size,
            name,
            facing,
            anchor: FootprintAnchor::BottomLeft,
        }
    }
}

pub struct DialogueTriggerSpec {
    pub id: &'static str,
    pub target: EntityId,
    pub facing: &'static [Direction],
    pub prompt_texture_path: Option<&'static str>,
    pub consumes_entity: bool,
    pub sets_flag: Option<&'static str>,
}

pub struct DialogueTriggerResult {
    pub trigger: TriggerId,
    pub prompt_entity: Option<EntityId>,
}

impl Scene {
    pub fn texture_path(name: &str, is_isometric: bool) -> String {
        if is_isometric {
            format!("assets/isometric_{name}.aseprite")
        } else {
            format!("assets/{name}.aseprite")
        }
    }

    // The normal, name-based path every real spawn site already uses
    pub fn spawn_entity(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        multiplying_factor: f32,
        is_isometric: bool,
        is_active: bool,
        spec: EntitySpec,
    ) -> Result<EntityId> {
        let path = Self::texture_path(&spec.name, is_isometric);
        self.spawn_entity_with_path(
            device,
            queue,
            multiplying_factor,
            is_isometric,
            is_active,
            &path,
            spec,
        )
    }

    pub fn spawn_entity_with_path(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        multiplying_factor: f32,
        is_isometric: bool,
        is_active: bool,
        path: &str,
        spec: EntitySpec,
    ) -> Result<EntityId> {
        let texture = self.texture_store.load(device, queue, path);

        let texture = match texture {
            Ok(v) => v,
            Err(e) => panic!("Texture not found. Error: {}", e),
        };

        let texture_offset = if is_isometric {
            // For isometric, view. (2:1 shear applied to 16x16 square grid, to get 32x16 diamond)
            match spec.anchor {
                // Human shaped entities
                FootprintAnchor::BottomCenter => Vec2::new(
                    -spec.render_size.x + spec.base_size.x * 0.5,
                    -spec.render_size.y * 0.5,
                ),
                // Other Entities
                FootprintAnchor::BottomLeft => {
                    // Determine how much of the render height comes from Z.
                    let base_x = spec.base_size.x / CELL_SIZE;
                    let base_y = spec.base_size.y / CELL_SIZE;

                    let base_height = tile::isometric_footprint_base_height(base_x, base_y);
                    let padding = tile::isometric_footprint_padding(base_x, base_y);

                    let z_extra = spec.render_size.y - base_height * CELL_SIZE - padding;
                    let z_offset = Vec2::splat(-z_extra * 0.5);

                    Vec2::new(
                        (spec.base_size.x - CELL_SIZE) * 0.5,
                        -(spec.base_size.y + CELL_SIZE) * 0.5,
                    ) + z_offset
                }
            }
        } else {
            // For non isometric, 2D flat view. (square 16x16 grid, no shear applied)
            match spec.anchor {
                // Human shaped entities
                FootprintAnchor::BottomCenter => Vec2::new(0.0, -spec.render_size.y * 0.5),
                // Other entities
                // The collider fully occupies the texture
                FootprintAnchor::BottomLeft => {
                    Vec2::new(spec.base_size.x * 0.5, -spec.render_size.y * 0.5)
                }
            }
        };
        let collider_center = match spec.anchor {
            FootprintAnchor::BottomCenter => Vec2::new(0.0, -spec.collider_size.y * 0.5),
            FootprintAnchor::BottomLeft => {
                Vec2::new(spec.collider_size.x * 0.5, -spec.collider_size.y * 0.5)
            }
        };
        self.entities.push(Entity {
            position: spec.position,
            size: spec.render_size * multiplying_factor,
            texture_offset: texture_offset * multiplying_factor,
            collider: Some(Collider {
                rect: Rect {
                    center: spec.collider_offset * multiplying_factor
                        + collider_center * multiplying_factor,
                    half_size: spec.collider_size * 0.5 * multiplying_factor,
                },
            }),
            texture_id: Some(texture),
            facing: spec.facing,
            active: is_active,
            is_overlay_layer: false,
        });
        Ok(EntityId(self.entities.len() - 1))
    }

    pub fn spawn_human(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        multiplying_factor: f32,
        is_isometric: bool,
        is_active: bool,
        spec: EntitySpec,
    ) -> Result<EntityId> {
        self.spawn_entity(
            device,
            queue,
            multiplying_factor,
            is_isometric,
            is_active,
            spec,
        )
    }

    pub fn spawn_player(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        multiplying_factor: f32,
        is_isometric: bool,
        spec: EntitySpec,
    ) -> Result<()> {
        let player_id =
            self.spawn_human(device, queue, multiplying_factor, is_isometric, true, spec)?;
        self.player_index = player_id.0;
        Ok(())
    }

    pub fn spawn_dialogue_trigger(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        multiplying_factor: f32,
        spec: DialogueTriggerSpec,
    ) -> Result<DialogueTriggerResult> {
        let target_rect = self.entities[spec.target.0]
            .world_collider()
            .unwrap_or(Rect {
                center: self.entities[spec.target.0].position,
                half_size: Vec2::ZERO,
            });

        let trigger_half_size = target_rect.half_size + Vec2::ONE * multiplying_factor;
        let trigger_rect = Rect {
            center: target_rect.center,
            half_size: trigger_half_size,
        };

        const PROMPT_MARGIN: f32 = 15.0;

        let (prompt_entity, prompt_texture) = match spec.prompt_texture_path {
            Some(texture_path) => {
                let texture = self.texture_store.load(device, queue, texture_path)?;
                self.entities.push(Entity {
                    position: Vec2::new(
                        trigger_rect.center.x,
                        trigger_rect.center.y
                            - trigger_rect.half_size.y
                            - PROMPT_MARGIN * multiplying_factor,
                    ),
                    size: Vec2::new(8.0, 8.0) * multiplying_factor,
                    texture_offset: Vec2::new(CELL_SIZE, -CELL_SIZE * 2.0) * multiplying_factor,
                    collider: None,
                    texture_id: Some(texture),
                    facing: Direction::Down,
                    active: true,
                    is_overlay_layer: true,
                });
                (Some(EntityId(self.entities.len() - 1)), Some(texture))
            }
            None => (None, None),
        };

        self.triggers.push(Trigger::new(
            trigger_rect,
            TriggerKind::Dialogue {
                id: spec.id,
                prompt_entity,
                prompt_texture,
                facing: spec.facing,
                consumes_entity: if spec.consumes_entity {
                    Some(spec.target)
                } else {
                    None
                },
                sets_flag: spec.sets_flag,
            },
        ));
        let trigger = TriggerId(self.triggers.len() - 1);

        Ok(DialogueTriggerResult {
            trigger,
            prompt_entity,
        })
    }
}
