mod dialogue;

use dialogue::DialogueState;
use glam::Vec2;
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use pollster::block_on;
use std::{sync::Arc, time::Instant};
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::engine::debug::notifications::Notification;
use crate::engine::entity::Direction;
use crate::engine::ids::SceneId;
use crate::engine::input::InputState;
use crate::engine::renderer::{Frame, Renderer};
use crate::engine::scene::{CameraMode, InteractResult, Scene};
use crate::engine::text;
use crate::engine::ui::Slider;
use crate::game::actions::{self, Action};
use crate::game::dialogue::{Register, line_for};
use crate::game::scenes::{home, outside};

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
    show_colliders: bool,
    show_debug_info: bool,
    dialogue: Option<DialogueState>,
    screen_mouse_position: (f64, f64),
    smoothed_fps: f32,
    audio: kira::AudioManager<kira::DefaultBackend>,
    audio_blip: Vec<StaticSoundData>,
    blip_step_counter: usize,
    blip_volume: f32,
    notifications: Vec<Notification>,
    left_mouse_down: bool,
    volume_slider: Slider,
}

impl AppState {
    /// Builds and GPU-prepares a scene. Free of `self` so it can run
    /// before AppState exists (resumed()'s first scene) as well as
    /// from change_scene, which just assigns the result afterward.
    fn build_scene(renderer: &mut Renderer, scene_id: SceneId, multiplying_factor: f32) -> Scene {
        let new_scene = match scene_id {
            SceneId::Home => home::build(renderer.device(), renderer.queue(), multiplying_factor)
                .expect("failed to build home scene"),
            SceneId::Outside => {
                outside::build(renderer.device(), renderer.queue(), multiplying_factor)
                    .expect("failed to build outside scene")
            }
        };
        renderer.prepare_scene(&new_scene);
        new_scene
    }

    fn change_scene(&mut self, scene_id: SceneId) {
        self.scene = Self::build_scene(&mut self.renderer, scene_id, self.multiplying_factor);
    }

    fn reset_scene(&mut self) {
        self.scene = Self::build_scene(&mut self.renderer, self.scene.id, self.multiplying_factor);
    }

    const BLIP_PITCH_STEPS: [f64; 4] = [0.95, 1.05, 1.0, 1.1]; // semitone-ish multipliers, cycling
    // const BLIP_PITCH_STEPS: [f64; 4] = [0.5, 1.0, 1.5, 2.0];
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
        let speed = 80.0 * self.multiplying_factor;
        let movement = actions::resolve_movement(&self.input);
        if movement != Vec2::ZERO {
            if let Some(dir) = Direction::from_movement(movement) {
                self.scene.player_mut().facing = dir;
            }
            let delta_move = movement.normalize() * speed * delta;
            self.scene.try_move_player(delta_move);
            if let Some((target_scene, target_warp_id)) =
                self.scene.check_triggers(self.show_debug_info)
            {
                self.change_scene(target_scene);
                if let Some(spawn_position) = self.scene.activate_warp(target_warp_id) {
                    self.scene.player_mut().position = spawn_position;
                }
            }
        }
        self.scene.update_interact_prompts();

        let camera_target = match self.scene.camera_mode {
            CameraMode::Static(anchor) => anchor,
            CameraMode::Follow => self.scene.player().position,
        };
        self.renderer.camera_position = camera_target;
    }

    fn draw_hud(&mut self, frame: &Frame) {
        // DEBUG::Notifications Text
        if !self.notifications.is_empty() {
            let screen_size = self.renderer.screen_size();
            let notification_height =
                text::GLYPH_SIZE.y * text::DEBUG_TEXT_SCALE + text::DEBUG_TEXT_PADDING * 2.0;
            let notification_gap = 10.0;
            for (i, notification) in self.notifications.iter().enumerate() {
                let origin = text::centered_text_origin(
                    &notification.message,
                    screen_size.x * 0.5,
                    screen_size.y - 25.0 - i as f32 * (notification_height + notification_gap),
                    text::DEBUG_TEXT_SCALE,
                );
                let glyphs =
                    text::layout_text_scaled(&notification.message, origin, text::DEBUG_TEXT_SCALE);
                self.renderer.render_text_with_bg(frame, &glyphs);
            }
            self.notifications.retain(|n| !n.expired());
        }
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
        if self.show_debug_info {
            // DEBUG::FPS Counter
            {
                let fps_text = format!("FPS: {:.0}", self.smoothed_fps);
                let glyphs = text::layout_text_scaled(
                    &fps_text,
                    Vec2::new(10.0, 10.0),
                    text::DEBUG_TEXT_SCALE,
                );
                self.renderer.render_text_with_bg(frame, &glyphs);
            }

            // DEBUG::Mouse Position
            {
                let screen_pos = Vec2::new(
                    self.screen_mouse_position.0 as f32,
                    self.screen_mouse_position.1 as f32,
                );
                let world_pos = self.renderer.screen_to_world(screen_pos);
                let authoring_pos = world_pos / self.multiplying_factor;
                let mouse_text =
                    format!("Mouse Pos: {:.0}, {:.0}", authoring_pos.x, authoring_pos.y);
                let screen_size = self.renderer.screen_size();
                let mouse_text_pos = Vec2::new(text::DEBUG_TEXT_PADDING, screen_size.y - 25.0);
                let glyphs =
                    text::layout_text_scaled(&mouse_text, mouse_text_pos, text::DEBUG_TEXT_SCALE);
                self.renderer.render_text_with_bg(frame, &glyphs);
            }

            // DEBUG:: Volume slider
            {
                let world_mouse = Vec2::new(
                    self.screen_mouse_position.0 as f32,
                    self.screen_mouse_position.1 as f32,
                );
                if self.volume_slider.update(world_mouse, self.left_mouse_down) {
                    self.blip_volume = self.volume_slider.value;
                }
                let slider_rects = self.volume_slider.build_rects();
                let projection = self.renderer.screen_projection();

                self.renderer.render_solid_rects(
                    frame,
                    &slider_rects,
                    projection,
                    glam::Mat4::IDENTITY,
                );
            }
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

        let initial_scene = AppState::build_scene(&mut renderer, SceneId::Home, multiplying_factor);

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
            show_colliders: true, // DEBUG: set to true for debugging
            show_debug_info: false,
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
                        state
                            .renderer
                            .render_scene(&frame, &state.scene, state.show_colliders);
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
                        state.show_colliders = !state.show_colliders;
                        println!(
                            "{} Colliders",
                            if state.show_colliders { "Show" } else { "Hide" }
                        );
                    } else if code == KeyCode::F3 {
                        state.show_debug_info = !state.show_debug_info;
                        println!(
                            "{} Debug Info",
                            if state.show_debug_info {
                                "Show"
                            } else {
                                "Hide"
                            }
                        );
                    } else if let Some(action) =
                        actions::resolve_key_press(code, state.dialogue.is_some())
                    {
                        match action {
                            Action::AdvanceOrSkip => {
                                if let Some(dialogue) = &mut state.dialogue {
                                    let consumed = dialogue
                                        .lines
                                        .get(dialogue.current_line)
                                        .and_then(|l| l.consumes_entity);
                                    if !dialogue.advance_or_skip() {
                                        if let Some(entity_id) = consumed {
                                            state.scene.consume_entity(entity_id);
                                        }
                                        state.dialogue = None;
                                    }
                                }
                            }
                            Action::Interact => match state.scene.try_interact() {
                                Some(InteractResult::Dialogue(id, consumes_entity)) => {
                                    let mut lines = line_for(id);
                                    if let (Some(last), Some(entity_id)) =
                                        (lines.last_mut(), consumes_entity)
                                    {
                                        last.consumes_entity = Some(entity_id);
                                    }
                                    state.dialogue = Some(DialogueState::new(lines));
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
                        state.notifications.push(Notification {
                            message: format!("Scene ({:?}) reset", state.scene.id),
                            duration: std::time::Duration::from_secs(2),
                            start_time: Instant::now(),
                        });
                    }
                    if state.show_debug_info {
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
