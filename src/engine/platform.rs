use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

pub fn run() {
    let event_loop = EventLoop::new().expect("failed to create event loop");

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        window: None,
        window_id: None,
        last_frame: Instant::now(),
        frame_count: 0,
    };
    event_loop.run_app(&mut app).expect("event loop error");
}

struct App {
    window: Option<Window>, // None until `resumed` — the fight I promised
    window_id: Option<WindowId>,
    last_frame: Instant,
    frame_count: u32,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Tetherwood: The Morning She Was Gone — window ok")
            .with_inner_size(winit::dpi::LogicalSize::new(512.0, 512.0));

        let window = event_loop.create_window(window_attributes).unwrap();
        self.window_id = Some(window.id());
        self.window = Some(window);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
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
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // println!("Window redraw requested");
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => match (code, key_state.is_pressed()) {
                (KeyCode::KeyW, false) => println!("{} Key Pressed", Directions::Up),
                (KeyCode::KeyA, false) => println!("{} Key Pressed", Directions::Left),
                (KeyCode::KeyS, false) => println!("{} Key Pressed", Directions::Down),
                (KeyCode::KeyD, false) => println!("{} Key Pressed", Directions::Right),
                (KeyCode::Escape, true) => {
                    println!("Escape key pressed; stopping");
                    event_loop.exit();
                }
                _ => {}
            },
            _ => {}
        }
    }
}

enum Directions {
    Up,
    Down,
    Left,
    Right,
}

impl std::fmt::Display for Directions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Directions::Up => write!(f, "Up"),
            Directions::Down => write!(f, "Down"),
            Directions::Left => write!(f, "Left"),
            Directions::Right => write!(f, "Right"),
        }
    }
}
