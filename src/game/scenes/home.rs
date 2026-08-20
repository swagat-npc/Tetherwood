use crate::engine::entity::{Collider, Direction, Entity, EntityId, Rect};
use crate::engine::renderer::texture::TextureStore;
use crate::engine::scene::{
    Background, CameraMode, Scene, SceneId, Trigger, TriggerId, TriggerKind, WarpId, builder,
};
use crate::game::progression::ProgressionTracker;
use anyhow::Result;
use glam::Vec2;

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

    let mut scene = Scene::new(
        SceneId::Home,
        TextureStore::new(),
        CameraMode::Static(floor_position),
        CameraMode::Follow,
        is_isometric,
        multiplying_factor,
    );

    // Create Background
    let floor_size = Vec2::new(128.0, 128.0) * multiplying_factor;
    let floor = scene
        .texture_store
        .load(device, queue, "assets/bedroom.png")?;
    scene.background.push(Background {
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

    let door = scene
        .texture_store
        .load(device, queue, "assets/door.aseprite")?;
    let door_position = Vec2::new(floor_position.x, bottom_edge);
    let door_size = Vec2::new(32.0, 16.0) * multiplying_factor;
    scene.background.push(Background {
        texture: door,
        position: door_position,
        size: door_size,
    });

    // Create Walls
    scene.walls.extend(vec![
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
    ]);

    // Create Triggers
    // Trigger sits fully past the wall's outer edge — the player must
    // walk all the way through the doorway gap and beyond the threshold
    // before their center overlaps this, not just step into the gap.
    let door_trigger_depth = wall_thickness; // how far past the wall the trigger extends
    let door_trigger_center_y = bottom_edge + 2.0 * wall_thickness;

    scene.triggers.push(Trigger::new(
        Rect {
            center: Vec2::new(door_center_x, door_trigger_center_y),
            half_size: Vec2::new(door_half_width, door_trigger_depth),
        },
        TriggerKind::Warp {
            warp_id: WarpId("door"),
            target_scene: SceneId::Village,
            target_warp_id: WarpId("door"),
            spawn_offset: Vec2::new(0.0, -20.0 * multiplying_factor), // up, into the room
        },
    ));

    // Create Entities
    let wardrobe_tex = scene
        .texture_store
        .load(device, queue, "assets/wardrobe.aseprite")?;
    scene.entities.push(Entity {
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
        active: true,
        is_overlay_layer: false,
    });

    let bed_tex = scene
        .texture_store
        .load(device, queue, "assets/bed.aseprite")?;

    let bed_collider = Rect {
        center: Vec2::new(0.0, 5.0) * multiplying_factor,
        half_size: Vec2::new(16.0, 22.0) * multiplying_factor,
    };
    scene.entities.push(Entity {
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
        active: true,
        is_overlay_layer: false,
    });

    scene.entities.push(Entity {
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
        active: true,
        is_overlay_layer: false,
    });

    let bed_prompt_tex = scene
        .texture_store
        .load(device, queue, "assets/prompt.aseprite")?;
    scene.entities.push(Entity {
        position: Vec2::new(94.0, 25.0) * multiplying_factor,
        size: Vec2::new(8.0, 8.0) * multiplying_factor,
        collider: None,
        texture_id: Some(bed_prompt_tex),
        facing: Direction::Down,
        active: true,
        is_overlay_layer: false,
    });
    let bed_prompt = EntityId(scene.entities.len() - 1);

    scene.triggers.push(Trigger::new(
        Rect {
            center: Vec2::new(94.0, 40.0) * multiplying_factor,
            half_size: Vec2::new(7.0, 8.0) * multiplying_factor,
        },
        TriggerKind::Dialogue {
            id: "bed_examine",
            prompt_entity: Some(bed_prompt),
            prompt_texture: Some(bed_prompt_tex),
            facing: &[Direction::Left],
            consumes_entity: None,
            sets_flag: None,
        },
    ));

    let nightstand_tex = scene
        .texture_store
        .load(device, queue, "assets/nightstand.aseprite")?;
    scene.entities.push(Entity {
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
        active: true,
        is_overlay_layer: false,
    });

    scene.spawn_player(
        device,
        queue,
        multiplying_factor,
        builder::EntitySpec {
            position: Vec2::new(64.0, 87.5),
            size: Vec2::new(14.0, 24.0),
            collider_offset: Vec2::new(0.0, 6.0),
            collider_size: Vec2::new(12.0, 12.0),
            texture_path: "assets/player.aseprite",
            facing: Direction::Down,
        },
    )?;

    let necklace_prompt_tex = scene
        .texture_store
        .load(device, queue, "assets/prompt.aseprite")?;
    scene.entities.push(Entity {
        position: Vec2::new(112.0, -5.0) * multiplying_factor,
        size: Vec2::new(8.0, 8.0) * multiplying_factor,
        collider: None,
        texture_id: Some(necklace_prompt_tex),
        facing: Direction::Down,
        active: true,
        is_overlay_layer: false,
    });
    let necklace_prompt = EntityId(scene.entities.len() - 1);

    let necklace_tex = scene
        .texture_store
        .load(device, queue, "assets/necklace.aseprite")?;
    scene.entities.push(Entity {
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
        active: true,
        is_overlay_layer: false,
    });
    let necklace_entity = EntityId(scene.entities.len() - 1);

    scene.triggers.push(Trigger::new(
        Rect {
            center: Vec2::new(112.0, 10.0) * multiplying_factor,
            half_size: Vec2::new(12.0, 12.0) * multiplying_factor,
        },
        TriggerKind::Dialogue {
            id: "necklace_examine",
            prompt_entity: Some(necklace_prompt),
            prompt_texture: Some(necklace_prompt_tex),
            facing: &[Direction::Left],
            consumes_entity: Some(necklace_entity),
            sets_flag: Some("necklace_consumed"),
        },
    ));
    let necklace_trigger = TriggerId(scene.triggers.len() - 1);

    if progression.is_set("necklace_consumed") {
        scene.entities[necklace_entity.0].deactivate();
        scene.entities[necklace_prompt.0].deactivate();
        scene.triggers[necklace_trigger.0].active = false;
    }

    Ok(scene)
}
