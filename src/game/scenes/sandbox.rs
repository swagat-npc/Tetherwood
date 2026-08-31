use crate::engine::entity::Direction;
use crate::engine::renderer::texture::TextureStore;
use crate::engine::renderer::tile;
use crate::engine::scene::{CameraMode, Scene, SceneId, builder};
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
    _progression: &ProgressionTracker,
) -> Result<Scene> {
    // Establish positioning
    let mut scene = Scene::new(
        SceneId::Sandbox,
        TextureStore::new(),
        CameraMode::Follow,
        CameraMode::Follow,
        is_isometric,
        multiplying_factor,
    );

    // Create Entities
    build_entities(&mut scene, device, queue, multiplying_factor, is_isometric)?;

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

fn build_entities(
    scene: &mut Scene,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    multiplying_factor: f32,
    is_isometric: bool,
) -> Result<()> {
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
            tile::world_at_cell((0, -5), multiplying_factor),
            nightstand_size,
            nightstand_base_size,
            Vec2::new(0.0, 0.0),
            nightstand_collider_size,
            "nightstand",
            Direction::Down,
        ),
    )?; // nightstand

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
            tile::world_at_cell((4, -5), multiplying_factor),
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
            tile::world_at_cell((9, -5), multiplying_factor),
            bed_size,
            bed_base_size,
            bed_collider_offset,
            bed_collider_size,
            "bed_her",
            Direction::Down,
        ),
    )?; // bed (right)

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
            tile::world_at_cell((0, -11), multiplying_factor)
                + Vec2::new(0.0, 6.0) * multiplying_factor,
            wardrobe_size,
            wardrobe_base_size,
            Vec2::ZERO,
            wardrobe_collider_size,
            "wardrobe",
            Direction::Down,
        ),
    )?; // wardrobe

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
            tile::world_at_cell((4, -11), multiplying_factor),
            crate_size,
            crate_base_size,
            Vec2::ZERO,
            crate_collider_size,
            "crate",
            Direction::Down,
        ),
    )?; // crate

    let player_collider_offset = if is_isometric {
        Vec2::new(0.0, 0.0)
    } else {
        Vec2::new(0.0, 0.0)
    };
    let player_collider_size = if is_isometric {
        Vec2::new(6.0, 6.0)
    } else {
        Vec2::new(12.0, 6.0)
    };

    scene.spawn_player(
        device,
        queue,
        multiplying_factor,
        is_isometric,
        builder::EntitySpec {
            position: tile::world_at_cell((2, -4), multiplying_factor),
            render_size: Vec2::new(16.0, 32.0),
            base_size: player_collider_size,
            collider_offset: player_collider_offset,
            collider_size: player_collider_size,
            name: "player",
            facing: Direction::Down,
            anchor: builder::FootprintAnchor::BottomCenter,
        },
    )?;

    for k in [0, 1, 2] {
        for i in [0, 1, 2] {
            for j in [0, 1, 2] {
                let x = (i + 1) as f32;
                let y = (j + 1) as f32;
                let z = (k + 1) as f32;
                let filename = format!("assets/blocks/Block {}x{}x{}.png", x, y, z);
                let block_base_size = tile::grid_to_pixel((x, y));

                let base_height = tile::isometric_footprint_base_height(x, y);
                let padding = tile::isometric_footprint_padding(x, y);
                let block_size_x = x + y;
                let block_size_y = base_height + (z - 1.0);
                let block_size =
                    tile::grid_to_pixel((block_size_x, block_size_y)) + Vec2::new(0.0, padding);

                let block_collider_size = block_base_size;
                let tile_gap = 3;
                let x_pos = 0 + j * (tile_gap + 1 * i) + i * (tile_gap * 3 + 1 * i);
                let y_pos = 0 + k * 6;
                scene.spawn_entity_with_path(
                    device,
                    queue,
                    multiplying_factor,
                    is_isometric,
                    true,
                    &filename,
                    builder::EntitySpec::new(
                        tile::world_at_cell((x_pos, y_pos), multiplying_factor),
                        block_size,
                        block_base_size,
                        Vec2::new(0.0, 0.0),
                        block_collider_size,
                        "Blocks",
                        Direction::Down,
                    ),
                )?; // block
            }
        }
    }

    Ok(())
}
