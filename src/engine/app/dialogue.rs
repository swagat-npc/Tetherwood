use crate::engine::app::AppState;
use crate::{
    engine::entity::EntityId,
    game::dialogue::{DialogueLine, Register},
};
use kira::sound::static_sound::StaticSoundSettings;

pub(super) struct DialogueState {
    pub(super) lines: Vec<DialogueLine>,
    pub(super) current_line: usize,
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
    consumes_entity: Option<EntityId>,
    sets_flag: Option<&'static str>,
}

const REVEAL_INTERVAL: f32 = 0.03;
const CARET_BLINK_INTERVAL: f32 = 0.5; // seconds per on/off half-cycle

/// A single revealed character, paired with the color it should
/// render in — flattened from DialogueLine's spans, so the renderer
/// doesn't need to know about span boundaries at all, just a flat
/// per-character color sequence.
pub(super) struct RevealedChar {
    pub(super) ch: char,
    pub(super) color: [f32; 4],
}

impl DialogueState {
    pub(super) fn new(
        lines: Vec<DialogueLine>,
        consumes_entity: Option<EntityId>,
        sets_flag: Option<&'static str>,
    ) -> Self {
        Self {
            lines,
            current_line: 0,
            revealed_chars: 0,
            reveal_timer: 0.0,
            caret_timer: 0.0,
            consumes_entity,
            sets_flag,
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
    pub(super) fn tick(&mut self, delta: f32) -> bool {
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
    pub(super) fn advance_or_skip(&mut self) -> bool {
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
    pub(super) fn caret_visible(&self) -> bool {
        if self.revealed_chars < self.full_len() {
            return false;
        }
        (self.caret_timer % (CARET_BLINK_INTERVAL * 2.0)) < CARET_BLINK_INTERVAL
    }

    /// Flattens the current line's spans into one per-character list,
    /// truncated to however many characters are currently revealed.
    pub(super) fn visible_chars(&self) -> Vec<RevealedChar> {
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

    pub(super) fn current_register(&self) -> Option<&Register> {
        self.lines.get(self.current_line).map(|l| &l.register)
    }

    pub(super) fn consumes_entity(&self) -> Option<EntityId> {
        self.consumes_entity
    }

    pub(super) fn sets_flag(&self) -> Option<&'static str> {
        self.sets_flag
    }
}

impl AppState {
    const BLIP_PITCH_STEPS: [f64; 4] = [0.95, 1.05, 1.0, 1.1]; // semitone-ish multipliers, cycling
    // const BLIP_PITCH_STEPS: [f64; 4] = [0.5, 1.0, 1.5, 2.0]; // more variation in pitch shift

    fn play_blip(&mut self, step_index: usize) {
        let Some(dialogue) = self.dialogue.as_mut() else {
            return;
        };
        let sound_data = match dialogue.current_register() {
            Some(crate::game::dialogue::Register::InnerMonologue) => &self.audio_blip[1],
            _ => &self.audio_blip[0],
        };
        let pitch = Self::BLIP_PITCH_STEPS[step_index % Self::BLIP_PITCH_STEPS.len()];
        let settings = StaticSoundSettings::new()
            .playback_rate(kira::PlaybackRate::from(pitch))
            .volume(kira::Decibels::from(self.blip_volume));
        let _ = self.audio.play(sound_data.clone().with_settings(settings));
    }

    pub fn tick_dialogue(&mut self, delta: f32) {
        if let Some(dialogue) = &mut self.dialogue {
            if dialogue.tick(delta) {
                let step = self.blip_step_counter;
                self.blip_step_counter = self.blip_step_counter.wrapping_add(1);
                self.play_blip(step);
            }
        }
    }
}
