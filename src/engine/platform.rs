use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
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

use crate::engine::ids::SceneId;
use crate::engine::renderer::Renderer;
use crate::engine::scene::{CameraMode, Scene};
use crate::game::dialogue::Register;

struct DialogueState {
    lines: Vec<crate::game::dialogue::DialogueLine>,
    current_line: usize,
    /// How many bytes of the current line's text are currently
    /// visible. Grows over time (typewriter); jumping straight to
    /// the line's full length is what "skip" does.
    revealed_chars: usize,
    /// Accumulates delta time; a new character reveals once this
    /// crosses REVEAL_INTERVAL.
    reveal_timer: f32,
    /// Blink cycle for the "press to continue" caret — independent of
    /// reveal_timer, since blinking should run continuously once the
    /// line is fully revealed, not restart with each new line.
    caret_timer: f32,
}

const REVEAL_INTERVAL: f32 = 0.03;
const CARET_BLINK_INTERVAL: f32 = 0.5; // seconds per on/off half-cycle

/// A single revealed character, paired with the color it should
/// render in — flattened from DialogueLine's spans, so the renderer
/// doesn't need to know about span boundaries at all, just a flat
/// per-character color sequence.
struct RevealedChar {
    ch: char,
    color: [f32; 4],
}

impl DialogueState {
    fn new(lines: Vec<crate::game::dialogue::DialogueLine>) -> Self {
        Self {
            lines,
            current_line: 0,
            revealed_chars: 0,
            reveal_timer: 0.0,
            caret_timer: 0.0,
        }
    }

    fn full_len(&self) -> usize {
        self.lines
            .get(self.current_line)
            .map(|line| line.spans.iter().map(|s| s.text.chars().count()).sum())
            .unwrap_or(0)
    }

    fn char_at(&self, index: usize) -> Option<char> {
        self.lines
            .get(self.current_line)?
            .spans
            .iter()
            .flat_map(|s| s.text.chars())
            .nth(index)
    }

    /// Advances the typewriter reveal by `delta` seconds. Call once
    /// per frame while dialogue is active, regardless of input.
    fn tick(&mut self, delta: f32) -> bool {
        self.caret_timer += delta;

        if self.lines.get(self.current_line).is_none() {
            return false; // no active lines, nothing to reveal
        };
        let full_len = self.full_len();
        if self.revealed_chars >= full_len {
            return false; // line is fully revealed, nothing to do
        }

        self.reveal_timer += delta;
        let mut revealed_non_space = false;
        while self.reveal_timer >= REVEAL_INTERVAL && self.revealed_chars < full_len {
            self.reveal_timer -= REVEAL_INTERVAL;
            if self.char_at(self.revealed_chars) != Some(' ') {
                revealed_non_space = true;
            }
            self.revealed_chars += 1;
        }
        revealed_non_space
    }

    /// The E/Space press handler. Skips to full reveal if the current
    /// line isn't done typing; otherwise advances to the next line.
    /// Returns false once there are no more lines — the caller closes
    /// the dialogue on that signal.
    fn advance_or_skip(&mut self) -> bool {
        if self.lines.get(self.current_line).is_none() {
            return false; // no active lines, nothing to reveal
        };
        if self.revealed_chars < self.full_len() {
            self.revealed_chars = self.full_len();
            return true;
        }

        self.current_line += 1;
        self.revealed_chars = 0;
        self.reveal_timer = 0.0;
        self.caret_timer = 0.0;
        self.current_line < self.lines.len()
    }

    /// True during the "on" half of the blink cycle, and only once the
    /// current line is fully revealed — no caret while still typing,
    /// since there's nothing to advance to yet.
    fn caret_visible(&self) -> bool {
        if self.revealed_chars < self.full_len() {
            return false;
        }
        (self.caret_timer % (CARET_BLINK_INTERVAL * 2.0)) < CARET_BLINK_INTERVAL
    }

    /// Flattens the current line's spans into one per-character list,
    /// truncated to however many characters are currently revealed.
    fn visible_chars(&self) -> Vec<RevealedChar> {
        let Some(line) = self.lines.get(self.current_line) else {
            return Vec::new();
        };
        line.spans
            .iter()
            .flat_map(|span| {
                span.text.chars().map(move |ch| RevealedChar {
                    ch,
                    color: span.color,
                })
            })
            .take(self.revealed_chars)
            .collect()
    }

    fn current_register(&self) -> Option<&crate::game::dialogue::Register> {
        self.lines.get(self.current_line).map(|l| &l.register)
    }
}

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
    held_keys: HashSet<KeyCode>,
    multiplying_factor: f32, // TODO: migrate this to a config struct to supply everywhere
    show_colliders: bool,
    show_debug_info: bool,
    show_test_text: bool,
    dialogue: Option<DialogueState>,
    screen_mouse_position: (f64, f64),
    smoothed_fps: f32,
    audio: kira::AudioManager<kira::DefaultBackend>,
    audio_blip: Vec<StaticSoundData>,
    blip_step_counter: usize,
    blip_volume: f32,
}

impl AppState {
    /// Builds and GPU-prepares a scene. Free of `self` so it can run
    /// before AppState exists (resumed()'s first scene) as well as
    /// from change_scene, which just assigns the result afterward.
    fn build_scene(renderer: &mut Renderer, scene_id: SceneId, multiplying_factor: f32) -> Scene {
        let new_scene = match scene_id {
            SceneId::Home => {
                Scene::new_home(renderer.device(), renderer.queue(), multiplying_factor)
                    .expect("failed to build home scene")
            }
            SceneId::Outside => {
                Scene::new_outside(renderer.device(), renderer.queue(), multiplying_factor)
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
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // TODO: Top level stuct to always be in memory, give it a hashmap of scene or anything that is needed everytime
        // and just call it when needed
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

        let state = AppState {
            window,
            renderer,
            scene: initial_scene,
            last_frame: Instant::now(),
            frame_count: 0,
            held_keys: HashSet::new(),
            multiplying_factor,
            show_colliders: true, // DEBUG: set to true for debugging
            show_debug_info: false,
            show_test_text: false,
            dialogue: None,
            screen_mouse_position: (0.0, 0.0),
            smoothed_fps: 60.0,
            audio,
            audio_blip,
            blip_step_counter: 0,
            blip_volume: -24.0,
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

                if let Some(dialogue) = &mut state.dialogue {
                    if dialogue.tick(delta.as_secs_f32()) {
                        let step = state.blip_step_counter;
                        state.blip_step_counter = state.blip_step_counter.wrapping_add(1);
                        state.play_blip(step);
                    }
                }

                // TODO(engine): raw KeyCode handling here is content, not machinery (ADR-035).
                let speed = 80.0 * state.multiplying_factor; // pixels per second scaled up to the factor
                let mut movement = glam::Vec2::ZERO;
                if state.held_keys.contains(&KeyCode::KeyW) {
                    movement.y -= 1.0;
                }
                if state.held_keys.contains(&KeyCode::KeyS) {
                    movement.y += 1.0;
                }
                if state.held_keys.contains(&KeyCode::KeyA) {
                    movement.x -= 1.0;
                }
                if state.held_keys.contains(&KeyCode::KeyD) {
                    movement.x += 1.0;
                }
                if movement != glam::Vec2::ZERO {
                    if let Some(dir) = crate::engine::entity::Direction::from_movement(movement) {
                        state.scene.player_mut().facing = dir;
                    }
                    let delta_move = movement.normalize() * speed * delta.as_secs_f32();
                    state.scene.try_move_player(delta_move);
                    if let Some((target_scene, target_warp_id)) =
                        state.scene.check_triggers(state.show_debug_info)
                    {
                        state.change_scene(target_scene);
                        if let Some(spawn_position) = state.scene.activate_warp(target_warp_id) {
                            state.scene.player_mut().position = spawn_position;
                        }
                    }
                }
                state.scene.update_interact_prompts();

                let camera_target = match state.scene.camera_mode {
                    CameraMode::Static(anchor) => anchor,
                    CameraMode::Follow => state.scene.player().position,
                };
                state.renderer.camera_position = camera_target;

                match state.renderer.acquire_frame() {
                    Ok(Some(frame)) => {
                        state
                            .renderer
                            .render_scene(&frame, &state.scene, state.show_colliders);

                        // DEBUG::Screen Text
                        if state.show_test_text {
                            let glyphs = crate::engine::text::layout_text(
                                "The Quick Brown Fox, Jumped Over the Lazy Dog! With the new font @ == estäblished*",
                                glam::Vec2::new(20.0, 500.0),
                            );
                            state.renderer.render_text(&frame, &glyphs);
                        }
                        // HUD::Displayed Text
                        if let Some(dialogue) = &state.dialogue {
                            state
                                .renderer
                                .render_dialogue_panel(&frame, dialogue.current_register());
                            let text_pos = state.renderer.dialogue_text_position();
                            let colored_chars: Vec<(char, [f32; 4])> = dialogue
                                .visible_chars()
                                .into_iter()
                                .map(|rc| (rc.ch, rc.color))
                                .collect();
                            let glyphs =
                                crate::engine::text::layout_colored_text(&colored_chars, text_pos);
                            state.renderer.render_text(&frame, &glyphs);

                            if dialogue.caret_visible() {
                                let caret_pos = state.renderer.dialogue_caret_position();
                                let caret_glyphs = crate::engine::text::layout_colored_text_scaled(
                                    &[('▼', [1.0, 1.0, 1.0, 1.0])],
                                    caret_pos,
                                    crate::engine::text::DIALOGUE_TEXT_SCALE + 2.0,
                                );
                                state.renderer.render_text(&frame, &caret_glyphs);
                            }
                        }
                        if state.show_debug_info {
                            // DEBUG::FPS Counter
                            {
                                let fps_text = format!("FPS: {:.0}", state.smoothed_fps);
                                let glyphs = crate::engine::text::layout_text_scaled(
                                    &fps_text,
                                    glam::Vec2::new(10.0, 10.0),
                                    crate::engine::text::DEBUG_TEXT_SCALE,
                                );
                                // --- Render Background
                                state.renderer.render_text_bg(
                                    &glyphs,
                                    &frame,
                                    [0.0, 0.0, 0.0, 0.85],
                                    None,
                                    0.0,
                                    crate::engine::text::DEBUG_TEXT_PADDING,
                                );
                                // --- Render Text
                                state.renderer.render_text(&frame, &glyphs);
                            }

                            // DEBUG::Mouse Position
                            {
                                let screen_pos = glam::Vec2::new(
                                    state.screen_mouse_position.0 as f32,
                                    state.screen_mouse_position.1 as f32,
                                );
                                let world_pos = state.renderer.screen_to_world(screen_pos);
                                let authoring_pos = world_pos / state.multiplying_factor;
                                let mouse_text = format!(
                                    "Mouse Pos: {:.0}, {:.0}",
                                    authoring_pos.x, authoring_pos.y
                                );
                                let screen_size = state.renderer.screen_size();
                                let mouse_text_pos = glam::Vec2::new(
                                    crate::engine::text::DEBUG_TEXT_PADDING,
                                    screen_size.y - 25.0,
                                );
                                let glyphs = crate::engine::text::layout_text_scaled(
                                    &mouse_text,
                                    mouse_text_pos,
                                    crate::engine::text::DEBUG_TEXT_SCALE,
                                );
                                // --- Render Background
                                state.renderer.render_text_bg(
                                    &glyphs,
                                    &frame,
                                    [0.0, 0.0, 0.0, 0.85],
                                    None,
                                    0.0,
                                    crate::engine::text::DEBUG_TEXT_PADDING,
                                );
                                // --- Render Text
                                state.renderer.render_text(&frame, &glyphs);
                            }
                        }
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
                    } else if code == KeyCode::F2 {
                        state.show_debug_info = !state.show_debug_info;
                        println!(
                            "{} Debug Info",
                            if state.show_debug_info {
                                "Show"
                            } else {
                                "Hide"
                            }
                        );
                    } else if code == KeyCode::F3 {
                        state.show_test_text = !state.show_test_text;
                        println!(
                            "{} Test Text",
                            if state.show_test_text { "Show" } else { "Hide" }
                        );
                    } else if code == KeyCode::KeyE || code == KeyCode::Space {
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
                        } else if code == KeyCode::KeyE {
                            match state.scene.try_interact() {
                                Some(crate::engine::scene::InteractResult::Dialogue(
                                    id,
                                    consumes_entity,
                                )) => {
                                    let mut lines = crate::game::dialogue::line_for(id);
                                    if let (Some(last), Some(entity_id)) =
                                        (lines.last_mut(), consumes_entity)
                                    {
                                        last.consumes_entity = Some(entity_id);
                                    }
                                    state.dialogue = Some(DialogueState::new(lines));
                                }
                                Some(crate::engine::scene::InteractResult::Toggle(entity_id)) => {
                                    state.scene.toggle_entity(entity_id);
                                }
                                None => {}
                            }
                        }
                    } else if code == KeyCode::KeyR
                        && (state.held_keys.contains(&KeyCode::ControlLeft)
                            || state.held_keys.contains(&KeyCode::ControlRight))
                    {
                        state.reset_scene();
                    }
                    if state.show_debug_info {
                        println!("{code:?} pressed");
                    }
                    state.held_keys.insert(code);
                }
                winit::event::ElementState::Released => {
                    state.held_keys.remove(&code);
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                state.screen_mouse_position = (position.x, position.y);
            }
            _ => {}
        }
    }
}
