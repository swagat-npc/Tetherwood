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
    // event_loop.set_control_flow(ControlFlow::Poll);

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App {
        window: None,
        window_id: None,
    };
    event_loop.run_app(&mut app).expect("event loop error");
}

struct App {
    window: Option<Window>, // None until `resumed` — the fight I promised
    window_id: Option<WindowId>,
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
                (KeyCode::KeyW, true) => println!("{} Key Pressed", Directions::Up),
                (KeyCode::KeyA, true) => println!("{} Key Pressed", Directions::Left),
                (KeyCode::KeyS, true) => println!("{} Key Pressed", Directions::Down),
                (KeyCode::KeyD, true) => println!("{} Key Pressed", Directions::Right),
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
