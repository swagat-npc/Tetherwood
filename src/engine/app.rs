mod dialogue;
pub mod input;
mod layers;
mod player;
mod scene_lifecycle;

use super::debug::DebugSettings;
use crate::engine::debug::inspector::Inspector;
use crate::engine::debug::notifications::Notification;
use crate::engine::entity;
use crate::engine::renderer::tile::{PaintMode, PaintState, TileEntry, TileGrid, TilemapFile};
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
use winit::event::MouseScrollDelta;
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
    left_mouse_pressed: bool,
    debug: DebugSettings,
    inspector: Inspector,
    is_isometric: bool,
    progression: ProgressionTracker,
    paint: PaintState,
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

    fn current_scene_id(&self) -> SceneId {
        self.scene.id
    }

    fn world_mouse_position(&self) -> Vec2 {
        self.renderer.screen_to_world(
            Vec2::new(
                self.screen_mouse_position.0 as f32,
                self.screen_mouse_position.1 as f32,
            ),
            self.is_isometric,
        )
    }

    fn tilemap_path(id: SceneId) -> String {
        match id {
            SceneId::Home => "assets/tile/home.tilemap".to_string(),
            SceneId::Village => "assets/tile/village.tilemap".to_string(),
            SceneId::Sandbox => "assets/tile/debug.tilemap".to_string(),
        }
    }

    fn reset_paint_session(&mut self) {
        self.paint.session = Some(self.scene.tile_grid.clone());
    }

    pub fn save_paint_session(&mut self) {
        let Some(session) = &self.paint.session else {
            return;
        };
        let names = tile::tile_names();

        let tiles = session
            .iter()
            .filter_map(|((x, y, layer_id), atlas_cell)| {
                TileGrid::tile_name_for(atlas_cell, &names).map(|name| TileEntry {
                    x,
                    y,
                    layer_id,
                    tile_name: name.to_string(),
                })
            })
            .collect();

        let file = TilemapFile {
            layers: self.paint.layers.clone(),
            tiles,
        };
        let contents = ron::ser::to_string_pretty(&file, ron::ser::PrettyConfig::default())
            .expect("failed to serialize tilemap");

        let path = Self::tilemap_path(self.current_scene_id());
        if let Some(dir) = std::path::Path::new(&path).parent() {
            let _ = std::fs::create_dir_all(dir);
        }

        match std::fs::write(&path, contents) {
            Ok(()) => {
                self.scene.tile_grid = self.paint.session.clone().unwrap();
                self.notify("Tilemap saved");
            }
            Err(e) => self.notify(format!("Failed to save tilemap: {e}")),
        }
    }

    pub fn load_tilemap(scene: &mut Scene, paint: &mut PaintState) {
        let path = Self::tilemap_path(scene.id);
        let names = tile::tile_names();

        match std::fs::read_to_string(&path) {
            Ok(contents) => {
                let file: TilemapFile = ron::from_str(&contents)
                    .unwrap_or_else(|e| panic!("malformed tilemap {path}: {e}"));
                paint.layers = file.layers;
                for entry in file.tiles {
                    scene.tile_grid.set_named(
                        (entry.x, entry.y),
                        entry.layer_id,
                        &entry.tile_name,
                        &names,
                    );
                }
            }
            Err(_) => paint.layers = PaintState::default_layers(),
        }

        // current_layer_id must always point at a layer that actually
        // exists in whatever layer set this scene just ended up with -
        // covers both the no-file case and a loaded file whose layers
        // don't happen to include the previous scene's current layer.
        if !paint.layers.iter().any(|l| l.id == paint.current_layer_id) {
            paint.current_layer_id = paint.layers.first().map(|l| l.id).unwrap_or(0);
        }
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

        let is_isometric = true;
        let mut initial_paint = PaintState::new();
        let initial_scene = AppState::build_scene(
            &mut renderer,
            SceneId::Home,
            multiplying_factor,
            is_isometric,
            &mut progression,
            &mut initial_paint,
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

        let inspector = Inspector::new(renderer.screen_size(), &renderer.ttf_glyphs);

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
            left_mouse_pressed: false,
            inspector,
            debug: DebugSettings::new(),
            is_isometric,
            progression,
            paint: initial_paint,
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
                let screen_size = Vec2::new(size.width as f32, size.height as f32);
                state
                    .inspector
                    .recompute_layout(screen_size, &state.renderer.ttf_glyphs);
            }
            WindowEvent::RedrawRequested => {
                let delta = state.tick_frame_timing();
                state.tick_dialogue(delta);
                state.update_player(delta);

                let player_position = state.scene.player().position;
                state
                    .renderer
                    .update_smoothed_camera(player_position, state.is_isometric);
                state.inspector.update();

                let frame = state.renderer.acquire_frame();
                // TODO: display frame time
                match frame {
                    Ok(Some(frame)) => {
                        state.renderer.clear_frame(&frame);

                        // Tile Layer
                        let active_grid = state
                            .paint
                            .session
                            .as_ref()
                            .unwrap_or(&state.scene.tile_grid);

                        let is_paint_active = state.debug.show_tile_editor;
                        let (hover_cell, ghost_atlas_cell) = if is_paint_active {
                            let world_pos = state.world_mouse_position();
                            let cell = tile::cell_at_position(world_pos, state.multiplying_factor);
                            let selected = state.inspector.selected_tile().unwrap_or((0, 0));
                            (
                                Some((cell.0, cell.1, state.paint.current_layer_id)),
                                selected,
                            )
                        } else {
                            (None, (0, 0))
                        };

                        let mut hover_cell_found = false;
                        let mut tile_entries: Vec<(Vec2, (i32, i32), f32)> = active_grid
                            .iter()
                            .filter_map(|(cell, atlas_cell)| {
                                let draw_cell = match state.paint.mode {
                                    PaintMode::Place => {
                                        if Some(cell) == hover_cell {
                                            hover_cell_found = true;
                                            Some(ghost_atlas_cell)
                                        } else {
                                            Some(atlas_cell)
                                        }
                                    }
                                    PaintMode::Remove => {
                                        if Some(cell) == hover_cell {
                                            None
                                        } else {
                                            Some(atlas_cell)
                                        }
                                    }
                                };
                                if let Some(draw_cell) = draw_cell {
                                    Some(tile::layered_tile_entry(
                                        cell,
                                        draw_cell,
                                        state.multiplying_factor,
                                        state.is_isometric,
                                        &state.renderer,
                                    ))
                                } else {
                                    None
                                }
                            })
                            .collect();

                        if let Some(cell) = hover_cell {
                            if !hover_cell_found && let PaintMode::Place = state.paint.mode {
                                tile_entries.push(tile::layered_tile_entry(
                                    cell,
                                    ghost_atlas_cell,
                                    state.multiplying_factor,
                                    state.is_isometric,
                                    &state.renderer,
                                ));
                            }
                        }
                        tile_entries.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

                        let projection = state.renderer.world_projection();
                        let sprite_camera_view =
                            state.renderer.sprite_camera_view(state.is_isometric);
                        state.renderer.render_tiles(
                            &frame,
                            &tile_entries,
                            projection,
                            sprite_camera_view,
                            state.multiplying_factor,
                            state.is_isometric,
                        );

                        // Texture Layer
                        state.renderer.draw_background_and_entities(
                            &frame,
                            &state.scene,
                            state.is_isometric,
                        );

                        // Debug Rect Layer
                        state.renderer.draw_debug_geometry(
                            &frame,
                            &state.scene,
                            &state.debug,
                            state.is_isometric,
                            state.multiplying_factor,
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
                            state.reset_paint_session();
                        } else {
                            state.window.set_cursor(CursorIcon::Default);
                            state.paint.session = None;
                        }
                    } else if code == KeyCode::F10 {
                        state.is_isometric = !state.is_isometric;
                        state.notify(format!(
                            "Isometric mode: {}",
                            if state.is_isometric { "ON" } else { "OFF" }
                        ));
                        state.refresh_scene();
                        state.scene.sync_camera_mode(state.is_isometric);
                    } else if code == KeyCode::F11 {
                        let state_msg = state.debug.toggle_player_collider();
                        state.notify(state_msg);
                    } else if code == KeyCode::F12 {
                        state.inspector.toggle();
                        let msg = if state.inspector.is_hidden {
                            "Inspector: hidden"
                        } else {
                            "Inspector: visible"
                        };
                        state.notify(msg);
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
                let was_down = state.left_mouse_down;
                state.left_mouse_down = button_state == winit::event::ElementState::Pressed;
                state.left_mouse_pressed = state.left_mouse_down && !was_down;
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_amount = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.01,
                };
                let mouse_pos = Vec2::new(
                    state.screen_mouse_position.0 as f32,
                    state.screen_mouse_position.1 as f32,
                );
                let pixels_per_line = 32.0; // px/line, tune by feel
                if entity::point_in_rect(mouse_pos, &state.inspector.bounds()) {
                    state
                        .inspector
                        .scroll(scroll_amount * pixels_per_line, &state.renderer.ttf_glyphs);
                } else {
                    state.renderer.zoom =
                        (state.renderer.zoom + scroll_amount * 0.1).clamp(0.5, 3.0);
                }
            }
            _ => {}
        }
    }
}
