mod dialogue;
pub mod input;

use super::debug::DebugSettings;
use crate::engine::debug::hud;
use crate::engine::debug::notifications::Notification;
use crate::engine::debug::ui::Slider;
use crate::engine::entity::Direction;
use crate::engine::renderer::{Frame, Renderer, text};
use crate::engine::scene::{CameraMode, InteractResult, Scene, SceneId};
use crate::game::actions::{self, Action};
use crate::game::dialogue::{Register, line_for};
use crate::game::progression::ProgressionTracker;
use crate::game::scenes::{home, village};
use dialogue::DialogueState;
use glam::Vec2;
use input::InputState;
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use pollster::block_on;
use std::time::Duration;
use std::{sync::Arc, time::Instant};
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
    smoothed_fps: f32,
    audio: kira::AudioManager<kira::DefaultBackend>,
    audio_blip: Vec<StaticSoundData>,
    blip_step_counter: usize,
    blip_volume: f32,
    notifications: Vec<Notification>,
    left_mouse_down: bool,
    debug: DebugSettings,
    volume_slider: Slider,
    is_isometric: bool,
    progression: ProgressionTracker,
}

impl AppState {
    /// Builds and GPU-prepares a scene. Free of `self` so it can run
    /// before AppState exists (resumed()'s first scene) as well as
    /// from change_scene, which just assigns the result afterward.
    fn build_scene(
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
        };
        new_scene.build_static_grid(multiplying_factor);
        renderer.prepare_scene(&new_scene);
        new_scene
    }

    fn change_scene(&mut self, scene_id: SceneId) {
        self.scene = Self::build_scene(
            &mut self.renderer,
            scene_id,
            self.multiplying_factor,
            self.is_isometric,
            &mut self.progression,
        );
    }

    fn reset_scene(&mut self) {
        self.scene = Self::build_scene(
            &mut self.renderer,
            self.scene.id,
            self.multiplying_factor,
            self.is_isometric,
            &mut self.progression,
        );
    }

    fn notify(&mut self, message: impl Into<String>) {
        self.notifications.push(Notification {
            message: message.into(),
            duration: Duration::from_secs(2),
            start_time: Instant::now(),
        })
    }

    const BLIP_PITCH_STEPS: [f64; 4] = [0.95, 1.05, 1.0, 1.1]; // semitone-ish multipliers, cycling
    // const BLIP_PITCH_STEPS: [f64; 4] = [0.5, 1.0, 1.5, 2.0]; // more variation in pitch shift

    fn play_blip(&mut self, step_index: usize) {
        let Some(dialogue) = self.dialogue.as_mut() else {
            return;
        };
        let sound_data = match dialogue.current_register() {
            Some(Register::InnerMonologue) => &self.audio_blip[1],
            _ => &self.audio_blip[0],
        };
        let pitch = Self::BLIP_PITCH_STEPS[step_index % Self::BLIP_PITCH_STEPS.len()];
        let settings = StaticSoundSettings::new()
            .playback_rate(kira::PlaybackRate::from(pitch))
            .volume(kira::Decibels::from(self.blip_volume));
        let _ = self.audio.play(sound_data.clone().with_settings(settings));
    }

    fn tick_dialogue(&mut self, delta: f32) {
        if let Some(dialogue) = &mut self.dialogue {
            if dialogue.tick(delta) {
                let step = self.blip_step_counter;
                self.blip_step_counter = self.blip_step_counter.wrapping_add(1);
                self.play_blip(step);
            }
        }
    }

    fn update_player(&mut self, delta: f32) {
        if self.dialogue.is_some() {
            return;
        }
        let speed = 80.0 * self.multiplying_factor;
        let movement = actions::resolve_movement(&self.input, self.is_isometric);
        if movement != Vec2::ZERO {
            // TODO: Direction::from_movement doesn't account for the isometric
            // movement table's diagonal/cardinal split - facing may be wrong in
            // isometric mode. Deferred until facing-while-isometric is a real need.
            if let Some(dir) = Direction::from_movement(movement) {
                self.scene.player_mut().facing = dir;
            }
            let delta_move = movement * speed * delta;
            self.scene
                .try_move_player(delta_move, self.multiplying_factor);
            if let Some((target_scene, target_warp_id)) =
                self.scene.check_triggers(self.debug.show_debug_info)
            {
                self.change_scene(target_scene);
                if let Some(spawn_position) = self.scene.activate_warp(target_warp_id) {
                    self.scene.player_mut().position = spawn_position;
                }
            }
        }
        self.scene.update_interact_prompts();

        let camera_target = match self.scene.camera_mode() {
            CameraMode::Static(anchor) => anchor,
            CameraMode::Follow => self.scene.player().position,
        };
        self.renderer.camera_position = camera_target;
    }

    fn draw_hud(&mut self, frame: &Frame) {
        // DEBUG::Notifications Text
        hud::draw_notifications(&mut self.renderer, frame, &mut self.notifications);

        // HUD::Displayed Text
        if let Some(dialogue) = &self.dialogue {
            self.renderer
                .render_dialogue_panel(frame, dialogue.current_register());
            let text_pos = self.renderer.dialogue_text_position();
            let max_width = self.renderer.dialogue_text_max_width();

            let colored_chars: Vec<(char, [f32; 4])> = dialogue
                .visible_chars()
                .into_iter()
                .map(|rc| (rc.ch, rc.color))
                .collect();

            let wrapped_lines =
                text::wrap_colored_text(&colored_chars, max_width, text::DIALOGUE_TEXT_SCALE);

            let line_height = text::GLYPH_SIZE.y * text::DIALOGUE_TEXT_SCALE + 4.0;
            for (i, line_chars) in wrapped_lines.iter().enumerate() {
                let line_origin = text_pos + Vec2::new(0.0, i as f32 * line_height);
                let glyphs = text::layout_colored_text_scaled(
                    line_chars,
                    line_origin,
                    text::DIALOGUE_TEXT_SCALE,
                );
                self.renderer.render_text(frame, &glyphs);
            }

            if dialogue.caret_visible() {
                let caret_pos = self.renderer.dialogue_caret_position();
                let caret_glyphs = text::layout_colored_text_scaled(
                    &[('▼', [1.0, 1.0, 1.0, 1.0])],
                    caret_pos,
                    text::DIALOGUE_TEXT_SCALE + 2.0,
                );
                self.renderer.render_text(frame, &caret_glyphs);
            }
        }

        if self.debug.show_debug_info {
            // DEBUG::FPS Counter
            hud::draw_fps_counter(&mut self.renderer, frame, self.smoothed_fps);

            // DEBUG::Mouse Position
            hud::draw_mouse_position(
                &mut self.renderer,
                frame,
                self.screen_mouse_position,
                self.multiplying_factor,
            );
        }

        if self.debug.show_debug_renderer {
            // DEBUG:: Volume slider
            let world_mouse = Vec2::new(
                self.screen_mouse_position.0 as f32,
                self.screen_mouse_position.1 as f32,
            );
            if self.volume_slider.update(world_mouse, self.left_mouse_down) {
                self.blip_volume = self.volume_slider.value;
            }
            let slider_rects = self.volume_slider.build_rects();
            hud::draw_slider(&mut self.renderer, frame, &slider_rects);
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

        let window_attributes = Window::default_attributes()
            .with_title("Tetherwood")
            .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
            .with_position(winit::dpi::LogicalPosition::new(0.0, 100.0));
        let window = event_loop.create_window(window_attributes).unwrap();
        let window = Arc::new(window);

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

        let volume_slider = Slider::new(
            Vec2::new(700.0, 30.0),
            Vec2::new(120.0, 12.0),
            -40.0,
            0.0,
            -24.0, // matches blip_volume's current default
        );

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
            smoothed_fps: 60.0,
            audio,
            audio_blip,
            blip_step_counter: 0,
            blip_volume: -24.0,
            notifications: Vec::new(),
            left_mouse_down: false,
            volume_slider,
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
                let now = Instant::now();
                let delta = now - state.last_frame;
                state.last_frame = now;

                let instantaneous_fps = 1.0 / delta.as_secs_f32();
                // Exponential moving average — each new sample nudges the displayed
                // value rather than replacing it outright, smoothing out single-frame
                // jitter (OS scheduling noise, etc.) without the update-lag of a
                // fixed skip-interval.
                state.smoothed_fps = state.smoothed_fps * 0.9 + instantaneous_fps * 0.1;

                state.frame_count += 1;
                if state.frame_count.is_multiple_of(60) {
                    println!(
                        "delta: {:.2?} (~{:.0} fps)",
                        delta,
                        1.0 / delta.as_secs_f32()
                    );
                }

                state.tick_dialogue(delta.as_secs_f32());
                state.update_player(delta.as_secs_f32());

                match state.renderer.acquire_frame() {
                    Ok(Some(frame)) => {
                        state.renderer.render_scene(
                            &frame,
                            &state.scene,
                            &state.debug,
                            state.is_isometric,
                            state.multiplying_factor,
                        );
                        state.draw_hud(&frame);
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
                    } else if code == KeyCode::F10 {
                        state.is_isometric = !state.is_isometric;
                        state.notify(format!(
                            "Isometric mode: {}",
                            if state.is_isometric { "ON" } else { "OFF" }
                        ));
                        state.scene.sync_camera_mode(state.is_isometric);
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
