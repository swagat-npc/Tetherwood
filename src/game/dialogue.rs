/// Which visual/narrative voice a line is spoken in (ADR-021). Only
/// the two registers Beat 2 needs — NPC dialogue (avatar + standard
/// frame) is M5's job, once the village exists to speak it.
pub enum Register {
    Narrator,
    InnerMonologue,
}

pub struct ColoredSpan {
    pub text: &'static str,
    pub color: [f32; 4],
}

pub struct DialogueLine {
    pub spans: Vec<ColoredSpan>,
    pub register: Register,
}

const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const FEAR_RED: [f32; 4] = [0.9, 0.2, 0.2, 1.0];

/// Content lookup by interact-trigger id. Static match, no data files
/// (D-C) — content volume doesn't yet justify anything more. Each
/// entry is a short sequence, since a single interaction (Beat 2's
/// bed) mixes narrator and inner-monologue lines together.
pub fn line_for(id: &str) -> Vec<DialogueLine> {
    match id {
        // Throwaway placeholder lines - still need to flesh out the dialogue
        "bed_examine" => vec![
            DialogueLine {
                spans: vec![ColoredSpan {
                    text: "Her bed is still made.",
                    color: WHITE,
                }],
                register: Register::Narrator,
            },
            DialogueLine {
                spans: vec![ColoredSpan {
                    text: "She never came home last night.",
                    color: WHITE,
                }],
                register: Register::Narrator,
            },
            DialogueLine {
                spans: vec![ColoredSpan {
                    text: "...where are you?",
                    color: FEAR_RED,
                }],
                register: Register::InnerMonologue,
            },
        ],
        "necklace_examine" => vec![DialogueLine {
            spans: vec![ColoredSpan {
                text: "A silver necklace, half-hidden behind the pillow.",
                color: WHITE,
            }],
            register: Register::Narrator,
        }],
        _ => vec![DialogueLine {
            spans: vec![ColoredSpan {
                text: "...",
                color: WHITE,
            }],
            register: Register::Narrator,
        }],
    }
}
