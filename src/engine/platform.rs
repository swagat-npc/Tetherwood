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

use crate::engine::renderer::Renderer;
use crate::engine::scene::Scene;

pub fn run() {
    let event_loop = EventLoop::new().expect("failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        window: None,
        last_frame: Instant::now(),
        frame_count: 0,
        renderer: None,
        scene: None,
        held_keys: HashSet::new(),
        multiplying_factor: 2.5,
    };
    event_loop.run_app(&mut app).expect("event loop error");
}

struct App {
    window: Option<Arc<Window>>,
    last_frame: Instant,
    frame_count: u32,
    renderer: Option<Renderer>,
    scene: Option<Scene>,
    held_keys: HashSet<KeyCode>,
    multiplying_factor: f32,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Tetherwood")
            .with_inner_size(winit::dpi::LogicalSize::new(512.0, 512.0));

        let window = event_loop.create_window(window_attributes).unwrap();
        let window = Arc::new(window);

        let mut renderer =
            block_on(Renderer::new(window.clone())).expect("failed to initialize renderer");

        let scene =
            Scene::new_bedroom(renderer.device(), renderer.queue(), self.multiplying_factor)
                .expect("failed to build bedroom scene");
        renderer.prepare_scene(&scene);

        // Static indoor camera: anchored once at the room's own center,
        // never updated per-frame. Follow-mode (M2's camera-tracks-
        // player behavior) is deferred until an outdoor/follow scene
        // needs it — no CameraMode abstraction built yet, since there's
        // only one real consumer so far (same reasoning as ADR-035).
        renderer.camera_position = glam::Vec2::new(64.0, 64.0) * self.multiplying_factor;

        self.scene = Some(scene);
        self.renderer = Some(renderer);
        self.window = Some(window);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
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
                self.renderer
                    .as_mut()
                    .unwrap()
                    .resize(size.width, size.height);
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
                if let Some(scene) = self.scene.as_mut() {
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
                        scene.try_move_player(delta_move);
                    }
                }

                if let (Some(renderer), Some(scene)) = (self.renderer.as_mut(), self.scene.as_ref())
                {
                    match renderer.render(scene) {
                        Ok(()) => {}
                        Err(e) => {
                            log::error!("render failed: {e}");
                            event_loop.exit();
                        }
                    };
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
