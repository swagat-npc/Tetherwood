# Derivation Pass — From Beats v2 to a Buildable Plan

**Document type:** Design→requirements derivation (docs-as-code, lives in `docs/`)
**Input:** Project Log v2, Beats v2 (Phase 7)
**Output:** Feature inventory → engine/game system split → Rust concept map → milestones → 30/45-day plan
**Status:** v1

---

## 0. Method

Each beat is ground into the features it demands. Features are sorted by the charter's machinery/content test ("could a different game reuse this code unchanged?") into **engine systems** (proto-engine, lives in `src/engine/`) and **game content** (lives in `src/game/`). Engine systems are then mapped to the Rust concepts they will force, in the order the build forces them. Milestones are cut so that every one ends with something visible on screen.

Scope tags: 🔷 slice · 🔶 Phase 1 thickening · ⬜ later phase.

---

## 1. Decisions Forced by This Pass (ADR candidates for Log v3)

| # | Decision | Rationale | Consequence |
|---|---|---|---|
| D-A | **No ECS for the slice.** Simple structs + plain collections; entities addressed by index/ID, not references. | ~12 live entities; ECS solves absent problems. The borrow-checker fight over cross-entity references IS the curriculum — adopting ECS first outsources the lesson. | ECS re-evaluated in Phase 1 with real experience. Expect and welcome the "indices, not references" refactor. |
| D-B | **Single crate.** One binary crate with `src/engine/` and `src/game/` modules. Workspace split deferred until extraction is real. | Honors "don't over-engineer repo structure." Module boundaries enforce the discipline; crates can follow later as a folder move. | The future `refactor(engine): extract …` commits will narrate the extraction. |
| D-C | **Static Rust data for slice content.** NPC tables, dialogue lines, enemy stats as `const`/`static` structures in code. No serde/RON yet. | Data-driven ≠ file-driven. Machinery consumes tables; the table's location is a detail. Defers a dependency and a format decision. | serde/RON evaluated at Phase 1 thickening when content volume hurts. |
| D-D | **Authored scenes, not an isometric tile engine.** Scene = hand-authored background image + collision rectangles + y-sorted entity sprites. | The isometric look is art direction, not math. Classic adventure games shipped exactly this way. Deletes tilemap formats, projection math, and tooling from slice scope. | Renderer's slice job reduces to: textured quads, draw order, camera. Isometric tilemaps become a ⬜ Phase 2 evaluation if ever needed. |
| D-E | **Time model:** variable frame delta for overworld movement; accumulator-driven fixed tick (~0.5s) inside battle for enemy actions; player input sampled every frame everywhere. | Simplest model that satisfies ADR-015's hybrid design. Full fixed-timestep machinery is premature. | Tick length is one constant. Revisit only if physics ever demands determinism. |

---

## 2. Feature Inventory (by beat)

### Beat 1 — Wake up 🔷
- Window + event loop + input handling
- Render an authored interior scene (background + sprites)
- Player entity: 4/8-direction movement, walk animation (can be 2-frame)
- Collision vs static rectangles (walls, furniture)
- Interact verb: proximity + facing check + button
- Examine triggers → narrator text (keepsake: flavor only, no mechanics hint — ADR-018)

### Beat 2 — The empty house 🔷
- Scene transition (room → room): unload/load scene, player spawn points
- Dialogue machinery: typewriter reveal, per-register blip sounds, advance/skip input
- Registers (ADR-021): narrator (no avatar) + inner monologue (avatar, distinct frame)
- Trigger system: examining X starts dialogue Y, sets flag Z

### Beat 3 — Village 🔷 partial / 🔶 full
- Exterior authored scene, larger walkable area, camera follow 🔷
- Ambient NPCs: 3–4 🔷, full village population 🔶
- Blocked main gate (static collision + examine line) 🔷
- Flat plank examine → narrator line (ADR-019) 🔷
- Cheerful village music track (single looping track) 🔷
- Day/night tint keyed to flags 🔶

### Beat 4 — Clue chain 🔷 shortest path / 🔶 full
- Flag store: named booleans, set by dialogue/triggers (ADR-020)
- Dialogue entries with flag conditions (NPC line selection)
- NPC dialogue register (avatar + standard frame) + per-character blip voice
- Slice chain: villager → vendor → child (3 conversations, ~2 flags) 🔷
- Full chain incl. alley, cloth shop, changed ambient lines 🔶

### Beat 5 — The guard 🔷
- Item-grant moment = dialogue + flag + HUD icon appears (no inventory system — ADR-022)
- Scene exit gated on flag (`has_sword`)

### Beat 6 — Cemetery & the fold 🔷
- Encounter trigger (enter zone with flag conditions met)
- Pre-fight dialogue (initiate line — exact words pending, direction per ADR-016)
- **Flatten transition v1:** freeze overworld frame → visual collapse effect (v1 acceptable: vertical squash of the captured frame into the battle backdrop, or a hard wipe) → battle scene
- Scene stack: overworld state preserved under the battle scene

### Beat 7 — First battle 🔷
- Grid model: 4 wide × (2 player + 3 enemy) rows; tile↔world mapping (ADR-014)
- Player on grid: instant/fast tile-to-tile movement, real-time input (ADR-015)
- Tick scheduler: accumulator, ~0.5s enemy action cadence
- Enemy behavior: initiate-soldier state machine (choose column → advance → telegraph → strike → recover)
- Telegraph system: target tiles highlighted one tick before damage (ADR-015)
- Player attack: 1-block (sword), lockout frames during animation
- Hit resolution, HP for both sides, damage numbers optional
- Win state → Beat 8; lose state → retry from road (no death spiral)
- 🔶 second enemy (initiate-archer, range 2) + simultaneous-enemy handling

### Beat 8 — Aftermath 🔷 / 🔶
- Un-flatten transition (reverse of the fold)
- Essence thread visual: particle/sprite from corpse → sigil; pendant-warm feedback 🔷 (no UI, no numbers — the slice's only progression leak)
- Map drop = flag + key-item moment 🔷
- Closing monologue + "to be continued" end screen 🔷
- First tree choice UI 🔶 (tree itself ⬜ full design)

---

## 3. Engine / Game Split

### Engine systems (machinery — `src/engine/`)

| ID | System | Serves beats | Slice scope |
|---|---|---|---|
| E1 | **Platform**: window, event loop, input mapping (winit) | all | 🔷 |
| E2 | **Renderer**: wgpu device/surface/pipeline, textured quads, sprite batching (naive ok), camera, y-sorted draw order, text rendering | all | 🔷 |
| E3 | **Time**: frame delta; battle tick accumulator | all / battle | 🔷 |
| E4 | **Scene system**: scene trait, stack (push battle over overworld), transitions incl. flatten hook | 2,6,8 | 🔷 |
| E5 | **Entity model**: plain structs, ID/index addressing (D-A) | all | 🔷 |
| E6 | **Dialogue machinery**: typewriter, three registers, blip playback, flag-conditioned line selection, advance/skip | 2–6,8 | 🔷 |
| E7 | **Audio**: one music track, SFX/blip playback (kira) | 3+, dialogue | 🔷 |
| E8 | **Interaction/trigger**: proximity + facing + button; zone triggers | 1,3,4,6 | 🔷 |
| E9 | **Flag store**: named booleans, query API | 3–8 | 🔷 |
| E10 | **Asset loading**: image decode → GPU texture, audio load; static tables for data (D-C) | all | 🔷 |
| E11 | **Grid combat substrate**: grid math, tile highlight rendering, tick scheduling | 7 | 🔷 (built in game module first; promoted to engine only if a second battle context ever wants it — the extraction principle in miniature) |

### Game content (`src/game/`)

Scenes (bedroom, house, village, road, cemetery, arena backdrop) · player + NPC + initiate sprites and definitions · dialogue tables · flag list · enemy behavior parameters (ranges, HP, tick costs) · the sigil/keepsake, sword, map as key-item flags + art · music + blip sounds · story sequencing.

**Explicitly absent from the slice** (unchanged): tree UI, essence numbers, save system, pause menu tabs, journal, second town, full cultists, isometric tilemaps, serde.

---

## 4. Rust Concept Map

Ordered by when the build forces each concept. "Bites" = where you will meet it whether you like it or not.

| Concept | Where it bites | Milestone |
|---|---|---|
| Ownership & moves | Everything; first felt passing assets/strings around | M0–M1 |
| Borrowing (& vs &mut) | Update loop mutating entities while something else reads them | M0, M3+ |
| Structs, impl, methods | Player, scenes, everything | M0+ |
| Enums + match | Input events (M1); dialogue registers (M4); **enemy state machines** (M6) — enums-as-state-machines is a Rust superpower, lean in | M1,M4,M6 |
| Option / Result, `?` | Asset loading, every fallible API | M1+ |
| Modules & visibility | `engine/` vs `game/` boundary from day one (D-B) | M1+ |
| Closures & `move` | winit's event loop closure; `'static` bound confusion is a rite of passage | M1 |
| Traits | wgpu's trait-heavy API (M2); `Scene` trait (M4) | M2,M4 |
| Trait objects (`dyn`) | Scene stack holding heterogeneous scenes | M4 |
| Lifetimes (reading them) | wgpu surface/device relationships; mostly *reading* signatures, rarely writing them | M2 |
| Slices, Vec, HashMap | Entity lists, flag store | M3,M5 |
| **Indices over references** | Enemy AI needs player position; borrow checker vetoes the C#-style object graph. THE signature Rust-gamedev lesson. Expect it; don't fight it; store IDs. | M5–M6 |
| String vs &str | Dialogue tables | M4 |
| Iterators & combinators | Entity queries, y-sort (`sort_by_key`) | M3+ |
| serde | — deferred (D-C) | ⬜ Phase 1 |
| Smart pointers (Rc/RefCell) | Only if fighting D-A's index rule; treat their appearance as a design smell here | (avoid) |
| WGSL (not Rust!) | Shader for the sprite pipeline — a separate small language; budget it | M2 |

**Baseline (M0) reading list, deliberately shallow:** Rust Book ch. 1–10 fast pass + Rustlings through structs/enums/traits/error-handling. Target: "I can read a compiler error and know which chapter to reopen." Not mastery. 1–2 weeks alongside M1.

---

## 5. Milestones (each ends visible)

| M | Name | Definition of done | Engine systems |
|---|---|---|---|
| M0 | Baseline | Rustlings sets done; Book ch1–10 skimmed; can explain ownership to a rubber duck | — |
| M1 | The Window | `cargo run` opens a window, cyan clear color, ESC quits, WASD logs to console | E1, E3 (delta) |
| M2 | The Sprite | Triangle → quad → **textured sprite at an arbitrary position**; camera offset works | E2, WGSL |
| M3 | The Room | Beat 1 playable: authored bedroom, player walks with collision, camera follows, y-sort proven (walk behind furniture) | E5, E8 (partial), E10 |
| M4 | The Voice | Beat 2 playable: room transition; examine bed → narrator text; typewriter + blips; inner-monologue frame distinct | E4, E6, E7 |
| M5 | The Village | Beats 3–5 slice path: village scene, 3–4 NPCs, flag-conditioned clue chain, guard grants sword (flag + HUD icon) | E9, E6 full, E8 full |
| M6 | The Fold | Beats 6–7: encounter trigger, flatten v1, full battle vs one initiate-soldier (grid, tick, telegraph, lockout, HP, win/lose/retry) | E11, E4 (stack) |
| M7 | The Hook | Beat 8: un-fold, essence thread, map drop, closing text, end screen. **Slice complete.** | polish |

---

## 6. The Plan — Days 1–30 (+ preview to 45)

> Assumes part-time evenings/weekends cadence. Every block ends with something to look at. If a block finishes early, pull the next forward; if late, cut polish, never cut the visible deliverable.

**Days 1–2 — Ignition.** Install toolchain (`rustup`, clippy, rust-analyzer). `git init` docs-first repo; commit Log v1 → v2 → this document. `cargo new` with `engine/`+`game/` module skeleton. Commit: `chore(repo): initialize rust project with module skeleton`.

**Days 3–7 — M0 ∥ M1.** Mornings/first-half: Rustlings + Book fast pass. Second-half: winit window, event loop closure (first real fight: `move` and `'static`), clear color, input logging. *Deliverable: the window exists. Screenshot it. It counts.*

**Days 8–16 — M2, the wgpu wall.** Budgeted a full week+ on purpose; ~300 lines of boilerplate before the first triangle is normal and is the GPU being honest, not you failing. Sub-goals so no dead days: day 10 triangle · day 12 quad · day 14 textured quad · day 16 camera offset + several sprites. Follow the `learn-wgpu` tutorial structure but type, don't paste. *Deliverable: a sprite you can move with arrow keys.*

**Days 17–20 — M3.** Bedroom scene art (placeholder rectangles legitimate!), movement + collision rects, y-sort, camera. *Deliverable: Beat 1 walkable.*

**Days 21–26 — M4.** Scene trait + stack, transition, dialogue machinery with typewriter + blips, two registers styled. *Deliverable: Beat 2 — examining her empty bed and reading the first monologue, with sound. The first moment the game has feelings.*

**Days 27–30 — M5 begins.** Flag store, NPC tables (static data), first two clue conversations. *Day-30 state: walkable village slice-path with working dialogue and flags — demo-able to another human.*

**Days 31–45 (preview) — M5 finish → M6 → M7.** Guard + sword (~d32) · grid + tick + player-on-grid (~d36) · initiate behavior + telegraphs + lockout (~d40) · flatten v1 + win/lose (~d43) · Beat 8 + end screen (~d45). **Slice complete around day 45.** The 30-day number is the *checkpoint*, not the finish line — plan says so out loud to prevent week-3 despair.

---

## 7. Risks Specific to This Plan

1. **The wgpu wall (days 8–16).** Mitigation: sub-goals every 2 days; typing not pasting; it is permitted to feel slow.
2. **Baseline rabbit-holing.** The Book is good; that's the trap. Hard rule: M0 ends day 7 whether "finished" or not — the project resumes teaching from there.
3. **Placeholder-art shame.** Rectangles-with-labels are the correct art for M3–M5. Real sprites are a Phase 1 thickening task; drawing them now is procrastination wearing a beret.
4. **Index-refactor panic (M5–M6).** When the borrow checker vetoes entity references, that's the curriculum arriving on schedule (D-A). Budgeted: ~a day of confusion, then it clicks and the entity model is better forever.
5. **Battle-feel perfectionism (M6).** Tick length/lockout/telegraph visuals are knobs (Log v2 OQ-1). Get it *functional* in the slice; get it *good* in Phase 1 playtesting.

---

## 8. What This Document Unblocks

- Repo + `cargo new` — immediately.
- Log v3: fold D-A…D-E in as ADR-025–029.
- The next session types code or debugs the toolchain. Design is, for now, done.
