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

use crate::engine::entity::TriggerKind::{self, Warp};
use crate::engine::ids::SceneId;
use crate::engine::renderer::Renderer;
use crate::engine::scene::Scene;

pub fn run() {
    let event_loop = EventLoop::new().expect("failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let window_attributes = Window::default_attributes()
        .with_title("Tetherwood")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .with_position(winit::dpi::LogicalPosition::new(0.0, 100.0));
    let window = event_loop.create_window(window_attributes).unwrap();
    let window = Arc::new(window);

    let renderer = block_on(Renderer::new(window.clone())).expect("failed to initialize renderer");
    let multiplying_factor = 5.0;

    let scenes = Vec::new();

    let mut app = App {
        window: window,
        last_frame: Instant::now(),
        frame_count: 0,
        renderer: renderer,
        scenes: scenes,
        current_scene: 0,
        held_keys: HashSet::new(),
        multiplying_factor: multiplying_factor,
        show_colliders: true, // TODO: set to true for debugging
        show_debug_info: false,
    };
    app.change_scene(SceneId::Home);
    event_loop.run_app(&mut app).expect("event loop error");
}

struct App {
    window: Arc<Window>,
    last_frame: Instant,
    frame_count: u32,
    renderer: Renderer,
    scenes: Vec<Scene>,
    current_scene: usize,
    held_keys: HashSet<KeyCode>,
    multiplying_factor: f32, // TODO: migrate this to a config struct to supply everywhere
    show_colliders: bool,
    show_debug_info: bool,
}

impl App {
    fn change_scene(&mut self, scene_id: SceneId) {
        if let Some(existing_index) = self.scenes.iter().position(|scene| scene.id == scene_id) {
            self.current_scene = existing_index;
        } else {
            let new_scene = match scene_id {
                SceneId::Home => Scene::new_home(
                    self.renderer.device(),
                    self.renderer.queue(),
                    self.multiplying_factor,
                )
                .expect("failed to build bedroom scene"),
                SceneId::Outside => Scene::new_outside(
                    self.renderer.device(),
                    self.renderer.queue(),
                    self.multiplying_factor,
                )
                .expect("failed to build outside scene"),
            };
            self.scenes.push(new_scene);
            self.current_scene = self.scenes.len() - 1;
        }
        self.renderer
            .prepare_scene(&self.scenes[self.current_scene]);
    }

    #[inline]
    fn get_current_scene(&self) -> &Scene {
        &self.scenes[self.current_scene]
    }

    #[inline]
    fn get_current_scene_mut(&mut self) -> &mut Scene {
        &mut self.scenes[self.current_scene]
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // TODO: Top level stuct to always be in memory, give it a hashmap of scene or anything that is needed everytime
        // and just call it when needed

        // Static indoor camera: anchored once at the room's own center,
        // never updated per-frame. Follow-mode (M2's camera-tracks-
        // player behavior) is deferred until an outdoor/follow scene
        // needs it — no CameraMode abstraction built yet, since there's
        // only one real consumer so far (same reasoning as ADR-035).
        self.renderer.camera_position = glam::Vec2::new(64.0, 64.0) * self.multiplying_factor;
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Close button pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.renderer.resize(size.width, size.height);
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let delta = now - self.last_frame;
                self.last_frame = now;

                self.frame_count += 1;
                if self.frame_count.is_multiple_of(60) {
                    println!(
                        "delta: {:.2?} (~{:.0} fps)",
                        delta,
                        1.0 / delta.as_secs_f32()
                    );
                }

                // TODO(engine): raw KeyCode handling here is content, not machinery (ADR-035).

                {
                    let speed = 80.0 * self.multiplying_factor; // pixels per second scaled up to the factor
                    let mut movement = glam::Vec2::ZERO;
                    if self.held_keys.contains(&KeyCode::KeyW) {
                        movement.y -= 1.0;
                    }
                    if self.held_keys.contains(&KeyCode::KeyS) {
                        movement.y += 1.0;
                    }
                    if self.held_keys.contains(&KeyCode::KeyA) {
                        movement.x -= 1.0;
                    }
                    if self.held_keys.contains(&KeyCode::KeyD) {
                        movement.x += 1.0;
                    }
                    if movement != glam::Vec2::ZERO {
                        let delta_move = movement.normalize() * speed * delta.as_secs_f32();
                        let scene = self.get_current_scene_mut();
                        scene.try_move_player(delta_move);
                        if let Some((target_scene, target_warp_id)) = scene.check_triggers() {
                            self.change_scene(target_scene);
                            self.get_current_scene_mut().activate_warp(target_warp_id);
                        }
                    }
                }

                match self
                    .renderer
                    .render(&self.scenes[self.current_scene], self.show_colliders)
                {
                    Ok(()) => {}
                    Err(e) => {
                        log::error!("render failed: {e}");
                        event_loop.exit();
                    }
                };
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
                        self.show_colliders = !self.show_colliders;
                        println!(
                            "{} Colliders",
                            if self.show_colliders { "Show" } else { "Hide" }
                        );
                    } else if code == KeyCode::F2 {
                        self.show_debug_info = !self.show_debug_info;
                        println!(
                            "{} Debug Info",
                            if self.show_debug_info { "Show" } else { "Hide" }
                        );
                    }
                    if self.show_debug_info {
                        println!("{code:?} pressed");
                    }
                    self.held_keys.insert(code);
                }
                winit::event::ElementState::Released => {
                    self.held_keys.remove(&code);
                }
            },
            _ => {}
        }
    }
}
