use crate::engine::entity::{Collider, Direction, Entity, EntityId, Rect};
use crate::engine::grid::CELL_SIZE;
use crate::engine::renderer::texture::{TextureId, TextureStore};
use crate::engine::scene::{
    Background, CameraMode, Scene, SceneId, Trigger, TriggerKind, WarpId, builder,
};
use crate::game::progression::ProgressionTracker;
use anyhow::Result;
use glam::Vec2;

pub fn build(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    multiplying_factor: f32,
    is_isometric: bool,
    _progression: &ProgressionTracker,
) -> Result<Scene> {
    // Establish positioning
    let village_position = Vec2::new(64.0, 64.0) * multiplying_factor;
    let village_size = Vec2::new(128.0, 128.0) * multiplying_factor;
    let patio_door_position = Vec2::new(108.0, 130.0) * multiplying_factor;
    let patio_door_half_width = 12.0 * multiplying_factor;
    let patio_door_half_height = 6.0 * multiplying_factor;

    let mut scene = Scene::new(
        SceneId::Village,
        TextureStore::new(),
        CameraMode::Follow,
        CameraMode::Follow,
        is_isometric,
        multiplying_factor,
    );

    build_background(&mut scene, device, queue, village_position, village_size)?;
    build_walls(
        &mut scene,
        multiplying_factor,
        village_position,
        village_size,
        patio_door_position,
        patio_door_half_width,
    );
    build_warp_trigger(&mut scene, multiplying_factor);

    let village_entities = build_entities(
        &mut scene,
        device,
        queue,
        multiplying_factor,
        is_isometric,
        patio_door_position,
        patio_door_half_width,
        patio_door_half_height,
    )?;

    build_triggers(
        &mut scene,
        device,
        queue,
        multiplying_factor,
        patio_door_position,
        patio_door_half_width,
        patio_door_half_height,
        &village_entities,
    )?;

    Ok(scene)
}

pub fn refresh(
    scene: &mut Scene,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    multiplying_factor: f32,
    is_isometric: bool,
    progression: &ProgressionTracker,
) -> Result<Scene> {
    let player_position = scene.player().position;
    let refreshed_scene = build(device, queue, multiplying_factor, is_isometric, progression);

    let mut refreshed_scene = match refreshed_scene {
        Ok(scene) => scene,
        Err(err) => return Err(err),
    };

    refreshed_scene.player_mut().position = player_position;

    Ok(refreshed_scene)
}

fn build_background(
    scene: &mut Scene,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    village_position: Vec2,
    village_size: Vec2,
) -> Result<()> {
    let village_texture = scene
        .texture_store
        .load(device, queue, "assets/ai_village.png")?;
    scene.background.push(Background {
        texture: village_texture,
        position: village_position,
        size: village_size,
    });
    Ok(())
}

fn build_walls(
    scene: &mut Scene,
    multiplying_factor: f32,
    village_position: Vec2,
    village_size: Vec2,
    patio_door_position: Vec2,
    patio_door_half_width: f32,
) {
    let wall_thickness = 8.0 * multiplying_factor;
    let village_half_width = village_size.x * 0.5;
    let village_half_height = village_size.y * 0.5;
    let patio_door_left_edge = patio_door_position.x - patio_door_half_width;
    let patio_door_right_edge = patio_door_position.x + patio_door_half_width;
    let village_left_edge = village_position.x - village_half_width;
    let village_right_edge = village_position.x + village_half_width;
    let village_top_edge = village_position.y - village_half_height;
    let village_bottom_edge = village_position.y + village_half_height;

    scene.walls.extend(vec![
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
    ]);
}

fn build_warp_trigger(scene: &mut Scene, multiplying_factor: f32) {
    let door_position = Vec2::new(64.0, 64.0) * multiplying_factor;
    let door_size = Vec2::new(16.0, 24.0) * multiplying_factor;
    scene.triggers.push(Trigger::new(
        Rect {
            center: door_position,
            half_size: door_size * 0.5,
        },
        TriggerKind::Warp {
            warp_id: WarpId("door"),
            target_scene: SceneId::Home,
            target_warp_id: WarpId("door"),
            spawn_offset: Vec2::new(0.0, 20.0 * multiplying_factor),
        },
    ));
}

/// IDs and data the village's triggers need once entities exist -
/// patio_door's Toggle trigger references its entity/textures/closed
/// collider; villager_1's dialogue trigger just needs its EntityId.
struct VillageEntities {
    patio_door_entity: EntityId,
    patio_door_closed_tex: TextureId,
    patio_door_open_tex: TextureId,
    patio_door_closed_collider: Rect,
    villager_1_id: EntityId,
}

fn build_entities(
    scene: &mut Scene,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    multiplying_factor: f32,
    is_isometric: bool,
    patio_door_position: Vec2,
    patio_door_half_width: f32,
    patio_door_half_height: f32,
) -> Result<VillageEntities> {
    let player_collider_offset = if is_isometric {
        Vec2::new(0.0, 0.0)
    } else {
        Vec2::new(0.0, 0.0)
    };
    let player_collider_size = if is_isometric {
        Vec2::new(6.0, 6.0)
    } else {
        Vec2::new(14.0, 6.0)
    };
    scene.spawn_player(
        device,
        queue,
        multiplying_factor,
        is_isometric,
        builder::EntitySpec {
            position: Vec2::new(64.0, 87.0),
            render_size: Vec2::new(16.0, 32.0),
            base_size: player_collider_size,
            collider_offset: player_collider_offset,
            collider_size: player_collider_size,
            name: "player",
            facing: Direction::Down,
            anchor: builder::FootprintAnchor::BottomCenter,
        },
    )?;

    // TODO: patio door still bypasses spawn_entity/FootprintAnchor entirely
    // Hand-authored, not spawn_entity: needs two texture frames
    // (closed/open) and a negative-Y collider offset - spawn_entity's
    // single-texture-path signature doesn't cover this case yet.
    let patio_door_closed_tex =
        scene
            .texture_store
            .load(device, queue, "assets/patio_door.aseprite")?;
    let patio_door_open_tex =
        scene
            .texture_store
            .load_aseprite_frame(device, queue, "assets/patio_door.aseprite", 1)?;
    let patio_door_closed_collider = Rect {
        center: Vec2::new(0.0, -4.0 * multiplying_factor),
        half_size: Vec2::new(patio_door_half_width, 2.0 * multiplying_factor),
    };
    scene.entities.push(Entity {
        position: patio_door_position,
        texture_offset: Vec2::new(CELL_SIZE, -CELL_SIZE * 2.0) * multiplying_factor,
        size: Vec2::new(patio_door_half_width * 2.0, patio_door_half_height * 2.0),
        collider: Some(Collider {
            rect: patio_door_closed_collider,
        }),
        texture_id: Some(patio_door_closed_tex),
        facing: Direction::Down,
        active: true,
        is_overlay_layer: false,
    });
    let patio_door_entity = EntityId(scene.entities.len() - 1);

    let villager_1_id = scene.spawn_human(
        device,
        queue,
        multiplying_factor,
        is_isometric,
        true,
        builder::EntitySpec::new(
            Vec2::new(150.0, 150.0),
            Vec2::new(12.0, 24.0),
            Vec2::new(6.0, 6.0),
            Vec2::ZERO,
            Vec2::new(6.0, 6.0),
            "villager_1",
            Direction::Left,
        ),
    )?;

    Ok(VillageEntities {
        patio_door_entity,
        patio_door_closed_tex,
        patio_door_open_tex,
        patio_door_closed_collider,
        villager_1_id,
    })
}

fn build_triggers(
    scene: &mut Scene,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    multiplying_factor: f32,
    patio_door_position: Vec2,
    patio_door_half_width: f32,
    patio_door_half_height: f32,
    entities: &VillageEntities,
) -> Result<()> {
    scene.triggers.push(Trigger::new(
        Rect {
            center: patio_door_position,
            half_size: Vec2::new(
                patio_door_half_width,
                patio_door_half_height + 6.0 * multiplying_factor,
            ),
        },
        TriggerKind::Toggle {
            target_entity: entities.patio_door_entity,
            closed_texture: entities.patio_door_closed_tex,
            open_texture: entities.patio_door_open_tex,
            closed_collider: entities.patio_door_closed_collider,
            facing: &[Direction::Up, Direction::Down],
        },
    ));

    scene.spawn_dialogue_trigger(
        device,
        queue,
        multiplying_factor,
        builder::DialogueTriggerSpec {
            id: "villager_1_interact",
            target: entities.villager_1_id,
            facing: &[
                Direction::Up,
                Direction::Down,
                Direction::Left,
                Direction::Right,
            ],
            prompt_texture_path: Some("assets/prompt.aseprite"),
            consumes_entity: false,
            sets_flag: None,
        },
    )?;
    Ok(())
}
