use crate::game::progression::ProgressionTracker;

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
const UNEASE_TINT: [f32; 4] = [0.75, 0.75, 0.95, 1.0];

/// Content lookup by interact-trigger id. Static match, no data files
/// (D-C) — content volume doesn't yet justify anything more. Each
/// entry is a short sequence, since a single interaction (Beat 2's
/// bed) mixes narrator and inner-monologue lines together.
pub fn line_for(id: &str, progression: &ProgressionTracker) -> Vec<DialogueLine> {
    match id {
        "bed_examine" => vec![
            DialogueLine {
                spans: vec![ColoredSpan {
                    text: "The blanket's kicked back, sheets creased like she left in a hurry.",
                    color: WHITE,
                }],
                register: Register::Narrator,
            },
            DialogueLine {
                spans: vec![ColoredSpan {
                    text: "Rain hammers the shutters. The storm must have woken her.",
                    color: WHITE,
                }],
                register: Register::Narrator,
            },
            DialogueLine {
                spans: vec![ColoredSpan {
                    text: "She wouldn't just walk out into this. Not without a reason.",
                    color: WHITE,
                }],
                register: Register::InnerMonologue,
            },
            DialogueLine {
                spans: vec![ColoredSpan {
                    text: "No note. No sound of her coming back.",
                    color: WHITE,
                }],
                register: Register::Narrator,
            },
            DialogueLine {
                spans: vec![
                    ColoredSpan {
                        text: "...[Name]... ",
                        color: WHITE,
                    },
                    ColoredSpan {
                        text: "where are you?",
                        color: FEAR_RED,
                    },
                ],
                register: Register::InnerMonologue,
            },
        ],
        "necklace_examine" => vec![
            DialogueLine {
                spans: vec![ColoredSpan {
                    text: "A thin silver chain, caught on the headboard.",
                    color: WHITE,
                }],
                register: Register::Narrator,
            },
            DialogueLine {
                spans: vec![ColoredSpan {
                    text: "She never takes this off. Not even to sleep.",
                    color: WHITE,
                }],
                register: Register::Narrator,
            },
            DialogueLine {
                spans: vec![ColoredSpan {
                    text: "Why would she leave without it?",
                    color: WHITE,
                }],
                register: Register::InnerMonologue,
            },
            DialogueLine {
                spans: vec![ColoredSpan {
                    text: "It feels... heavier than it should.",
                    color: UNEASE_TINT,
                }],
                register: Register::InnerMonologue,
            },
        ],
        "villager_1_interact" => {
            if progression.is_set("necklace_consumed") {
                vec![DialogueLine {
                    spans: vec![ColoredSpan {
                        text: "Check your pockets.",
                        color: WHITE,
                    }],
                    register: Register::Narrator,
                }]
            } else {
                vec![DialogueLine {
                    spans: vec![ColoredSpan {
                        text: "Check behind the bed.",
                        color: WHITE,
                    }],
                    register: Register::Narrator,
                }]
            }
        }
        _ => vec![DialogueLine {
            spans: vec![ColoredSpan {
                text: "...",
                color: WHITE,
            }],
            register: Register::Narrator,
        }],
    }
}
