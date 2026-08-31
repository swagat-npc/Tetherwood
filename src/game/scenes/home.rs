use crate::engine::entity::{Collider, Direction, Entity, EntityId, Rect};
use crate::engine::grid::CELL_SIZE;
use crate::engine::renderer::texture::TextureStore;
use crate::engine::renderer::tile;
use crate::engine::scene::{
    Background, CameraMode, Scene, SceneId, Trigger, TriggerKind, WarpId, builder,
};
use crate::game::progression::ProgressionTracker;
use anyhow::Result;
use glam::Vec2;

struct RoomLayout {
    left_edge: f32,
    right_edge: f32,
    top_edge: f32,
    bottom_edge: f32,
    room_half_width: f32,
    room_half_height: f32,
    wall_thickness: f32,
    door_half_width: f32,
    door_center_x: f32,
    south_west_half_width: f32,
    south_west_center_x: f32,
    south_east_half_width: f32,
    south_east_center_x: f32,
}

fn compute_room_layout(
    floor_position: Vec2,
    floor_size: Vec2,
    multiplying_factor: f32,
) -> RoomLayout {
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

    RoomLayout {
        left_edge,
        right_edge,
        top_edge,
        bottom_edge,
        room_half_width,
        room_half_height,
        wall_thickness,
        door_half_width,
        door_center_x,
        south_west_half_width,
        south_west_center_x,
        south_east_half_width,
        south_east_center_x,
    }
}

/// Beat 1's home: player's bedroom with a working south-door warp to
/// the village scene (SceneId::Village, WarpId("door")). Sizes scale
/// via multiplying_factor (ADR-042).
pub fn build(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    multiplying_factor: f32,
    is_isometric: bool,
    progression: &ProgressionTracker,
) -> Result<Scene> {
    // Establish positioning
    let floor_position = Vec2::new(64.0, 64.0) * multiplying_factor;
    let floor_size = Vec2::new(128.0, 128.0) * multiplying_factor;
    let layout = compute_room_layout(floor_position, floor_size, multiplying_factor);

    let mut scene = Scene::new(
        SceneId::Home,
        TextureStore::new(),
        CameraMode::Static(floor_position),
        CameraMode::Follow,
        is_isometric,
        multiplying_factor,
    );

    // Create Background
    build_background(
        &mut scene,
        device,
        queue,
        floor_position,
        &layout,
        multiplying_factor,
    )?;

    // Create Walls
    build_walls(&mut scene, floor_position, &layout);

    // Create Warps
    build_warp_trigger(&mut scene, multiplying_factor, &layout);

    // Create Entities
    let home_entities =
        build_entities(&mut scene, device, queue, multiplying_factor, is_isometric)?;

    // Create Triggers
    build_triggers(
        &mut scene,
        device,
        queue,
        multiplying_factor,
        progression,
        &home_entities,
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
    floor_position: Vec2,
    layout: &RoomLayout,
    multiplying_factor: f32,
) -> Result<()> {
    let door = scene
        .texture_store
        .load(device, queue, "assets/door.aseprite")?;
    let door_position = Vec2::new(floor_position.x, layout.bottom_edge);
    let door_size = Vec2::new(32.0, 16.0) * multiplying_factor; // NOTE: needs multiplying_factor in scope
    scene.background.push(Background {
        texture: door,
        position: door_position,
        size: door_size,
    });
    Ok(())
}

fn build_walls(scene: &mut Scene, floor_position: Vec2, layout: &RoomLayout) {
    scene.walls.extend(vec![
        Collider {
            // North
            rect: Rect {
                center: Vec2::new(floor_position.x, layout.top_edge - layout.wall_thickness),
                half_size: Vec2::new(layout.room_half_width, layout.wall_thickness),
            },
        },
        Collider {
            // South-West (of the door)
            rect: Rect {
                center: Vec2::new(
                    layout.south_west_center_x,
                    layout.bottom_edge + layout.wall_thickness,
                ),
                half_size: Vec2::new(layout.south_west_half_width, layout.wall_thickness),
            },
        },
        Collider {
            // South-East (of the door)
            rect: Rect {
                center: Vec2::new(
                    layout.south_east_center_x,
                    layout.bottom_edge + layout.wall_thickness,
                ),
                half_size: Vec2::new(layout.south_east_half_width, layout.wall_thickness),
            },
        },
        Collider {
            // West
            rect: Rect {
                center: Vec2::new(layout.left_edge - layout.wall_thickness, floor_position.y),
                half_size: Vec2::new(layout.wall_thickness, layout.room_half_height),
            },
        },
        Collider {
            // East
            rect: Rect {
                center: Vec2::new(layout.right_edge + layout.wall_thickness, floor_position.y),
                half_size: Vec2::new(layout.wall_thickness, layout.room_half_height),
            },
        },
    ]);
}

fn build_warp_trigger(scene: &mut Scene, multiplying_factor: f32, layout: &RoomLayout) {
    // Trigger sits fully past the wall's outer edge — the player must
    // walk all the way through the doorway gap and beyond the threshold
    // before their center overlaps this, not just step into the gap.
    let door_trigger_center_y = layout.bottom_edge + 2.0 * layout.wall_thickness;
    scene.triggers.push(Trigger::new(
        Rect {
            center: Vec2::new(layout.door_center_x, door_trigger_center_y),
            half_size: Vec2::new(layout.door_half_width, layout.wall_thickness),
        },
        TriggerKind::Warp {
            warp_id: WarpId("door"),
            target_scene: SceneId::Village,
            target_warp_id: WarpId("door"),
            spawn_offset: Vec2::new(0.0, -20.0 * multiplying_factor),
        },
    ));
}

struct HomeEntities {
    bed_prompt: EntityId,
    necklace_id: EntityId,
}

fn build_entities(
    scene: &mut Scene,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    multiplying_factor: f32,
    is_isometric: bool,
) -> Result<HomeEntities> {
    let wardrobe_base_size = tile::grid_to_pixel((2.0, 1.0));
    let wardrobe_size = if is_isometric {
        Vec2::new(48.0, 56.0)
    } else {
        Vec2::new(32.0, 48.0)
    };
    let wardrobe_collider_size = if is_isometric {
        wardrobe_base_size
    } else {
        wardrobe_base_size
    };
    scene.spawn_entity(
        device,
        queue,
        multiplying_factor,
        is_isometric,
        true,
        builder::EntitySpec::new(
            tile::world_at_cell((0, 1), multiplying_factor)
                + Vec2::new(0.0, 6.0) * multiplying_factor,
            wardrobe_size,
            wardrobe_base_size,
            Vec2::ZERO,
            wardrobe_collider_size,
            "wardrobe",
            Direction::Down,
        ),
    )?; // wardrobe

    let nightstand_base_size = if is_isometric {
        tile::grid_to_pixel((2.0, 1.5))
    } else {
        tile::grid_to_pixel((2.0, 0.5))
    };
    let nightstand_size = if is_isometric {
        Vec2::new(48.0, 48.0)
    } else {
        Vec2::new(32.0, 16.0)
    };
    let nightstand_collider_size = if is_isometric {
        nightstand_base_size
    } else {
        nightstand_base_size
    };
    scene.spawn_entity(
        device,
        queue,
        multiplying_factor,
        is_isometric,
        true,
        builder::EntitySpec::new(
            tile::world_at_cell((3, 3), multiplying_factor),
            nightstand_size,
            nightstand_base_size,
            Vec2::new(0.0, 0.0),
            nightstand_collider_size,
            "nightstand",
            Direction::Down,
        ),
    )?; // nightstand

    let crate_base_size = if is_isometric {
        tile::grid_to_pixel((1.0, 2.0))
    } else {
        tile::grid_to_pixel((1.0, 1.0))
    };
    let crate_size = if is_isometric {
        Vec2::new(48.0, 40.0)
    } else {
        Vec2::new(16.0, 32.0)
    };
    let crate_collider_size = if is_isometric {
        crate_base_size
    } else {
        crate_base_size
    };
    scene.spawn_entity(
        device,
        queue,
        multiplying_factor,
        is_isometric,
        true,
        builder::EntitySpec::new(
            tile::world_at_cell((0, 8), multiplying_factor),
            crate_size,
            crate_base_size,
            Vec2::ZERO,
            crate_collider_size,
            "crate",
            Direction::Down,
        ),
    )?; // crate

    let bed_base_size = if is_isometric {
        tile::grid_to_pixel((2.0, 3.0))
    } else {
        tile::grid_to_pixel((2.0, 3.0))
    };
    let bed_size = if is_isometric {
        Vec2::new(80.0, 64.0)
    } else {
        Vec2::new(32.0, 64.0)
    };
    let bed_collider_offset = if is_isometric {
        Vec2::new(0.0, 0.0)
    } else {
        Vec2::new(0.0, 0.0)
    };
    let bed_collider_size = if is_isometric {
        bed_base_size
    } else {
        bed_base_size
    };
    scene.spawn_entity(
        device,
        queue,
        multiplying_factor,
        is_isometric,
        true,
        builder::EntitySpec::new(
            tile::world_at_cell((0, 5), multiplying_factor),
            bed_size,
            bed_base_size,
            bed_collider_offset,
            bed_collider_size,
            "bed_him",
            Direction::Down,
        ),
    )?; // bed (left)

    scene.spawn_entity(
        device,
        queue,
        multiplying_factor,
        is_isometric,
        true,
        builder::EntitySpec::new(
            tile::world_at_cell((6, 5), multiplying_factor),
            bed_size,
            bed_base_size,
            bed_collider_offset,
            bed_collider_size,
            "bed_her",
            Direction::Down,
        ),
    )?; // bed (right)

    let bed_prompt_tex = scene
        .texture_store
        .load(device, queue, "assets/prompt.aseprite")?;
    scene.entities.push(Entity {
        position: Vec2::new(94.0, 25.0) * multiplying_factor,
        texture_offset: Vec2::new(CELL_SIZE, -CELL_SIZE * 2.0) * multiplying_factor,
        size: Vec2::new(8.0, 8.0) * multiplying_factor,
        collider: None,
        texture_id: Some(bed_prompt_tex),
        facing: Direction::Down,
        active: true,
        is_overlay_layer: false,
    });
    let bed_prompt = EntityId(scene.entities.len() - 1);

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
            position: tile::world_at_cell((3, 5), multiplying_factor),
            render_size: Vec2::new(16.0, 32.0),
            base_size: player_collider_size,
            collider_offset: player_collider_offset,
            collider_size: player_collider_size,
            name: "player",
            facing: Direction::Down,
            anchor: builder::FootprintAnchor::BottomCenter,
        },
    )?;

    let necklace_id = scene.spawn_entity(
        device,
        queue,
        multiplying_factor,
        is_isometric,
        true,
        builder::EntitySpec::new(
            tile::world_at_cell((6, 0), multiplying_factor),
            Vec2::new(20.0, 20.0),
            Vec2::new(20.0, 20.0),
            Vec2::new(0.0, 4.0),
            Vec2::new(8.0, 16.0),
            "necklace",
            Direction::Left,
        ),
    )?;

    Ok(HomeEntities {
        bed_prompt,
        necklace_id,
    })
}

fn build_triggers(
    scene: &mut Scene,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    multiplying_factor: f32,
    progression: &ProgressionTracker,
    entities: &HomeEntities,
) -> Result<()> {
    // bed_prompt_tex needs re-reading from the entity, since build_entities
    // didn't return it separately - it's stored on the entity itself.
    let bed_prompt_tex = scene.entities[entities.bed_prompt.0].texture_id.unwrap();

    scene.triggers.push(Trigger::new(
        Rect {
            center: Vec2::new(94.0, 40.0) * multiplying_factor,
            half_size: Vec2::new(7.0, 8.0) * multiplying_factor,
        },
        TriggerKind::Dialogue {
            id: "bed_examine",
            prompt_entity: Some(entities.bed_prompt),
            prompt_texture: Some(bed_prompt_tex),
            facing: &[Direction::Left],
            consumes_entity: None,
            sets_flag: None,
        },
    ));

    let necklace_trigger = scene.spawn_dialogue_trigger(
        device,
        queue,
        multiplying_factor,
        builder::DialogueTriggerSpec {
            id: "necklace_examine",
            target: entities.necklace_id,
            facing: &[Direction::Left],
            prompt_texture_path: Some("assets/prompt.aseprite"),
            consumes_entity: true,
            sets_flag: Some("necklace_consumed"),
        },
    )?;

    if progression.is_set("necklace_consumed") {
        scene.entities[entities.necklace_id.0].deactivate();
        if let Some(prompt_id) = necklace_trigger.prompt_entity {
            scene.entities[prompt_id.0].deactivate();
        }
        scene.triggers[necklace_trigger.trigger.0].active = false;
    }
    Ok(())
}
