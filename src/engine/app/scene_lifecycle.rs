use crate::engine::app::AppState;
use crate::engine::renderer::Renderer;
use crate::engine::scene::{Scene, SceneId};
use crate::game::progression::ProgressionTracker;
use crate::game::scenes::{home, sandbox, village};

impl AppState {
    /// Builds and GPU-prepares a scene. Free of `self` so it can run
    /// before AppState exists (resumed()'s first scene) as well as
    /// from change_scene, which just assigns the result afterward.
    pub fn build_scene(
        renderer: &mut Renderer,
        scene_id: SceneId,
        multiplying_factor: f32,
        is_isometric: bool,
        progression: &mut ProgressionTracker,
    ) -> Scene {
        let mut new_scene = match scene_id {
            SceneId::Home => home::build(
                renderer.device(),
                renderer.queue(),
                multiplying_factor,
                is_isometric,
                progression,
            )
            .expect("failed to build home scene"),
            SceneId::Village => village::build(
                renderer.device(),
                renderer.queue(),
                multiplying_factor,
                is_isometric,
                progression,
            )
            .expect("failed to build village scene"),
            SceneId::Sandbox => sandbox::build(
                renderer.device(),
                renderer.queue(),
                multiplying_factor,
                is_isometric,
                progression,
            )
            .expect("failed to debug home scene"),
        };
        Self::load_tilemap(&mut new_scene);
        new_scene.build_static_grid(multiplying_factor);
        renderer.prepare_scene(&new_scene);

        renderer.camera_position = new_scene.camera_target();
        let player_position = new_scene.player().position;
        renderer.snap_camera(player_position, is_isometric);

        new_scene
    }

    pub fn change_scene(&mut self, scene_id: SceneId) {
        self.scene = Self::build_scene(
            &mut self.renderer,
            scene_id,
            self.multiplying_factor,
            self.is_isometric,
            &mut self.progression,
        );
        // Note: This is only relevant if player is allowed to
        // move when in the tile editor mode
        if self.debug.show_tile_editor {
            self.reset_paint_session();
        }
    }

    pub fn reset_scene(&mut self) {
        self.scene = Self::build_scene(
            &mut self.renderer,
            self.scene.id,
            self.multiplying_factor,
            self.is_isometric,
            &mut self.progression,
        );
    }

    pub fn refresh_scene(&mut self) {
        let mut new_scene = match self.scene.id {
            SceneId::Home => home::refresh(
                &mut self.scene,
                self.renderer.device(),
                self.renderer.queue(),
                self.multiplying_factor,
                self.is_isometric,
                &mut self.progression,
            )
            .expect("failed to refresh home scene"),
            SceneId::Village => village::refresh(
                &mut self.scene,
                self.renderer.device(),
                self.renderer.queue(),
                self.multiplying_factor,
                self.is_isometric,
                &mut self.progression,
            )
            .expect("failed to refresh village scene"),
            SceneId::Sandbox => sandbox::refresh(
                &mut self.scene,
                self.renderer.device(),
                self.renderer.queue(),
                self.multiplying_factor,
                self.is_isometric,
                &mut self.progression,
            )
            .expect("failed to refresh debug scene"),
        };

        Self::load_tilemap(&mut new_scene);
        new_scene.build_static_grid(self.multiplying_factor);
        self.renderer.prepare_scene(&new_scene);

        self.renderer.camera_position = new_scene.camera_target();
        let player_position = new_scene.player().position;
        self.renderer
            .snap_camera(player_position, self.is_isometric);

        self.scene = new_scene;
    }
}
