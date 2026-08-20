use crate::engine::app::AppState;
use crate::engine::entity::Direction;
use crate::engine::scene::CameraMode;
use crate::game::actions;
use glam::Vec2;

impl AppState {
    pub fn update_player(&mut self, delta: f32) {
        if self.dialogue.is_some() {
            return;
        }
        let speed = 80.0 * self.multiplying_factor;
        let movement = actions::resolve_movement(&self.input, self.is_isometric);
        if movement != Vec2::ZERO {
            // TODO: Direction::from_movement doesn't account for the isometric
            // movement table's diagonal/cardinal split - facing may be wrong in
            // isometric mode. Deferred until facing-while-isometric is a real need.
            if let Some(dir) = Direction::from_movement(movement) {
                self.scene.player_mut().facing = dir;
            }
            let delta_move = movement * speed * delta;
            self.scene.try_move_player(
                delta_move,
                self.multiplying_factor,
                self.debug.enable_player_collider,
            );
            if let Some((target_scene, target_warp_id)) =
                self.scene.check_triggers(self.debug.show_debug_info)
            {
                self.change_scene(target_scene);
                if let Some(spawn_position) = self.scene.activate_warp(target_warp_id) {
                    self.scene.player_mut().position = spawn_position;
                }
            }
        }
        self.scene.update_interact_prompts();

        let camera_target = match self.scene.camera_mode() {
            CameraMode::Static(anchor) => anchor,
            CameraMode::Follow => self.scene.player().position,
        };
        self.renderer.camera_position = camera_target;
    }
}
