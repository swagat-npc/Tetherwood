mod dialogue;
pub mod input;
mod layers;
mod player;
mod scene_lifecycle;

use super::debug::DebugSettings;
use crate::engine::debug::inspector::Inspector;
use crate::engine::debug::notifications::Notification;
use crate::engine::renderer::{Renderer, tile};
use crate::engine::scene::{InteractResult, Scene, SceneId};
use crate::game::actions::{self, Action};
use crate::game::dialogue::line_for;
use crate::game::progression::ProgressionTracker;
use dialogue::DialogueState;
use glam::Vec2;
use input::InputState;
use kira::sound::static_sound::StaticSoundData;
use pollster::block_on;
use std::time::Duration;
use std::{sync::Arc, time::Instant};
use winit::window::{CursorIcon, CustomCursor};
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

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
    input: InputState,
    multiplying_factor: f32, // TODO: migrate this to a config struct to supply everywhere
    dialogue: Option<DialogueState>,
    screen_mouse_position: (f64, f64),
    tile_cursor: CustomCursor,
    smoothed_fps: f32,
    audio: kira::AudioManager<kira::DefaultBackend>,
    audio_blip: Vec<StaticSoundData>,
    blip_step_counter: usize,
    blip_volume: f32,
    notifications: Vec<Notification>,
    left_mouse_down: bool,
    debug: DebugSettings,
    inspector: Inspector,
    is_isometric: bool,
    progression: ProgressionTracker,
}

impl AppState {
    fn notify(&mut self, message: impl Into<String>) {
        self.notifications.push(Notification {
            message: message.into(),
            duration: Duration::from_secs(2),
            start_time: Instant::now(),
        })
    }

    fn tick_frame_timing(&mut self) -> f32 {
        let now = Instant::now();
        let delta = now - self.last_frame;
        self.last_frame = now;

        let instantaneous_fps = 1.0 / delta.as_secs_f32();
        self.smoothed_fps = self.smoothed_fps * 0.9 + instantaneous_fps * 0.1;

        self.frame_count += 1;
        if self.frame_count.is_multiple_of(60) {
            println!(
                "delta: {:.2?} (~{:.0} fps)",
                delta,
                1.0 / delta.as_secs_f32()
            );
        }

        delta.as_secs_f32()
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            // resumed() firing again (Android window-reclaim) is a
            // no-op on this desktop-only target - guard kept explicit
            // rather than assumed away.
            return;
        }

        let window_size = Vec2::new(800.0, 600.0);
        let window_attributes = Window::default_attributes()
            .with_title("Tetherwood")
            .with_inner_size(winit::dpi::LogicalSize::new(window_size.x, window_size.y))
            .with_position(winit::dpi::LogicalPosition::new(0.0, 100.0));
        let window = event_loop.create_window(window_attributes).unwrap();
        let window = Arc::new(window);

        // Load custom cursor
        let image = image::open("assets/cursor-pointer.png").unwrap().to_rgba8();
        let (width, height) = image.dimensions();
        let rgba_pixels = image.into_raw();

        // Define hotspot coordinates (e.g., center or top-left 0,0)
        let hotspot_x: u16 = 0;
        let hotspot_y: u16 = 0;

        // Create the custom cursor source
        let cursor_source = winit::window::CustomCursor::from_rgba(
            rgba_pixels,
            width as u16,
            height as u16,
            hotspot_x,
            hotspot_y,
        )
        .unwrap();

        // Register the cursor with the event loop
        let custom_cursor = event_loop.create_custom_cursor(cursor_source);

        let mut renderer =
            block_on(Renderer::new(window.clone())).expect("failed to initialize renderer");
        let multiplying_factor = 5.0;
        let mut progression = ProgressionTracker::new();

        let initial_scene = AppState::build_scene(
            &mut renderer,
            SceneId::Home,
            multiplying_factor,
            false,
            &mut progression,
        );

        let audio =
            kira::AudioManager::<kira::DefaultBackend>::new(kira::AudioManagerSettings::default())
                .expect("failed to initialize audio");
        let mut audio_blip = Vec::new();
        audio_blip.push(
            StaticSoundData::from_file("assets/sound/blip_narrator.wav")
                .expect("failed to load narrator blip"),
        );
        audio_blip.push(
            StaticSoundData::from_file("assets/sound/blip_monologue.wav")
                .expect("failed to load monologue blip"),
        );

        let inspector = Inspector::new(renderer.screen_size());

        let state = AppState {
            window,
            renderer,
            scene: initial_scene,
            last_frame: Instant::now(),
            frame_count: 0,
            input: InputState::new(),
            multiplying_factor,
            dialogue: None,
            screen_mouse_position: (0.0, 0.0),
            tile_cursor: custom_cursor,
            smoothed_fps: 60.0,
            audio,
            audio_blip,
            blip_step_counter: 0,
            blip_volume: -24.0,
            notifications: Vec::new(),
            left_mouse_down: false,
            inspector,
            debug: DebugSettings::new(),
            is_isometric: false,
            progression: ProgressionTracker::new(),
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
                let delta = state.tick_frame_timing();
                state.tick_dialogue(delta);
                state.update_player(delta);

                let frame = state.renderer.acquire_frame();
                // TODO: display frame time
                match frame {
                    Ok(Some(frame)) => {
                        state.renderer.clear_frame(&frame);

                        // Tile Layer
                        let tile_entries: Vec<(Vec2, (i32, i32))> = state
                            .scene
                            .tile_grid
                            .iter()
                            .map(|(cell, atlas_cell)| {
                                (
                                    tile::tile_world_position(cell, state.multiplying_factor),
                                    atlas_cell,
                                )
                            })
                            .collect();
                        let projection = state.renderer.screen_projection();
                        let sprite_camera_view =
                            state.renderer.sprite_camera_view(state.is_isometric);
                        state.renderer.render_tiles(
                            &frame,
                            &tile_entries,
                            projection,
                            sprite_camera_view,
                            state.multiplying_factor,
                        );

                        // Texture Layer
                        state.renderer.draw_background_and_entities(
                            &frame,
                            &state.scene,
                            state.is_isometric,
                        );

                        // Debug Rect Layer
                        let world_pos = state.renderer.screen_to_world(Vec2::new(
                            state.screen_mouse_position.0 as f32,
                            state.screen_mouse_position.1 as f32,
                        ));
                        state.renderer.draw_debug_geometry(
                            &frame,
                            &state.scene,
                            &state.debug,
                            state.is_isometric,
                            state.multiplying_factor,
                            world_pos,
                        );

                        // HUD Layer
                        state.draw_ui(&frame);

                        // Debug Info Layer
                        state.update_debug_ui();
                        state.draw_debug_info(&frame);

                        state.renderer.present_frame(frame);
                    }
                    Ok(None) => {} // surface not ready yet, skip this frame
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
                        let state_msg = state.debug.toggle_debug_info();
                        state.notify(state_msg);
                    } else if code == KeyCode::F2 {
                        let state_msg = state.debug.toggle_colliders();
                        state.notify(state_msg);
                    } else if code == KeyCode::F3 {
                        let state_msg = state.debug.toggle_debug_renderer();
                        state.notify(state_msg);
                    } else if code == KeyCode::F4 {
                        let state_msg = state.debug.toggle_grid();
                        state.notify(state_msg);
                    } else if code == KeyCode::F5 {
                        let state_msg = state.debug.toggle_player_neighbours();
                        state.notify(state_msg);
                    } else if code == KeyCode::F6 {
                        let state_msg = state.debug.toggle_occupied_cells();
                        state.notify(state_msg);
                    } else if code == KeyCode::F8 {
                        let state_msg = state.debug.toggle_tile_editor();
                        state.notify(state_msg);
                        if state.debug.show_tile_editor {
                            state.window.set_cursor(state.tile_cursor.clone());
                        } else {
                            state.window.set_cursor(CursorIcon::Default);
                        }
                    } else if code == KeyCode::F10 {
                        state.is_isometric = !state.is_isometric;
                        state.notify(format!(
                            "Isometric mode: {}",
                            if state.is_isometric { "ON" } else { "OFF" }
                        ));
                        state.scene.sync_camera_mode(state.is_isometric);
                    } else if code == KeyCode::F11 {
                        let state_msg = state.debug.toggle_player_collider();
                        state.notify(state_msg);
                    } else if let Some(action) =
                        actions::resolve_key_press(code, state.dialogue.is_some())
                    {
                        match action {
                            Action::AdvanceOrSkip => {
                                if let Some(dialogue) = &mut state.dialogue {
                                    let consumed = dialogue.consumes_entity();
                                    let flag = dialogue.sets_flag();
                                    if !dialogue.advance_or_skip() {
                                        if let Some(entity_id) = consumed {
                                            state.scene.consume_entity(entity_id);
                                        }
                                        if let Some(flag) = flag {
                                            state.progression.set(flag, true);
                                        }
                                        state.dialogue = None;
                                    }
                                }
                            }
                            Action::Interact => match state.scene.try_interact() {
                                Some(InteractResult::Dialogue(id, consumes_entity, sets_flag)) => {
                                    let lines = line_for(id, &state.progression);
                                    state.dialogue =
                                        Some(DialogueState::new(lines, consumes_entity, sets_flag));
                                }
                                Some(InteractResult::Toggle(entity_id)) => {
                                    state.scene.toggle_entity(entity_id);
                                }
                                None => {}
                            },
                        }
                    } else if code == KeyCode::KeyR
                        && (state.input.is_held(KeyCode::ControlLeft)
                            || state.input.is_held(KeyCode::ControlRight))
                    {
                        state.reset_scene();
                        state.notify(format!("Scene ({:?}) reset", state.scene.id));
                    } else if code == KeyCode::Numpad8 {
                        let msg = state.debug.increase_grid_cell_size();
                        state.notify(msg);
                    } else if code == KeyCode::Numpad2 {
                        let msg = state.debug.decrease_grid_cell_size();
                        state.notify(msg);
                    }
                    if state.debug.show_debug_info {
                        println!("{code:?} pressed");
                    }
                    state.input.press(code);
                }
                winit::event::ElementState::Released => {
                    state.input.release(code);
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                state.screen_mouse_position = (position.x, position.y);
            }
            WindowEvent::MouseInput {
                state: button_state,
                button: winit::event::MouseButton::Left,
                ..
            } => {
                state.left_mouse_down = button_state == winit::event::ElementState::Pressed;
            }
            _ => {}
        }
    }
}
