use super::{Direction, Entity, EntityId, Rect, Scene, Trigger, TriggerKind};
use crate::engine::entity::Collider;
use anyhow::Result;
use glam::Vec2;

pub struct EntitySpec {
    pub position: Vec2,
    pub size: Vec2,
    pub collider_offset: Vec2,
    pub collider_size: Vec2,
    pub texture_path: &'static str,
    pub facing: Direction,
}

pub struct DialogueTriggerSpec {
    pub id: &'static str,
    pub target: EntityId,
    pub facing: &'static [Direction],
    pub prompt_texture_path: Option<&'static str>,
    pub consumes_entity: bool,
    pub sets_flag: Option<&'static str>,
}

impl Scene {
    pub fn spawn_entity(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        multiplying_factor: f32,
        spec: EntitySpec,
    ) -> Result<EntityId> {
        let texture = self.texture_store.load(device, queue, spec.texture_path)?;
        self.entities.push(Entity {
            position: spec.position * multiplying_factor,
            size: spec.size * multiplying_factor,
            collider: Some(Collider {
                rect: Rect {
                    center: spec.collider_offset * multiplying_factor,
                    half_size: spec.collider_size * 0.5 * multiplying_factor,
                },
            }),
            texture_id: Some(texture),
            facing: spec.facing,
            active: true,
        });
        Ok(EntityId(self.entities.len() - 1))
    }

    pub fn spawn_player(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        multiplying_factor: f32,
        spec: EntitySpec,
    ) -> Result<()> {
        let player_id = self.spawn_entity(device, queue, multiplying_factor, spec)?;
        self.player_index = player_id.0;
        Ok(())
    }

    pub fn spawn_dialogue_trigger(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        multiplying_factor: f32,
        spec: DialogueTriggerSpec,
    ) -> Result<()> {
        let target_position = self.entities[spec.target.0].position;
        let target_collider_half_size = self.entities[spec.target.0]
            .collider
            .as_ref()
            .map(|c| c.rect.half_size)
            .unwrap_or(Vec2::ZERO);

        let trigger_half_size = target_collider_half_size + Vec2::ONE * multiplying_factor;
        let trigger_rect = Rect {
            center: target_position,
            half_size: trigger_half_size,
        };

        const PROMPT_MARGIN: f32 = 5.0;

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
                    collider: None,
                    texture_id: Some(texture),
                    facing: Direction::Down,
                    active: true,
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
        Ok(())
    }
}
