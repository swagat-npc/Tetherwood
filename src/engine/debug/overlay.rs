use crate::engine::entity;
use crate::engine::renderer::SolidRect;
use crate::engine::scene::{Scene, TriggerKind};

fn push_center_marker(debug_rects: &mut Vec<SolidRect>, center: glam::Vec2, scale: f32) {
    const ARM_LENGTH: f32 = 8.0;
    const THICKNESS: f32 = 2.0;
    const X_COLOR: [f32; 4] = [1.0, 0.15, 0.15, 1.0]; // X-Axis
    const Y_COLOR: [f32; 4] = [0.15, 1.0, 0.15, 1.0]; // Y-Axis

    debug_rects.push(SolidRect {
        position: center,
        size: glam::Vec2::new(ARM_LENGTH * scale, THICKNESS * scale),
        fill_color: X_COLOR,
        border_color: X_COLOR,
        border_thickness_px: 3.0,
    });
    debug_rects.push(SolidRect {
        position: center,
        size: glam::Vec2::new(THICKNESS * scale, ARM_LENGTH * scale),
        fill_color: Y_COLOR,
        border_color: Y_COLOR,
        border_thickness_px: 3.0,
    });
}

fn push_facing_marker(
    debug_rects: &mut Vec<SolidRect>,
    center: glam::Vec2,
    facing: entity::Direction,
    scale: f32,
) {
    const FACING_COLOR: [f32; 4] = [0.2, 0.8, 1.0, 1.0]; // distinct from X/Y axis colors

    // Three segments, each shorter and thinner than the last as they
    // move away from center — a stepped taper reads as "pointing this
    // way" even using only rects, no true arrowhead geometry needed.
    const SEGMENTS: [(f32, f32); 3] = [(3.0, 5.0), (8.0, 3.0), (13.0, 1.5)];

    let direction_vec = match facing {
        entity::Direction::Up => glam::Vec2::new(0.0, -1.0),
        entity::Direction::Down => glam::Vec2::new(0.0, 1.0),
        entity::Direction::Left => glam::Vec2::new(-1.0, 0.0),
        entity::Direction::Right => glam::Vec2::new(1.0, 0.0),
    };
    let along_axis = direction_vec.x != 0.0;

    for &(offset, thickness) in &SEGMENTS {
        let seg_center = center + direction_vec * (offset * scale);
        let seg_length = 5.0 * scale;
        let size = if along_axis {
            glam::Vec2::new(seg_length, thickness * scale)
        } else {
            glam::Vec2::new(thickness * scale, seg_length)
        };

        debug_rects.push(SolidRect {
            position: seg_center,
            size,
            fill_color: FACING_COLOR,
            border_color: FACING_COLOR,
            border_thickness_px: 0.0,
        });
    }
}

pub fn build_debug_rects(scene: &Scene) -> Vec<SolidRect> {
    const WALL_FILL: [f32; 4] = [1.0, 0.0, 0.0, 0.15];
    const WALL_BORDER: [f32; 4] = [0.7, 0.0, 0.0, 0.9];
    const ENTITY_FILL: [f32; 4] = [0.0, 0.4, 1.0, 0.15];
    const ENTITY_BORDER: [f32; 4] = [0.0, 0.2, 0.8, 0.9];
    const TRIGGER_FILL: [f32; 4] = [0.0, 1.0, 0.0, 0.15];
    const TRIGGER_BORDER: [f32; 4] = [0.0, 0.7, 0.0, 0.9];
    const DIALOGUE_FILL: [f32; 4] = [1.0, 1.0, 0.0, 0.15];
    const DIALOGUE_BORDER: [f32; 4] = [0.7, 0.7, 0.0, 0.9];
    const INTERACT_FILL: [f32; 4] = [1.0, 0.0, 1.0, 0.15];
    const INTERACT_BORDER: [f32; 4] = [0.7, 0.0, 0.7, 0.9];

    let mut debug_rects: Vec<SolidRect> = Vec::new();
    push_center_marker(&mut debug_rects, glam::Vec2::ZERO, 1.0);

    for wall in &scene.walls {
        debug_rects.push(SolidRect {
            position: wall.rect.center,
            size: wall.rect.half_size * 2.0,
            fill_color: WALL_FILL,
            border_color: WALL_BORDER,
            border_thickness_px: 3.0,
        });
        push_center_marker(&mut debug_rects, wall.rect.center, 1.0);
    }
    for entity in &scene.entities {
        if let Some(collider) = &entity.collider {
            debug_rects.push(SolidRect {
                position: entity.position + collider.rect.center,
                size: collider.rect.half_size * 2.0,
                fill_color: ENTITY_FILL,
                border_color: ENTITY_BORDER,
                border_thickness_px: 3.0,
            });
            push_center_marker(
                &mut debug_rects,
                entity.position + collider.rect.center,
                1.0,
            );
        }
        if entity.texture_id.is_some() {
            push_facing_marker(&mut debug_rects, entity.position, entity.facing, 1.0);
        }
    }
    for trigger in &scene.triggers {
        if !trigger.active {
            continue;
        }
        let (fill_color, border_color) = match trigger.kind {
            TriggerKind::Warp { .. } => (TRIGGER_FILL, TRIGGER_BORDER),
            TriggerKind::Dialogue { .. } => (DIALOGUE_FILL, DIALOGUE_BORDER),
            TriggerKind::Toggle { .. } => (INTERACT_FILL, INTERACT_BORDER),
        };

        debug_rects.push(SolidRect {
            position: trigger.rect.center,
            size: trigger.rect.half_size * 2.0,
            fill_color,
            border_color,
            border_thickness_px: 3.0,
        });
        push_center_marker(&mut debug_rects, trigger.rect.center, 1.0);
    }

    debug_rects
}
