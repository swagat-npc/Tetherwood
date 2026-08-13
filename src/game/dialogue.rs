/// Content lookup by interact-trigger id. Static match, no data files
/// (D-C) — content volume doesn't yet justify anything more.
pub fn line_for(id: &str) -> &'static str {
    match id {
        "bed_examine" => "Her bed is still made. She never came home.",
        "necklace_examine" => "A silver necklace, half-hidden behind the bed.",
        "mouse_coordinate" => "Mouse Pos: ",
        _ => "...",
    }
}
