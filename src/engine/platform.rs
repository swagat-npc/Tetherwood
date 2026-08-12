use pollster::block_on;
use std::collections::HashSet;
use std::{sync::Arc, time::Instant};
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::engine::ids::SceneId;
use crate::engine::renderer::Renderer;
use crate::engine::scene::{CameraMode, Scene};

pub fn run() {
    let event_loop = EventLoop::new().expect("failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App { state: None };
    event_loop.run_app(&mut app).expect("event loop error");
}

struct App {
    state: Option<AppState>,
}

struct AppState {
    window: Arc<Window>,
    renderer: Renderer,
    scene: Scene,
    last_frame: Instant,
    frame_count: u32,
    held_keys: HashSet<KeyCode>,
    multiplying_factor: f32, // TODO: migrate this to a config struct to supply everywhere
    show_colliders: bool,
    show_debug_info: bool,
    show_test_text: bool,
}

impl AppState {
    /// Builds and GPU-prepares a scene. Free of `self` so it can run
    /// before AppState exists (resumed()'s first scene) as well as
    /// from change_scene, which just assigns the result afterward.
    fn build_scene(renderer: &mut Renderer, scene_id: SceneId, multiplying_factor: f32) -> Scene {
        let new_scene = match scene_id {
            SceneId::Home => {
                Scene::new_home(renderer.device(), renderer.queue(), multiplying_factor)
                    .expect("failed to build home scene")
            }
            SceneId::Outside => {
                Scene::new_outside(renderer.device(), renderer.queue(), multiplying_factor)
                    .expect("failed to build outside scene")
            }
        };
        renderer.prepare_scene(&new_scene);
        new_scene
    }

    fn change_scene(&mut self, scene_id: SceneId) {
        self.scene = Self::build_scene(&mut self.renderer, scene_id, self.multiplying_factor);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // TODO: Top level stuct to always be in memory, give it a hashmap of scene or anything that is needed everytime
        // and just call it when needed
        if self.state.is_some() {
            // resumed() firing again (Android window-reclaim) is a
            // no-op on this desktop-only target - guard kept explicit
            // rather than assumed away.
            return;
        }

        let window_attributes = Window::default_attributes()
            .with_title("Tetherwood")
            .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
            .with_position(winit::dpi::LogicalPosition::new(0.0, 100.0));
        let window = event_loop.create_window(window_attributes).unwrap();
        let window = Arc::new(window);

        let mut renderer =
            block_on(Renderer::new(window.clone())).expect("failed to initialize renderer");
        let multiplying_factor = 5.0;

        let initial_scene = AppState::build_scene(&mut renderer, SceneId::Home, multiplying_factor);

        let state = AppState {
            window,
            renderer,
            scene: initial_scene,
            last_frame: Instant::now(),
            frame_count: 0,
            held_keys: HashSet::new(),
            multiplying_factor,
            show_colliders: true, // DEBUG: set to true for debugging
            show_debug_info: false,
            show_test_text: false,
        };

        self.state = Some(state);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let Some(state) = &self.state else { return };
        state.window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = &mut self.state else { return };

        match event {
            WindowEvent::CloseRequested => {
                println!("Close button pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                state.renderer.resize(size.width, size.height);
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let delta = now - state.last_frame;
                state.last_frame = now;

                state.frame_count += 1;
                if state.frame_count.is_multiple_of(60) {
                    println!(
                        "delta: {:.2?} (~{:.0} fps)",
                        delta,
                        1.0 / delta.as_secs_f32()
                    );
                }

                // TODO(engine): raw KeyCode handling here is content, not machinery (ADR-035).
                let speed = 80.0 * state.multiplying_factor; // pixels per second scaled up to the factor
                let mut movement = glam::Vec2::ZERO;
                if state.held_keys.contains(&KeyCode::KeyW) {
                    movement.y -= 1.0;
                }
                if state.held_keys.contains(&KeyCode::KeyS) {
                    movement.y += 1.0;
                }
                if state.held_keys.contains(&KeyCode::KeyA) {
                    movement.x -= 1.0;
                }
                if state.held_keys.contains(&KeyCode::KeyD) {
                    movement.x += 1.0;
                }
                if movement != glam::Vec2::ZERO {
                    let delta_move = movement.normalize() * speed * delta.as_secs_f32();
                    state.scene.try_move_player(delta_move);
                    if let Some((target_scene, target_warp_id)) =
                        state.scene.check_triggers(state.show_debug_info)
                    {
                        state.change_scene(target_scene);
                        if let Some(spawn_position) = state.scene.activate_warp(target_warp_id) {
                            state.scene.player_mut().position = spawn_position;
                        }
                    }
                }

                let camera_target = match state.scene.camera_mode {
                    CameraMode::Static(anchor) => anchor,
                    CameraMode::Follow => state.scene.player().position,
                };
                state.renderer.camera_position = camera_target;

                match state.renderer.acquire_frame() {
                    Ok(Some(frame)) => {
                        state
                            .renderer
                            .render_scene(&frame, &state.scene, state.show_colliders);
                        if state.show_test_text {
                            let glyphs = crate::engine::text::layout_text(
                                "The Quick Brown Fox, Jumped Over the Lazy Dog! With the new font @ == estäblished*",
                                glam::Vec2::new(20.0, 500.0),
                            );
                            state.renderer.render_text(&frame, &glyphs);
                        }
                        state.renderer.present_frame(frame);
                    }
                    Ok(None) => {} // surface not ready yet, skip this frame
                    Err(e) => {
                        log::error!("render failed: {e}");
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        repeat: false,
                        ..
                    },
                ..
            } => match key_state {
                winit::event::ElementState::Pressed => {
                    if code == KeyCode::Escape {
                        println!("Escape key pressed; stopping");
                        event_loop.exit();
                    } else if code == KeyCode::F1 {
                        state.show_colliders = !state.show_colliders;
                        println!(
                            "{} Colliders",
                            if state.show_colliders { "Show" } else { "Hide" }
                        );
                    } else if code == KeyCode::F2 {
                        state.show_debug_info = !state.show_debug_info;
                        println!(
                            "{} Debug Info",
                            if state.show_debug_info {
                                "Show"
                            } else {
                                "Hide"
                            }
                        );
                    } else if code == KeyCode::F3 {
                        state.show_test_text = !state.show_test_text;
                        println!(
                            "{} Test Text",
                            if state.show_test_text { "Show" } else { "Hide" }
                        );
                    }
                    if state.show_debug_info {
                        println!("{code:?} pressed");
                    }
                    state.held_keys.insert(code);
                }
                winit::event::ElementState::Released => {
                    state.held_keys.remove(&code);
                }
            },
            _ => {}
        }
    }
}
