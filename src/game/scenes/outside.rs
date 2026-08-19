use crate::engine::entity::{Collider, Direction, Entity, EntityId, Rect};
use crate::engine::renderer::texture::TextureStore;
use crate::engine::scene::{Background, CameraMode, Scene, SceneId, Trigger, TriggerKind, WarpId};
use anyhow::Result;
use glam::Vec2;

pub fn build(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    multiplying_factor: f32,
    is_isometric: bool,
) -> Result<Scene> {
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

    let patio_door_position = Vec2::new(108.0, 130.0) * multiplying_factor;
    let patio_door_half_width = 12.0 * multiplying_factor;
    let patio_door_half_height = 6.0 * multiplying_factor;
    let patio_door_left_edge = patio_door_position.x - patio_door_half_width;
    let patio_door_right_edge = patio_door_position.x + patio_door_half_width;
    // let door_top_edge = door_position.y - door_half_height;
    // let door_bottom_edge = door_position.y + door_half_height;

    let village_left_edge = village_position.x - village_half_width;
    let village_right_edge = village_position.x + village_half_width;
    let village_top_edge = village_position.y - village_half_height;
    let village_bottom_edge = village_position.y + village_half_height;

    let walls = vec![
        Collider {
            rect: Rect {
                center: Vec2::new(village_position.x, village_top_edge - wall_thickness),
                half_size: Vec2::new(village_half_width, wall_thickness),
            },
        }, // north
        Collider {
            rect: Rect {
                center: Vec2::new(
                    village_left_edge + (patio_door_left_edge - village_left_edge) * 0.5,
                    village_bottom_edge + wall_thickness,
                ),
                half_size: Vec2::new(
                    (patio_door_left_edge - village_left_edge) * 0.5,
                    wall_thickness,
                ),
            },
        }, // south-west
        Collider {
            rect: Rect {
                center: Vec2::new(
                    village_right_edge - (village_right_edge - patio_door_right_edge) * 0.5,
                    village_bottom_edge + wall_thickness,
                ),
                half_size: Vec2::new(
                    (village_right_edge - patio_door_right_edge) * 0.5,
                    wall_thickness,
                ),
            },
        }, // south-east
        Collider {
            rect: Rect {
                center: Vec2::new(village_left_edge - wall_thickness, village_position.y),
                half_size: Vec2::new(wall_thickness, village_half_height),
            },
        }, // west
        Collider {
            rect: Rect {
                center: Vec2::new(village_right_edge + wall_thickness, village_position.y),
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
            center: door_position,
            half_size: door_size * 0.5,
        },
        recently_used: false,
        kind: TriggerKind::Warp {
            warp_id: WarpId("door"),
            target_scene: SceneId::Home,
            target_warp_id: WarpId("door"),
            spawn_offset: Vec2::new(0.0, 20.0 * multiplying_factor), // down, into the patio
        },
        active: true,
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

    let patio_door_closed_tex = texture_store.load(device, queue, "assets/patio_door.aseprite")?;
    let patio_door_open_tex =
        texture_store.load_aseprite_frame(device, queue, "assets/patio_door.aseprite", 1)?;
    let closed_collider = Rect {
        center: Vec2::new(0.0, -4.0 * multiplying_factor),
        half_size: Vec2::new(patio_door_half_width, 2.0 * multiplying_factor),
    };
    entities.push(Entity {
        position: patio_door_position,
        size: Vec2::new(patio_door_half_width * 2.0, patio_door_half_height * 2.0),
        collider: Some(Collider {
            rect: closed_collider,
        }),
        texture_id: Some(patio_door_closed_tex),
        facing: Direction::Down,
    });
    let patio_door_entity = EntityId(entities.len() - 1);

    // Two separate triggers, one per approach side — NOT one trigger
    // with both Up and Down listed in required_facing. A single shared
    // rect can't distinguish "standing above, facing down (correct)"
    // from "standing below, facing down (facing away, wrong)" — see
    // ADR-060, first discovered on the necklace's two-sided approach,
    // now repurposed here for the same reason.
    let patio_door_top_toggle_center = Vec2::new(patio_door_position.x, 120.0 * multiplying_factor);
    let patio_door_top_toggle_half_height = 4.0 * multiplying_factor;
    let patio_door_bottom_toggle_center =
        Vec2::new(patio_door_position.x, 136.0 * multiplying_factor);
    let patio_door_bottom_toggle_half_height = 8.0 * multiplying_factor;

    triggers.push(Trigger {
        rect: Rect {
            center: patio_door_top_toggle_center,
            half_size: Vec2::new(patio_door_half_width, patio_door_top_toggle_half_height),
        },
        recently_used: false,
        kind: TriggerKind::Toggle {
            target_entity: patio_door_entity,
            closed_texture: patio_door_closed_tex,
            open_texture: patio_door_open_tex,
            closed_collider,
            required_facing: &[Direction::Down],
        },
        active: true,
    });

    triggers.push(Trigger {
        rect: Rect {
            center: patio_door_bottom_toggle_center,
            half_size: Vec2::new(patio_door_half_width, patio_door_bottom_toggle_half_height),
        },
        recently_used: false,
        kind: TriggerKind::Toggle {
            target_entity: patio_door_entity,
            closed_texture: patio_door_closed_tex,
            open_texture: patio_door_open_tex,
            closed_collider,
            required_facing: &[Direction::Up],
        },
        active: true,
    });

    Ok(Scene::new(
        SceneId::Outside,
        background,
        walls,
        triggers,
        entities,
        texture_store,
        player_index,
        CameraMode::Follow,
        CameraMode::Follow,
        is_isometric,
    ))
}
