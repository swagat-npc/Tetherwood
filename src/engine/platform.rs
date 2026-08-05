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

// Call the Renderer sub module inside the engine module
use crate::engine::renderer::Renderer;

pub fn run() {
    let event_loop = EventLoop::new().expect("failed to create event loop");

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        window: None,
        last_frame: Instant::now(),
        frame_count: 0,
        renderer: None,
        held_keys: HashSet::new(),
    };
    event_loop.run_app(&mut app).expect("event loop error");
}

struct App {
    window: Option<Arc<Window>>,
    last_frame: Instant,
    frame_count: u32,
    renderer: Option<Renderer>,
    held_keys: HashSet<KeyCode>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Tetherwood")
            .with_inner_size(winit::dpi::LogicalSize::new(512.0, 512.0));

        let window = event_loop.create_window(window_attributes).unwrap();
        let window = Arc::new(window);

        let renderer =
            block_on(Renderer::new(window.clone())).expect("failed to initialize renderer");
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
                // println!("Window redraw requested");
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                let now = Instant::now();
                let delta = now - self.last_frame; // std::time::Duration
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
                // Extract an Action/InputMap layer once a second input-consuming system exists
                // (dialogue advance, menus, battle input).
                if let Some(renderer) = self.renderer.as_mut() {
                    let speed = 100.0; // pixels per second
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
                        renderer.sprite_position +=
                            movement.normalize() * speed * delta.as_secs_f32();
                    }
                    renderer.camera_position = renderer.sprite_position;
                }

                match self.renderer.as_mut().unwrap().render() {
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
