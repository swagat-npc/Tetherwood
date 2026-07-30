use pollster::block_on;
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
    };
    event_loop.run_app(&mut app).expect("event loop error");
}

struct App {
    window: Option<Arc<Window>>,
    last_frame: Instant,
    frame_count: u32,
    renderer: Option<Renderer>,
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
                        repeat,
                        ..
                    },
                ..
            } => match (code, key_state.is_pressed(), repeat) {
                (KeyCode::KeyW, true, false) => println!("{:?} Key Pressed", Directions::Up),
                (KeyCode::KeyA, true, false) => println!("{:?} Key Pressed", Directions::Left),
                (KeyCode::KeyS, true, false) => println!("{:?} Key Pressed", Directions::Down),
                (KeyCode::KeyD, true, false) => println!("{:?} Key Pressed", Directions::Right),
                (KeyCode::Escape, true, false) => {
                    println!("Escape key pressed; stopping");
                    event_loop.exit();
                }
                _ => {}
            },
            _ => {}
        }
    }
}

#[derive(Debug)]
enum Directions {
    Up,
    Down,
    Left,
    Right,
}
