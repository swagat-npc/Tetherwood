# Project Log — Tetherwood
### Game: *Tetherwood* · Act I / vertical slice: *"The Morning She Was Gone"* · Engine: unnamed (deliberate)

**Document type:** Living project record (docs-as-code)
**Started:** July 2026
**Revision:** v5
**Status:** M1–M3 complete; M4 (The Voice) in progress — scene transitions, camera modes, lazy scene reload, and text rendering foundation all built and verified; dialogue machinery (typewriter, registers, blips) and Beat 2 content next. Companion document: `docs/DERIVATION.md` (feature inventory, engine split, Rust map, milestones, 45-day plan).
**Maintenance model:** Single canonical file at `docs/PROJECT_LOG.md`, versioned with git. Updated when decisions accumulate, not on a timer.
**Screenshots at each visual milestone** are part of this ritual, not an afterthought — historically the step most likely to be skipped (M4's first two phases shipped with none until caught retroactively); a phase isn't fully closed until its screenshots exist and are named.

---

## 1. How to Read This Document

Four layers:

1. **Project Charter** — the stable "constitution." Changes rarely.
2. **Phase Log** — chronological record of work and conclusions.
3. **Decision Log (ADRs)** — every significant decision: context, decision, rationale, consequences. Immutable; changed decisions get a superseding ADR.
4. **Current State & Open Questions** — snapshot, rewritten each revision.

---

## 2. Project Charter

### 2.1 Mission

Build a playable game in Rust, and extract a reusable, eventually open-source game engine from the process — in that order of priority.

### 2.2 The Governing Principle

> "I am building a game in Rust, and extracting a reusable game engine from the process."
> — NOT: "I am building a general-purpose game engine, and eventually I will make a game."

```
Game needs feature → Implement → Use in game → Learn from real usage
→ Refactor/generalize where appropriate → Promote reusable parts into engine
```

**Micro-scale application (the system/content split):** every feature is built as *machinery* (reusable, data-driven — proto-engine) plus *content* (game-specific data the machinery consumes). Test for machinery: "could a different game reuse this code without editing it?" Example: the dialogue system (typewriter, blips, flag-conditions, interaction detection) is machinery; the fishmonger's sprite, position, and lines are content. Extraction later = moving folders, not surgery.

### 2.3 The Twelve Principles

1. Build a game, not just an engine.
2. Let the game drive engine development.
3. Start with 2D.
4. Learn Rust through practical implementation.
5. Avoid premature generalization.
6. Prefer small working vertical slices.
7. Use existing libraries for sensible low-level infrastructure.
8. Understand the technology rather than blindly depending on frameworks.
9. Extract reusable systems when real use cases emerge.
10. Treat open source as a long-term goal.
11. Prioritize finishing something playable.
12. Keep architecture clean enough that the engine can eventually become independently reusable.

### 2.4 Developer Profile

- 6+ years professional software development; M.Sc. Informatics (TU Munich).
- Strong: TypeScript/Angular, C#/.NET, full-stack, general architecture.
- Beginner: Rust, game engine architecture, graphics programming.
- Implication: skip general programming explanations; never assume Rust idioms or graphics knowledge.

### 2.5 Technology Stack (slice-committed vs deferred)

| Concern | Choice | Status |
|---|---|---|
| Language / build | Rust + Cargo | committed |
| Windowing | winit | committed (slice) |
| Graphics | wgpu (+ WGSL shaders) | committed (slice) |
| Math | glam | committed (slice) |
| Audio | kira | committed (slice) |
| ECS | none for slice (ADR-025) | re-evaluate Phase 1 |
| Serialization | none for slice (ADR-027) | serde/RON at Phase 1 thickening |
| Physics | none — grid combat needs no physics lib | likely never |

---

## 3. Phase Log

### Phase 0 — Project Framing (completed)

Established identity, constraints, feasibility, and the ten-step blueprint. Conclusions: feasible with controlled scope; 2D first; engine emerges from game; first task is concept discovery.

### Phase 1 — Concept Discovery: Systems Identity (completed)

- **Visual identity:** 2D isometric overworld with a 3D-like feel.
- **Exploration:** scene-based (interiors/exteriors, small interconnected areas).
- **Narrative interaction:** NPC dialogue, typewriter effect, character-specific text blips.
- **Combat:** encounters transition to a dedicated top-down battle scene — real-time grid-based combat, inspired by Mega Man Battle Network's *design philosophy*, not its mechanics.
- **Loop:** Home → explore → talk → discover goal → travel → encounter → grid battle → continue → goal.

### Phase 2 — Concept Discovery: Premise (completed)

Five candidates generated and rated (Last Delivery, Village Festival, Missing Person, Pilgrimage, Strange Morning). Selection: **Missing Person frame + Last Delivery's "not who they seem" twist, inverted** — the sister is secretly *better* than known: hidden, unregretted sacrifices for the protagonist.

### Phase 3 — Story & Theme Synthesis (completed)

**Arc:** Sister missing → village investigation → next town → she was involved with masked strangers → confrontations escalate → it's a cult → they will sacrifice her → their magic is **dimensional flattening** (sacrifice the z-axis for power) → ritual understood, countdown begins, tone shifts to non-stop escalating combat → at the altar, the ritual is explained: it requires a sacrifice whose own selfless sacrifices are *unknown* to their beneficiary → **the explanation itself reveals her sacrifices, breaking the ritual's precondition** → the villains defeat themselves by monologuing → the bond deepens; end.

**Conclusions:**
- The z-axis flattening is the game's **thesis**, not a joke: entering combat = being pulled into the cult's magic. Visual identity, combat system, and climax are one idea.
- Progression arc told through the camera: early, flattening is done *to* you; late, you fight comfortably in it or invoke it.
- Reserve idea (uncommitted): the protagonist may qualify as a replacement sacrifice by journey's end.
- Setting: pre-modern fantasy (walking-pace earnestness; no tech plot holes; Game Boy-era tone).
- Emotional presentation: still frames + music at revelations.
- Scope warnings accepted: inventory analysis deferred; escalation via behavior combination not stat inflation; fixed scene budget; no party members.

### Phase 4 — Methodology: Design-First Development (completed)

Storyboard → feature list → engine requirements → Rust concept map → **minimal Rust baseline** (1–2 weeks, shallow: ownership, borrowing, structs, enums, match, traits) → build milestones, learn on demand. The slice storyboard is a deliberate blocking dependency for coding.

### Phase 5 — Vertical Slice Storyboard v1 (completed, superseded by Phase 7)

Drafted the 7-beat slice with per-beat system demands. Retained conceptually; structure revised in Phase 7 after combat and first-act design.

### Phase 6 — Combat & Progression Design (completed)

**Grid:** Split grid, player bottom vs enemy top. **2 player rows vs 3 enemy rows** (asymmetry discourages back-row camping: depth trades safety against reach in both directions). Width ~4, tuning knob. Enemies cannot enter player territory (v1). **Third player row is a story-unlocked milestone** ("you now exhibit her selfless nature — you are on par").

**Time model:** Hybrid. **Enemies act on a game tick (~0.5s); the player moves in real time.** The enemy is a board game; the player is an action game invading it — and diegetically, the flattened dimension runs on the cult's rules while the player remains a 3D creature.

**Readability:** **Telegraph-then-strike** — attacks wind up for a tick with target tiles highlighted, damage lands next tick. Dodging = reading tiles under pressure, not sprite reaction speed.

**Risk/reward:** Player attacks have **lockout frames** (cannot move during attack animation). Committing to an attack means eating telegraphs you saw coming.

**Enemy tiers by range:** soldier (1, advance-and-stab) / archer (2, kite-and-shoot) / mage-cultist (3). Each tier **erodes a safe zone**; escalation is spatial, not statistical. **Initiates** (recruited members who haven't yet paid the flattening price) provide the mundane early tiers: same robes-and-masks silhouette, no magic; full cultists with 3-range magic stay in reserve. Art economy: one base sprite, sword/bow swap.

**Progression:** the **sigil tech tree** subsumes both the XP and stat-choice alternatives:
- Currency: **z-essence** — when a cult member dies, their severed z-axis has nowhere to go; the player absorbs it. Every battle pays (no grind staleness); branches allow different runs.
- Double juxtaposition: they grow by *rejecting* a dimension, you by *inheriting* what they discard; they exploit selflessness, you become selfless.
- **Nodes are verbs, not percentages** (~10–15 total): 2-block attack, 3-block attack, dash, reduced lockout, third row, etc.
- Diegetic form: wooden sigil worn around the neck; the completed tree resolves into a winged/angel silhouette — **the reveal is earned, not shown**, landing near the altar revelation.
- **Keepsake unification:** the Beat 1 keepsake *is* the sister's sigil. The emotional seed and the progression system are one artifact.

### Phase 7 — First Act Design & Beats v2 (completed)

**Scope model formalized:** the **vertical slice is the thin path through Phase 1**, not Phase 1 itself. Build the slice, prove the systems, then *thicken* into Phase 1 (ambient NPCs, full clue chain, menus, second enemy). Same map, two zoom levels. Development proceeds in iterative phases/passes.

**Motivation repair:** a **rainstorm the night before** — the sister didn't come home *through a storm*; that is panic, an empty room is not. The storm also covers the cult's sabotage of the main gate (wall damage blocks the front exit naturally, guiding the player to the back path) — readable in hindsight as *planned*, an aha for the player, invisible to the character.

**Foreshadowing:** inspectable **flat plank** in the damaged wall — "This plank looks oddly peculiar. You feel a sense of unease." One narrator line, zero systems, the thesis whispering. Justified by the sibling spiritual tether (her selfless bond → the player *feels* the in-progress ritual).

**Context system:** **named boolean flags** (`talked_to_vendor`, `knows_about_masks`, …), not an integer level. Dialogue entries carry flag conditions; NPC presence, obstacles, and **day/night keyed to knowledge** — the sun moves when you *learn*, not when you wander (Festival-premise pacing, resurrected; thematically rhymes with the ending where understanding is the weapon).

**Voice system — three registers, one machinery:** silent protagonist (never speaks in dialogue). (a) NPC dialogue: avatar + standard frame. (b) **Inner monologue**: avatar + visually distinct thought-bubble frame ("XYZ… where are you?"). (c) **Narrator**: no avatar ("You feel a sense of unease"). 4th-wall tiers rationed: T1 in-world texture (unlimited), T2 deniable winks (occasional), T3 full breaks (once or twice per game, ever) — the altar scene spends sincerity the narrator must not have already eroded. Parked question: *who narrates?* (possible tether tie-in; not designed, not foreclosed).

**World texture:** village full of ambient NPCs (Pokémon-style "busy standing"). Meta-content: period-appropriate jokes only (tulip mania as the "crypto" joke; the lost red hood) — brainstorm session pending. **Scrapped premises live on as NPCs**: a weary courier with one important letter, a festival-goer, a northbound pilgrim; narrator may append T2 winks ("Looks like he's also searching for something. Hope you both find it.").

**Weapons: story-granted milestones, no equipment system.** HUD icons are non-interactable, so there is no equipment layer to build. The tree unlocks the *capacity* (1/2/3-block attacks); the story hands the *instrument* (guard's sword → bow → staff), mirroring the enemy tiers as essence is absorbed. Sigil stays on HUD (with tree-open button hint); everything else lives in the pause menu.

**Pause menu (Phase 1 design):** key items / map / tech tree / **clue journal** (text-only list of learned facts — the "did I miss something" tool). The pokédex-style character glossary with portraits is the Phase 2 glow-up of the journal (art bill staged out).

**Diegetic map:** dropped by the defeated initiate; a magical cult map that shows the player's position *because the player is siphoning their power* — the z-essence rule doing narrative work. **Show-then-tell exposition**: the map behaves impossibly first, the narrator explains second.

**Enemy dialogue as progress bar (acknowledgment ladder):** enemies acknowledge *you* → *her* → *the cult* → *the ritual*, in that order across the game. First fight line direction: they recognize you and warn you off without confirming anything ("Tough luck, kid…" — exact line workshop pending).

**Beats v2 (Phase 1, slice path marked):**

| # | Beat | Slice? |
|---|------|--------|
| 1 | Wake up; storm last night; her bed empty — panic has a reason. Keepsake/sigil examined (flavor text hints nothing mechanical). | ✅ |
| 2 | The empty house; discovery via inner monologue + narrator (registers per the voice system). | ✅ |
| 3 | Village, alive: ambient NPCs, cheerful music, nothing visibly wrong. Main gate blocked (storm damage, mayor repairing). Flat plank inspection: the eerie line. | partial — 3–4 NPCs, plank yes |
| 4 | Clue chain via flags: villagers saw her scurry → alley → trinket vendor (chalk, candles, the missing white silk scarf) → directions to the cloth shop → the child's promise. | ✅ shortest path |
| 5 | Back exit: guard warns of lurkers near the cemetery, hands over his backup sword (tutorial excuse + first milestone item). | ✅ |
| 6 | Cemetery: masked figures. "Tough luck, kid." **The world folds.** | ✅ (1 initiate in slice; 2 in Phase 1 — sword + bow, advance vs kite on display) |
| 7 | First battle: split grid, ticks, telegraphs, lockout. | ✅ |
| 8 | Aftermath: essence thread sinks into the sigil (no UI, pendant warms — the slice's only progression leak); magical map drops; Phase 1 adds first tree choice. "She's headed somewhere." | ✅ minus tree UI |

**Slice exclusions (unchanged in spirit):** tree UI, z-essence UI, multiple simultaneous enemies beyond one initiate, abilities, second town, cult reveal content, save system, pause-menu tabs, glossary.

### Phase 8 — Derivation Pass (completed)

Beats v2 ground into: full feature inventory (per beat, slice/Phase-1 tagged) → engine/game system split (E1–E11 machinery vs content) → Rust concept map (concept → where it bites → milestone) → milestones M0–M7, each ending visible → day plan (30-day checkpoint: walkable village with dialogue and flags; **slice complete ~day 45**). Full output lives in `docs/DERIVATION.md`. Five decisions forced by the pass are recorded below as ADR-025–029. Named risks: the wgpu wall (days 8–16, budgeted), baseline rabbit-holing (M0 hard-stops day 7), placeholder-art shame (rectangles are correct until M5), the index-refactor (curriculum, not crisis), battle-feel perfectionism (knobs are for Phase 1 playtesting).

### Phase 9 — Naming (completed)

Title brainstorm to stress-test the working title. Criteria that emerged: layered referents that re-read after the credits; revelation energy, not secrecy energy; shaped as a tiny story or as the driving artifact (Ocarina pattern); weighty; not on-the-nose. Result: **Tetherwood** (ADR-030). The former working title survives as the Act I / slice chapter title. Collision check performed (small itch.io jam prototype of the same name; assessed and accepted as soft). Engine remains deliberately unnamed.

### Phase 10 — M2: The Wall (completed)

Full wgpu pipeline stood up: instance/adapter/device/queue, WGSL shader,
unit quad + index buffer, texture module (decode, upload, view, sampler),
two bind group slots (texture, transform uniform). 2D world coordinate
system decided and implemented (ADR-031). Transform chain established as
projection * view * model (ADR-032), recomputed every frame. Sprite
position convention corrected mid-milestone to mean center, not top-left
(ADR-033), discovered via camera-centering testing rather than anticipated
up front — a real example of the "let the second use case teach the
abstraction" pattern the project already trusts (cf. ADR-025). Back-face
culling disabled as a consequence of the y-down projection (ADR-034).
Held-state WASD input implemented directly in App, with the raw-keycode
placement flagged as deliberate, temporary debt rather than a missed
abstraction (ADR-035). Camera-follow implemented and proven correct.

M2 definition of done met in full: textured sprite, arbitrary position,
movable with arrow keys, camera offset working. Docs-as-code extended:
docs/screenshots/ established, 10 images taken across M1–M2, each tied to
a specific lesson (clear color, first triangle, interpolation, winding/
culling, texture skew before/after transform, held-input movement,
camera-follow).

### Phase 11 — M3: The Room (completed)

Full entity/scene layer built from scratch and proven against real,
playable content. `Entity` (E5): position (center, per ADR-033), size,
`Option<Rect>` collider, `Option<TextureId>` texture — the same shape
covers visible solid furniture, invisible solid blockers (Kecleon-
style), and visible non-solid decoration without special-casing any
of them. `TextureStore`: scene-scoped, load-once, never-mutated asset
storage; textures referenced by a `Copy` newtype index rather than
owned or `Rc`-shared per entity, extending ADR-025's indices-over-
references principle from entities to assets (ADR-036). Entities are
deactivated, never removed, from a scene's entity list during play
(ADR-037) — verified against both a defeated-enemy case and the Beat 1
keepsake-pickup case without needing new machinery.

Collision: AABB overlap derived and hand-traced (center-distance vs.
summed half-extents, per axis) before being written as code
(`aabb_overlap`); walls (`Vec<Rect>`, world-space) and entity colliders
feed the same test, source-agnostic. Movement resolves as two
sequential per-axis proposals (x, then y from the resulting x) rather
than one combined 2D proposal — produces correct sliding along walls
and furniture edges, verified against inside-corner cases by hand
(ADR-038).

Y-sort: entities draw in ascending order of baseline
(`position.y + size.y / 2.0`), not center — confirmed against tall-
furniture cases where center-sort gives the wrong order. Feet-only
collider boxes (deliberately smaller than sprite bounds) are designed
to work with this rule, producing the walk-behind-furniture effect
(ADR-039).

Renderer rewritten end to end: the M2 single hardcoded sprite is gone.
`prepare_scene` builds one `wgpu::BindGroup` per texture at scene load;
`render` now takes a `&Scene`, y-sorts its entities, and draws the
background plus every visible entity, each with its own command
encoder and submission — discovered and fixed a real correctness bug
where writing the shared transform buffer multiple times before one
shared `submit()` would have let every draw read only the last-written
transform (ADR-040).

Camera gains a second mode: a static, scene-authored anchor (bedroom
anchors at its own center) alongside M2's follow-camera, giving
indoor/outdoor scenes distinct feel. No `CameraMode` abstraction built
yet — one concrete consumer so far, deferred per the same reasoning as
ADR-035 (ADR-041).

`multiplying_factor` scales the entire scene composition — every
position, size, and collider dimension, for entities, walls, and the
background alike — uniformly from world origin, plus player movement
speed. An early version that scaled size only (leaving position and
the room boundary fixed) caused furniture to visually drift out of
proportion with the room; scaling the whole composition together
preserves every placement ratio regardless of factor, decoupling
layout correctness from visual zoom (ADR-042).

Debug tooling added as permanent, reusable engine machinery rather
than a throwaway M3 aid: a collider overlay (F1) via a second render
pipeline that computes fill/border directly from each fragment's UV
coordinate — not a stretched texture, which cannot represent a
constant-pixel-width border across non-square rects. Border thickness
is converted from pixels to UV space per axis, per draw, since UV
space is stretched independently per axis (ADR-043). Keypress logging
(F2) hooked into the existing press-state match arm, not per-frame
held-key checks, so each press logs exactly once. Both tools were then
used to fine-tune the bedroom's actual furniture placement and sizes,
visually confirming collision and the walk-behind y-sort effect
against real content, not just hand-derived numbers.

M3 definition of done met in full: authored bedroom scene, player
movement collision-checked against both walls and furniture, y-sort
proven by walking behind furniture — confirmed on screen, not just on
paper. Docs-as-code extended: docs/screenshots/ now covers m3-01
through m3-06, each tied to a specific lesson (multi-entity render,
factor scaling, debug overlay alone and combined with scaling, tuned
collision, tuned y-sort).

### Phase 12 — M4 Design Session: Scene Transitions & Persistence (completed)

Design pass completed before any M4 code was written, resolving five
open questions the milestone's own agenda had flagged. Scene
abstraction scope deliberately narrowed (ADR-044) rather than built to
DERIVATION's original E4 description, following the same
defer-until-second-consumer instinct as ADR-035/041. Door transitions
settled as automatic zone-walkthrough, GBA/MMBN-style, no button
prompt (ADR-045). Trigger system split into two flavors — existing
interact triggers vs. new zone triggers — with dispatch kept to a
single-variant enum rather than generalized callback machinery
(ADR-046). Warp system adopted Pokémon-style paired-object identity
rather than per-scene named spawn points, chosen specifically because
it scales flat to N doors without combinatorial spawn-point tables
(ADR-047). Scene persistence resolved by separating two previously
conflated concerns — GPU resource lifetime vs. narrative state
lifetime — reusing ADR-020's flag store rather than building new
persistence machinery; scenes remain fully lazy-unloaded, reconstructed
deterministically from flags on revisit (ADR-048). Aseprite native
file loading (raised as a workflow-friction question) assessed as
genuinely feasible but explicitly parked — no dependency from M4,
revisit only when PNG-export friction becomes a real cost, not a
curiosity (ADR-049).

Aseprite native loading (ADR-049's parked status) was revisited and
implemented mid-milestone once PNG-export friction was felt directly
during scene-content work — see ADR-050.

A scope misunderstanding was caught and corrected: no "hallway" scene
was ever part of the design — that term arose only in reference to the
Village Chief's House wireframe, where it names ordinary walkable
floor space connecting rooms *within one scene*, not a warp-mediated
destination. M4's actual warp target is a minimal placeholder exterior
scene (`new_outside`), alongside the player's house (renamed
`new_bedroom` → `new_home`, since the constructed content is the whole
house, not just the bedroom). Multi-room single-scene houses (Chief's
House) are parked as M5+ content; they surface a real, separate design
question — per-scene camera mode (ADR-041) would need to vary *within*
a scene (static in a room, following through a connecting corridor,
static again) rather than being fixed once at scene load. Not
designed further now; noted for whenever the first such house is
built.

Trigger/warp identity design settled (ADR-051): WarpId changed from a
u32 newtype to a `&'static str` newtype, since warps are always matched
associatively rather than indexed into a Vec (unlike TextureId, which
does need to be numeric) — this lets one authored value serve as both
routing key and human-readable debug label with no separate registry
to maintain. TriggerKind::Warp gained its own warp_id field (missing
from the original design), needed for a resolved warp to find its
named partner trigger inside a freshly-constructed destination scene.
Re-trigger suppression after a warp is a per-trigger `recently_used`
flag, not a scene- or App-wide flag, so only the door just used is
suppressed. Scene gains a `pub id: SceneId` field for self-identifying
debug output. A startup validation pass over all warp pairs was
proposed and explicitly deferred (ADR-052) until warp count makes
manual playtesting an unreliable way to catch a typo'd WarpId.

Scene-transition implementation followed the design in full: the
warp_id mismatch caught between Home's and Outside's paired triggers
was corrected, and recently_used suppression was wired end to end
(Scene::check_triggers clears stale flags and skips already-used
triggers; Scene::activate_warp marks the destination trigger used on
arrival) — confirmed via cargo run that walking through a doorway no
longer re-fires the warp or repeatedly rebuilds renderer bind groups
every frame spent standing in it.

Implementation also surfaced two gaps the original design hadn't
covered. First, App's window/renderer/scene fields, having drifted to
eager construction in run() rather than resumed() during earlier
scaffolding, were restructured into a single Option<AppState> guarded
by resumed() — see ADR-053. Second, spawning the player exactly on a
warp trigger's center produced a visually incorrect landing (standing
on top of the door), and — separately — revealed that a scene's
player position was previously left stale across repeat visits, since
visited scenes are cached rather than reconstructed; TriggerKind::Warp
gained a spawn_offset field to address both — see ADR-054.

A main-menu / pause-menu need was raised and deliberately deferred —
see ADR-055.

Docs-as-code extended: docs/screenshots/ now covers m4-01 through
m4-05, each tied to a specific confirmed behavior rather than general
progress — the new green trigger-debug color in Home (m4-01) and
Outside (m4-02) under the existing F1 overlay, correct off-trigger
spawn positioning after a warp (m4-03), and a deliberate before/after
pair proving CameraMode::Follow actually tracks the player at a scene
boundary rather than merely differing from Static by coincidence
(m4-04 static, m4-05 follow).

### Phase 13 — M4: Text Rendering Foundation (completed)

E6's low-level foundation — the engine's ability to draw a string of
text on screen at all — built from scratch. DERIVATION's own M4
agenda had flagged this as the milestone's likely "wgpu wall," a new
real subsystem rather than a small addition; that held true (a new
shader, a new pipeline, a genuine restructuring of Renderer's frame
lifecycle), but is now cleared.

A pre-rendered bitmap font atlas was chosen over runtime glyph
rasterization, given the project's committed pixel-art visual identity
and the absence of any localization/arbitrary-font requirement — see
ADR-056. The Good Neighbors font (CC0, OpenGameArt) was hand-arranged
in Aseprite into a uniform 10x9 grid (9x15 glyph cells, 10x16 pitch,
1px gaps against texture bleeding), chosen deliberately over the
font's original packed/variable-width release specifically to avoid
needing an external metadata file — glyph position becomes pure
arithmetic from an explicit char-to-cell lookup table
(engine/text.rs::glyph_cell), since the grid's character ordering
doesn't follow any contiguous range.

Renderer::render() — a single method since M2 — was split into
acquire_frame / render_scene / render_text / present_frame, since
drawing text requires a second draw pass into the *same* acquired
swapchain frame as the scene; two independent acquire+present calls
would each land on a different buffer and flicker — see ADR-057. Text
rendering was scoped to screen-space only (no camera_view term),
since dialogue/narrator text needs to stay fixed to the window
regardless of camera position or CameraMode — see ADR-058.

A new text_shader.wgsl + text_pipeline reuses the existing
textured-quad vertex layout and the shared transform uniform bind
group unchanged from the sprite pipeline, adding one new bind group
(GlyphUniform: uv_offset, uv_scale) that remaps a quad's 0..1 UVs
onto a single glyph's sub-rectangle of the shared atlas texture, done
per-vertex rather than per-fragment. The atlas texture itself is
loaded once in Renderer::new, independent of any scene's TextureStore,
since text is not per-scene content and shouldn't reload on scene
transitions.

An F3 debug toggle renders a hardcoded test string end to end,
confirming correct glyph lookup, UV sampling, monospace spacing
(including the expected wide trailing gap after narrow glyphs), and
silent-skip-with-warning behavior for unsupported characters — before
any real dialogue content exists to exercise the pipeline.

Not yet built: dialogue machinery (typewriter reveal, three registers,
blip playback, advance/skip input), the dialogue panel/avatar frame,
and any actual Beat 2 content (examine-bed narrator text). Text
rendering is foundation these will be built on top of, not itself.

Docs-as-code extended: docs/screenshots/ now covers m4-06, the F3
debug toggle rendering a full test string through the new bitmap font
pipeline — the first confirmed on-screen text this engine has ever
produced.

### Phase 14 — M4: Interact Triggers & Debug Tooling (completed)

E8's second trigger flavor — interact (proximity + facing + button) —
built and proven against real content for the first time, alongside
new debug tooling that made hand-placing that content practical.

Entity gained facing: Direction, a four-way enum rather than a raw
Vec2, chosen so call sites read as Direction::Right rather than
memorizing which unit vector means which cardinal direction — see
ADR-059. TriggerKind grew its second variant, Interact, the genuine
second consumer ADR-046 had been waiting on; a plain match remained
sufficient, no callback dispatch machinery earned. Proximity (icon
visibility) and proximity+facing+button (the actual interaction) share
one Rect per approach side rather than one Rect with a facing list,
after tracing through why a single shared box couldn't distinguish
"standing north, facing away" from "standing north, facing toward" —
an object reachable from two sides needs two Triggers, one prompt
Entity shared between them, not one Trigger with both directions
listed — see ADR-060. EntityId (engine/ids.rs) extends the
indices-over-references principle (ADR-025) from textures/scenes/warps
to entities, needed once a Trigger had to reference a specific prompt
icon Entity by identity.

game/dialogue.rs opened as the project's first real src/game/ content
module — a minimal static lookup (line_for), matching D-C's no-data-
files approach. The bed's real interact content landed: examine text
dropping Beat 2's central lore beat, gated to a single required
facing (Right) matching its physically-constrained approach zone
(confirmed against a reference mockup image, not guessed). The
necklace's equivalent content — a two-sided approach, testing the
opposite (multi-direction) case of the same mechanism — is sketched
in code but commented out, not yet finished.

Two rounds of new debug tooling followed directly from the difficulty
of hand-placing this content blind. A world-space mouse position
readout (Renderer::screen_to_world, shown via F2) inverts camera_view's
translation, recomputed fresh every frame — not just on CursorMoved —
so it stays live even when the camera pans under a stationary cursor
in CameraMode::Follow. It was further adjusted to display
authoring-space coordinates (world_pos / multiplying_factor) rather
than raw world-space ones, after a real false alarm: a trigger placed
at literal (94, 40) read back as (469, 200) under the readout's first
version — not a bug, just world-space already reflecting
multiplying_factor's scale, confirmed and then fixed for direct
comparability against source literals — see ADR-061. A center-position
crosshair marker (push_center_marker), reusing the existing debug-rect
pipeline with no new shader, now draws at the world origin and at
every wall/collider/trigger's center under the F1 overlay, making
Rect placement visually verifiable rather than trusted blind. Trigger
debug rects are now color-coded by kind (green Warp, yellow Interact),
derived fresh from trigger.kind rather than stored as a redundant
field.

The added crosshairs roughly tripled per-frame debug-rect count,
making the pre-existing per-rect draw-call cost (ADR-043's accepted
cost at "a dozen or so" rects) measurably worse — F1 became genuinely
unusable at the resulting frame rate. Text rendering hit the
equivalent wall independently: a full dialogue-length string (~83
glyphs) at one draw call per glyph dropped the game from ~60fps to
~18fps. Both are the same underlying cost (one GPU buffer + bind group
+ encoder + submit per drawn primitive) crossing from "tolerable at
slice scale" to "actively blocking work" — batching (one shared
vertex/index buffer, one draw call per whole string or whole overlay
pass) is the identified fix for both, deferred as its own focused task
— see Open Questions.

Docs-as-code extended: docs/screenshots/ now covers m4-07 through
m4-10 — the interact prompt icon's proximity-gated visibility and the
bed's dialogue text rendering after a successful interaction (m4-07,
m4-08), and the world-space mouse readout confirmed correct before
(m4-09) and after (m4-10) its authoring-space unit correction.

---

## 4. Decision Log (ADRs)

> Format: Context / Decision / Rationale / Consequences. Immutable — changes arrive as superseding ADRs.

### ADR-001: Rust as the project language
- **Context:** Learning systems programming; choosing a language for a game+engine project.
- **Decision:** Rust, exclusively.
- **Rationale:** Learning goal in itself; performance, safety, ownership, strong types; ecosystem (winit/wgpu).
- **Consequences:** Steep early curve; compiler fights are the learning mechanism.

### ADR-002: Game-first, engine-extracted
- **Context:** Classic failure mode — endless engine, no game.
- **Decision:** Game is the product; engine is extracted from proven, game-driven code.
- **Rationale:** Concrete requirements prevent speculative architecture; finishing is a core goal.
- **Consequences:** Early engine code may need later refactoring for reuse. Accepted.

### ADR-003: Low-level libraries, not a full framework
- **Context:** Bevy ships games faster but does the architecture learning for us.
- **Decision:** Compose winit/wgpu/glam etc.; study frameworks for inspiration only. (Bevy ECS *as a library* still under evaluation — see ADR-025.)
- **Rationale:** Understanding engine internals is a primary goal.
- **Consequences:** Slower start; more surface area; full ownership of the result.

### ADR-004: 2D first
- **Context:** 3D multiplies scope.
- **Decision:** First game is 2D; avoid *unnecessarily* blocking 3D, but not at the cost of present simplicity.
- **Rationale:** Scope; 2D covers every fundamental.
- **Consequences:** Portfolio impact rides on distinctiveness (see ADR-006).

### ADR-005: Premise — Missing Person with inverted twist
- **Context:** Five premises evaluated.
- **Decision:** Missing sister; investigation; she is revealed as secretly *more* selfless, not secretly villainous.
- **Rationale:** Strongest motivation; gives dialogue/exploration real purpose; lands the theme.
- **Consequences:** Dialogue and clue systems are load-bearing.

### ADR-006: Z-axis flattening as diegetic thesis mechanic
- **Context:** Needed justification for the isometric→grid transition; developer proposed cult magic sacrificing the z-axis for power.
- **Decision:** Canonized as thesis. Entering combat = being pulled into the cult's flattened dimension.
- **Rationale:** One idea unifies visuals, combat, and climax — maximum solo-dev economy; enables a camera-told progression arc.
- **Consequences:** The flatten transition is a first-class feature, present even in the slice (crude v1 acceptable). Ritual logic must stay consistent.

### ADR-007: The cult's exposition breaks its own ritual
- **Context:** The ritual requires the sacrifice's selfless deeds to be *unknown* to their beneficiary; the altar scene requires explanation.
- **Decision:** The explanation disqualifies the sister — the reveal is the resolution.
- **Rationale:** Elegant; thematically resonant; rhymes with ADR-006's genre-aware wit.
- **Consequences:** The altar writing carries the climax. Reserve (uncommitted): the protagonist as replacement candidate.

### ADR-008: Pre-modern fantasy setting
- **Context:** Village-to-village travel on foot must feel natural.
- **Decision:** Old-world setting; no GPS/vehicles/instant communication.
- **Rationale:** Walking-pace earnestness; no tech plot holes; Game Boy-era tone.
- **Consequences:** Art/audio must carry the atmosphere; **anachronisms policed in writing** (see ADR-020).

### ADR-009: No party members
- **Context:** Parties multiply AI, UI, balance, narrative complexity.
- **Decision:** Solo protagonist, potentially vs multiple enemies.
- **Rationale:** Simplification; MMBN lineage.
- **Consequences:** Depth must come from movement, timing, and enemy behavior combinations.

### ADR-010: Escalation via behavior combination, not stat inflation
- **Context:** "More HP + more goons" is content-expensive and repetitive.
- **Decision:** ~4–6 enemy behavior patterns total; difficulty = combining them.
- **Rationale:** Intersecting simple patterns are new puzzles at near-zero cost.
- **Consequences:** Behaviors designed composable from the start. (Realized concretely in ADR-016.)

### ADR-011: Inventory analysis system deferred
- **Context:** "Analyze dropped items" implies UI + interactions + content.
- **Decision:** Not in slice, not in v1. Clues via dialogue and found notes.
- **Rationale:** Core loop doesn't need it; hidden-subsystem trap.
- **Consequences:** Future addition would be a new ADR. (Partially superseded in practice by ADR-022's clue journal.)

### ADR-012: Design-first pipeline with minimal Rust baseline
- **Context:** Textbook-first vs purely on-demand learning.
- **Decision:** Storyboard → features → engine requirements → concept map → 1–2 week shallow baseline → build and learn on demand.
- **Rationale:** Problems make learning stick; zero baseline makes compiler errors unreadable.
- **Consequences:** Slice storyboard blocks coding — by design. (Fulfilled: the derivation pass, Phase 8, was the gate.)

### ADR-013: Docs-as-code with ADRs
- **Context:** Start-to-end record wanted, maintained during the project; assistant has no cross-session persistence.
- **Decision:** This log lives in the repo, updated on request at session ends; decisions as immutable ADRs.
- **Rationale:** Git guarantees continuity; ADRs answer "why."
- **Consequences:** Developer owns the update ritual. **Versioning is git commits on one canonical file** — filename suffixes (_v3) are a pre-repo stopgap only. Log revs when decisions accumulate, not on a timer. `docs(repo)` is used specifically for repo-wide milestone-closing commits — a Phase Log entry, a batch of ADRs, current-state and next-session rewrites, all together, closing out a completed milestone (cf. the M2 and M3 log-close commits). Smaller, isolated documentation edits (e.g., a single screenshot addition) use `docs(log)` instead.

### ADR-014: Split-grid combat with asymmetric depth
- **Context:** Split grid vs shared free-movement grid; camping/cheese concerns.
- **Decision:** Player bottom (2 rows) vs enemy top (3 rows); width ~4 (tuning knob); enemies cannot enter player territory (v1); third player row is a story-unlocked milestone tied to "you now share her nature."
- **Rationale:** Depth asymmetry makes positioning a two-way trade (back row = safer but shorter reach); readable; evokes MMBN; the row unlock is progression-as-story-beat.
- **Consequences:** Escalation design must exploit the depth structure (ADR-016). Territory-crossing (panel-steal style) explicitly out of v1 scope.

### ADR-015: Hybrid tick system + telegraphs + attack lockout
- **Context:** Full real-time is chaos to read; full tick-lock feels input-dead.
- **Decision:** Enemies act on a ~0.5s game tick; the player moves in real time. Attacks telegraph target tiles for one tick, land the next. Player attacks impose lockout frames (no movement during animation).
- **Rationale:** "Enemy is a board game, player is an action game invading it" — the feel contrast *is* the game, and diegetically the flattened dimension runs on cult rules. Telegraphs make dodging about reading, not reflexes. Lockout creates informed risk/reward.
- **Consequences:** Tick length, lockout duration, and telegraph visuals are named tuning knobs. Global-tick metronome risk mitigated by telegraph design.

### ADR-016: Enemy tiers as safe-zone erosion; initiates as the early tier
- **Context:** Escalation needed without stat inflation (ADR-010); robed cultists must not be spent in the tutorial.
- **Decision:** Range tiers — soldier (1) / archer (2) / mage-cultist (3) — each tier deleting a previously safe zone. Early enemies are **initiates**: recruits who haven't yet paid the flattening price; same silhouette (robes, masks), no magic. Art: one base sprite, weapon swap.
- **Rationale:** Spatial escalation is cheap and legible; the initiate tier preserves the mage reveal, adds sinister lore (some are still deciding), and keeps the mask clue chain intact.
- **Consequences:** Full cultists reserved for the countdown phase. Enemy dialogue follows the acknowledgment ladder: you → her → cult → ritual.

### ADR-017: Sigil tech tree as sole progression; z-essence as currency
- **Context:** XP (steady but stale grind), stat-choice (instant but mobile-feeling), or diegetic tech tree.
- **Decision:** The tree subsumes both. Currency: z-essence — dying cult members' severed z-axis is absorbed by the player. Nodes are **verbs, not percentages** (~10–15 total: 2/3-block attacks, dash, reduced lockout, third row…). Form: wooden neck sigil; the completed tree resolves into a winged figure — **reveal earned near the altar, never shown early**.
- **Rationale:** Every battle pays + branching runs + no grind; double juxtaposition (reject vs inherit a dimension; exploit vs become selflessness); percentage nodes are imperceptible in a 30–60 min game; small tree = every node felt.
- **Consequences:** Tree UI is Phase 1+, not slice. Cult victories literally feed their antithesis — writing must honor this.

### ADR-018: Keepsake unification
- **Context:** Beat 1 had "a keepsake from your sister"; ADR-017 created a worn sigil.
- **Decision:** Same object. She left it / gave it to the protagonist.
- **Rationale:** Emotional seed and progression system as one artifact, planted in the first thirty seconds; the slice can tease progression (essence thread, pendant warms) with zero UI.
- **Consequences:** Beat 1 flavor text must not hint at mechanics.

### ADR-019: Rainstorm as motivation and cult cover
- **Context:** "Empty room = panic" is not practical motivation; front exit needed a natural block.
- **Decision:** A storm the night before. She never came home through it. Storm damage blocks the main gate (mayor repairing); in hindsight, cult sabotage under weather cover. Flat plank inspectable for one eerie narrator line, justified by the sibling spiritual tether.
- **Rationale:** Real panic; natural player routing without visible rails; readable-later planning by the villains (player aha, character oblivious); one-line thesis foreshadowing at zero system cost.
- **Consequences:** Opening writing must sell "suspiciously normal morning."

### ADR-020: Flag-based context system; NPCs as data
- **Context:** Dialogue/world state needed; integer "knowledge level" vs booleans.
- **Decision:** Named boolean flags (`talked_to_vendor`, `knows_about_masks`, …). Dialogue entries carry flag conditions. NPC presence, obstacles, and day/night key off flags — **time advances with understanding, not wandering**. NPCs are data (sprite, position, condition→lines table) consumed by machinery.
- **Rationale:** Flags survive non-linear knowledge (the int would not); knowledge-driven daylight resurrects the one-day pacing and rhymes with the ending; data-driven NPCs make the dialogue system proto-engine (charter 2.2).
- **Consequences:** Flag set must be named/curated deliberately. Meta-content rules: period-appropriate jokes only (tulip mania, the red hood); scrapped premises become NPCs (courier, festival-goer, pilgrim).

### ADR-021: Silent protagonist; three-register text system; rationed 4th wall
- **Context:** Protagonist voice undefined; v1 mixed monologue and narration.
- **Decision:** Protagonist never speaks in dialogue. Three registers over one dialogue machinery: NPC dialogue (avatar, standard frame), inner monologue (avatar, thought-bubble frame), narrator (no avatar). Narrator 4th-wall tiers: T1 in-world texture (unlimited), T2 deniable winks (occasional), T3 full breaks (once or twice per game).
- **Rationale:** "Where are you?" cannot live in third person; registers are presentation parameters on shared machinery (cheap); the altar scene requires sincerity a habitual 4th wall would erode — a T3 break lands *because* it betrays an established voice.
- **Consequences:** Three frame styles + blip voices to design. Parked, not foreclosed: *who is the narrator?* (possible tether tie-in).

### ADR-022: Weapons as story milestones; no equipment system; staged journal
- **Context:** Sword→bow→staff sequence risked adding an equipment layer atop the tree; glossary implied an art bill.
- **Decision:** No equipment system. Weapons are story-granted, non-interactable milestones: the tree unlocks capacity (1/2/3-block attacks), the story hands the instrument (guard's sword first). HUD: sigil only (with tree-open hint); everything else in the pause menu (key items / map / tree / clue journal). Journal ships Phase 1 as text-only learned-facts list; portrait glossary is Phase 2.
- **Rationale:** Non-interactable icons ≠ a system; a parallel equipment layer would duplicate the tree and add UI/balance/choice overhead; weapons mirroring enemy tiers reinforces the essence-absorption fiction; the journal's need is real, its chrome is not.
- **Consequences:** "The weapon in your hand is the weapon you have" — no loadout decisions, ever, without a superseding ADR.

### ADR-023: Diegetic magical map
- **Context:** The player needs a map + a lore-sound way to get one.
- **Decision:** The defeated initiate drops a magical cult map marking the temple; it shows the *player's* position because the player is siphoning cult power. Exposition is show-then-tell: impossible behavior first, narrator explanation second.
- **Rationale:** Organic lore delivery over wall-of-text; the z-essence rule doing narrative work; consistent with ADR-008 (nothing shortens the walking).
- **Consequences:** Map tab justified; map behavior must track the essence fiction.

### ADR-024: Slice ⊂ Phase 1 (thin path / thickening model)
- **Context:** First-act design expanded far beyond the 7-beat slice; risk of the slice silently becoming a three-month milestone.
- **Decision:** The vertical slice is the *thin path through* Phase 1: wake → house → shortest clue chain → one fight (one initiate) → hook. Phase 1 thickens the same map: ambient NPCs, full chain, second enemy, menus, tree UI. Development proceeds in iterative phases.
- **Rationale:** Preserves ADR-002's protection; same content, two zoom levels; each phase ends playable.
- **Consequences:** Every feature is tagged slice / Phase 1 / later. Beats v2 table (Phase 7) carries the tags.

### ADR-025: No ECS for the slice; indices over references
- **Context:** ECS evaluation (formerly an open question) came due at the derivation pass; the slice has ~12 live entities.
- **Decision:** Simple structs + plain collections; entities addressed by index/ID, never by held references. No ECS crate. Re-evaluate at Phase 1 with real experience.
- **Rationale:** ECS solves problems the slice doesn't have. The borrow-checker fight over cross-entity references (enemy AI reading player position) *is* the core Rust-gamedev curriculum; adopting ECS first outsources the lesson. Rc/RefCell appearing in entity code is treated as a design smell (fighting the index rule).
- **Consequences:** Expect and welcome the "indices, not references" refactor around M5–M6 (budgeted ~1 day of confusion). A future ECS adoption would be a superseding ADR made from experience, not anticipation.

### ADR-026: Single crate; engine/game as modules
- **Context:** Workspace (multi-crate) vs single crate for the repo's first structure.
- **Decision:** One binary crate `tetherwood` with `src/engine/` and `src/game/` modules from day one. Workspace split deferred until extraction is real.
- **Rationale:** Honors "don't over-engineer repo structure" (charter). Module visibility boundaries enforce the machinery/content discipline; the crate split later is a folder move narrated by `refactor(engine): extract …` commits.
- **Consequences:** Module hygiene is the discipline that makes the eventual split cheap. Scope `engine` vs `game` in commit messages tracks the boundary from commit one.

### ADR-027: Static Rust data for slice content
- **Context:** Dialogue tables, NPC definitions, and enemy parameters need to live somewhere; serde+RON/JSON was the assumed default.
- **Decision:** Slice content is `const`/`static` Rust data structures in `src/game/`. No serde, no data files, no format decision yet.
- **Rationale:** Data-driven ≠ file-driven — the machinery consumes tables; the table's location is a detail. Defers a dependency and a format debate until content volume creates real pain (Phase 1 thickening).
- **Consequences:** serde/RON evaluated at Phase 1. Content edits require recompiles during the slice — acceptable at slice content volume.

### ADR-028: Authored scenes, not an isometric tile engine
- **Context:** "Isometric rendering" silently implied tile math, projection transforms, map formats, and tooling.
- **Decision:** A scene is a hand-authored background image + collision rectangles + y-sorted entity sprites. The isometric *look* is art direction; there is no isometric *engine* in the slice.
- **Rationale:** Classic adventure games shipped exactly this way; deletes an entire subsystem from slice scope. The renderer's slice job reduces to textured quads, draw order, and a camera.
- **Consequences:** Isometric tilemaps become a Phase 2+ evaluation *if ever needed*. Scene art (even placeholder rectangles) is authored per scene.

### ADR-029: Time model — frame delta + battle tick accumulator
- **Context:** ADR-015's hybrid design needed a concrete implementation model; full fixed-timestep machinery was the over-engineered default.
- **Decision:** Variable frame delta for overworld movement; accumulator-driven fixed tick (~0.5s) inside battle for enemy actions; player input sampled every frame everywhere.
- **Rationale:** Simplest model satisfying the design. Tick length is one constant (a named tuning knob).
- **Consequences:** Revisit only if determinism is ever genuinely required (unlikely — no physics, no netplay planned).

### ADR-030: Title — Tetherwood
- **Context:** Working title "The Morning She Was Gone" was adopted without brainstorming; stress-tested via a naming session. Criteria that emerged: layered post-credits re-reading; revelation energy (not secrecy — secrecy is the villains' mode); shaped as a tiny story or as the driving artifact (the Ocarina pattern); weighty; not on-the-nose.
- **Decision:** The game is **Tetherwood** — repo `tetherwood`, Cargo package `tetherwood`. The former working title survives as the **Act I / vertical slice chapter title**: *"The Morning She Was Gone."* The engine remains deliberately unnamed (OQ).
- **Rationale:** Names the central artifact (the wooden sigil is literally a tether: sibling bond, z-essence absorption, the map's obedience, her link to the ritual, and the parked narrator question). The "-wood" family (driftwood, heartwood, wormwood) makes the coinage read as a natural word; the game cashes the title from Beat 1, defusing the try-hard risk. Consistent with the developer's naming voice (cf. *Isolated*). Collision check: a 2-day itch.io jam prototype shares the name — assessed as soft (no commercial presence, no apparent trademark); accepted. Proper trademark search deferred to any commercial release.
- **Consequences:** Repo/crate named before `git init` — no rename churn. The window title in M1 is `Tetherwood`. Chapter-title convention established (acts may carry their own titles).

### ADR-031: 2D world coordinates — pixels, top-left origin, y-down
- **Context:** wgpu's clip space is y-up, origin-center, −1..1 — standard
  for 3D engines but foreign to every 2D authoring surface the project
  actually uses (images, UV space, Aseprite, Godot).
- **Decision:** Tetherwood's world space is pixels, origin top-left,
  y-down. The projection matrix (built via `glam::Mat4::orthographic_rh`
  with bottom/top swapped) is the single place the y-flip into clip
  space happens.
- **Rationale:** Matches every 2D surface the developer already thinks
  in; makes M3's y-sort read naturally (greater y = lower on screen =
  drawn in front, matching draw-order intuition). The alternative
  (native y-up) buys nothing for a 2D adventure game and costs a mental
  flip at every sprite, every scene, forever.
- **Consequences:** The y-flip reverses on-screen triangle winding,
  forcing ADR-034 (culling disabled). Any future code reading raw clip-
  space or NDC values (shouldn't be common) must remember the flip lives
  only in the projection matrix, not scattered through game code.

### ADR-032: Transform chain — projection * view * model, rebuilt per frame
- **Context:** Sprites need independent position/size (model), the
  camera needs to offset the whole world (view), and pixel space needs
  to become clip space (projection). Needed a combination order and a
  recomputation policy.
- **Decision:** `transform = projection * view * model`, matrices
  multiplied right-to-left (model applies first, then view, then
  projection). All three rebuilt from scratch every frame, uploaded via
  `queue.write_buffer` each `render()` call — no dirty-flag/caching.
- **Rationale:** Right-to-left order is the only order that produces
  correct results (translating before scaling blows up position, as
  discovered by hand-tracing coordinates during derivation). Rebuilding
  every frame is negligible cost at slice scale (single sprite, few
  matrix multiplies) — correctness and simplicity beat an unearned
  optimization. `view` implements the camera as "shift the whole world
  opposite the camera's motion," the standard fake since there is no
  literal lens.
- **Consequences:** Revisit only once entity count makes per-frame
  matrix rebuilding for every entity a measured cost (Phase 1+, not
  before). `view`'s screen-center offset makes `camera_position` mean
  "the world point mapped to screen center," not "world point mapped to
  top-left" — a deliberate, documented convention (see ADR-033 for the
  matching decision on entity position).

### ADR-033: Entity position means center, not top-left corner
- **Context:** The model matrix's translate initially placed the unit
  quad's top-left corner at `sprite_position`. Camera-centering testing
  surfaced that this convention put the sprite's visual center off-
  screen-center by half its size — a CSS-instinct fix (offset the
  camera by half-size) was considered and rejected in favor of fixing
  the convention at the source.
- **Decision:** `sprite_position` (and any future entity position field)
  means the entity's visual center. The model matrix subtracts half the
  sprite's size before translating, so the corner-vs-center conversion
  happens once, at construction, rather than being compensated for at
  every consumer (camera, collision, etc).
- **Rationale:** Center is the convention every downstream system will
  want: collision checks, facing direction, distance-to-player for
  enemy AI (M6), y-sort ordering (M3) all reason about "where is this
  entity" most naturally as a single center point. Fixing the meaning
  once beats every future system re-deriving a half-size offset
  independently. Rejected alternative (offset only at the camera step)
  would have left the top-left convention intact and forced every other
  consumer to repeat the same correction.
- **Consequences:** Any future position field on any entity follows this
  convention by default — worth stating explicitly so M3's entity model
  doesn't have to rediscover it. Sprite size must be known wherever a
  position is placed (already true — read from `texture::Texture`).

### ADR-034: Back-face culling disabled for 2D sprites
- **Context:** The pipeline's `cull_mode: Some(Face::Back)` was set
  during the triangle/quad chunks, before ADR-031's y-down projection
  existed. The y-flip reverses screen-space winding for every quad,
  which the existing cull setting would silently discard.
- **Decision:** `cull_mode: None`.
- **Rationale:** Back-face culling exists to skip triangles facing away
  from the camera in closed 3D meshes — a screen-aligned 2D sprite quad
  has no "back" to skip; the optimization doesn't apply to this
  project's geometry at all, independent of the winding issue it
  happened to also fix.
- **Consequences:** None expected — this is the standard setting for 2D
  renderers built on 3D-oriented APIs like wgpu.

### ADR-035: Raw key handling in App is temporary; input abstraction deferred
- **Context:** Held-state WASD movement is currently implemented as
  direct `KeyCode` checks inside `App` (platform-adjacent code), which
  fails the machinery/content test — a different game built on this
  engine could not remap controls without editing this code.
- **Decision:** Ship the raw version for M2. Defer building an
  Action/InputMap abstraction layer (engine exposes abstract actions;
  game supplies the key mapping and the meaning) until a second real
  input-consuming system exists — candidates: M4 dialogue advance/skip,
  or menu navigation.
- **Rationale:** Same reasoning as ADR-025's ECS deferral: designing the
  abstraction now means guessing its shape from a single data point
  (movement only). Real design questions are still open — does battle's
  hybrid real-time/tick input (ADR-015) even fit the same action model
  as overworld movement? A second consumer will answer that; anticipating
  it won't. Building it now would be premature generalization against
  Principle 5.
- **Consequences:** A `// TODO(engine): ...` marker is left at the raw
  `KeyCode` handling site in `App`, naming this ADR, so the debt surfaces
  in-editor rather than relying on memory. Flagged explicitly here so a
  future chat doesn't mistake "not yet built" for "not noticed."

### ADR-036: Assets referenced by index into a scene-scoped store
- **Context:** M3's multi-entity scene needs shared textures (two beds use one sprite). ADR-025 established indices-over-references for entities but never covered asset ownership.
- **Decision:** Textures are owned exclusively by a `TextureStore` (`Vec<Texture>`), scene-scoped, loaded once at scene construction and never mutated during play. Entities/walls reference textures via `TextureId` (a `Copy` newtype wrapping `usize`), never by direct reference or `Rc`.
- **Rationale:** Extends ADR-025's reasoning to assets — sharing must not require lifetimes or reference counting; an append-only `Vec` plus plain copyable indices is the cheapest correct mechanism.
- **Consequences:** `TextureStore` must never remove/reorder entries during a scene's lifetime, or indices silently point at the wrong texture — this is a load-order *policy*, not a compiler-enforced guarantee. The renderer builds one `wgpu::BindGroup` per texture at scene-load (`prepare_scene`), indexed identically to `TextureId`.

### ADR-037: Entities are deactivated, never removed, during a scene's lifetime
- **Context:** Beats needing an entity to "go away" (defeated enemy, consumed key item, opened door) raised the question of shrinking `Scene.entities`, risking invalidation of `player_index` or any other stored index.
- **Decision:** `Scene.entities` only ever grows during play. An entity that should disappear is deactivated instead: `texture_id: None` (invisible), `collider: None` (non-solid) if applicable. True removal (safe reindexing, stable handles) is deferred to the ECS re-evaluation already scheduled for Phase 1 (ADR-025).
- **Rationale:** Mirrors the asset-store immutability policy (ADR-036) — a fixed-during-play `Vec` means any stored index stays valid for the scene's whole lifetime, at zero bookkeeping cost. Verified against two real slice cases (defeated cult initiate, Beat 1 keepsake pickup) without new machinery.
- **Consequences:** Every entity's `Option` fields double as a small state machine. True deletion remains a named, deliberately parked problem, not a forgotten one.

### ADR-038: Two-source AABB collision, resolved as sequential per-axis proposals
- **Context:** M3 needed a concrete collision model spanning two data sources (scene walls, entity colliders) and a rule for diagonal movement against corners/edges.
- **Decision:** Walls are plain `Vec<Rect>` in world-space; entity colliders are `Rect`s offset from `entity.position`. Both feed one `aabb_overlap` test, source-agnostic. Movement resolves by proposing the x-move alone, accepting/refusing it, then proposing the y-move from the resulting position — never as one combined 2D proposal.
- **Rationale:** AABB overlap reduces to a per-axis center-distance-vs-summed-half-extents comparison (hand-derived and verified this session). Sequential per-axis resolution produces sliding along an edge on diagonal movement; hand-traced against inside-corner cases to confirm no ordering bug and no tunneling-through-unvalidated-position hole.
- **Consequences:** Wall thickness is a tuned safety margin against tunneling (frame movement distance vs. thickness), not derived from entity size.

### ADR-039: Y-sort by baseline, not entity center
- **Context:** M3's y-sort needed a concrete key. Center-y was hand-tested against tall furniture (wardrobe) and found to produce wrong draw order.
- **Decision:** Entities draw in ascending order of baseline = `position.y + size.y / 2.0` (the sprite's bottom edge).
- **Rationale:** "In front of" is best modeled by where an entity touches the ground, not its visual center. Confirmed against both same-height (bed) and taller (wardrobe) cases.
- **Consequences:** Feet-only collider boxes (deliberately smaller than sprite bounds) are designed to work *with* this rule — the two decisions produce the walk-behind-furniture effect together.

### ADR-040: Per-draw command submission for multi-entity frames
- **Context:** M2 wrote one transform and submitted once per frame — safe for exactly one sprite. M3 needed several independently-positioned entities drawn in one frame.
- **Decision:** Each draw (background + every visible entity, y-sorted) gets its own command encoder and its own `queue.submit`, immediately following its own transform-buffer write. First draw clears; subsequent draws load (paint over).
- **Rationale:** `queue.write_buffer` uploads independent of submission timing — multiple writes before one shared submit would let every draw in that submission read the last-written transform only, silently misplacing every sprite. Per-draw submission guarantees correctness. A shared uniform buffer with per-draw dynamic offsets is the more scalable fix, but is unjustified machinery at slice scale (~a dozen draws/frame).
- **Consequences:** Revisit only if draw count or submission overhead becomes a measured cost (Phase 1+), same deferral pattern as ADR-032.

### ADR-041: Camera mode is a per-scene, authored choice — static anchor vs. follow
- **Context:** M2 shipped follow-camera as the only behavior. M3's indoor bedroom raised whether every scene should behave identically.
- **Decision:** Camera behavior is chosen per scene, as design intent. Indoor scenes use a static camera anchored at an authored focal point (the bedroom anchors at its own center; a future multi-room house would anchor at its connecting corridor). Outdoor/large scenes use M2's follow-mode. No `CameraMode` abstraction exists yet — the bedroom's static camera is currently just `camera_position` set once at scene load, never updated per frame — deferred until a second concrete camera behavior is needed side-by-side with the first (same reasoning as ADR-035).
- **Rationale:** Gives the indoor/outdoor distinction narrative weight (cozy, fully-visible interiors vs. vast, player-centered exteriors), rather than treating camera behavior as an incidental default.
- **Consequences:** A real `CameraMode` type is expected, scoped for whenever the first outdoor/follow scene is built alongside the first static one — not before.

### ADR-042: multiplying_factor scales the whole scene composition uniformly
- **Context:** Content is authored at raw, small pixel dimensions (matching source art and hand-derived layout math) but needs visual scale-up for legibility. An early attempt scaled only sprite/collider size while leaving position and the room boundary fixed, causing furniture to visually drift out of proportion with the (unscaled) room as the factor changed.
- **Decision:** `multiplying_factor` is applied uniformly to every position, size, and collider offset/half_size — entities, walls, and background alike — from world origin, at scene-construction time (`Scene::new_bedroom(..., multiplying_factor)`). It also scales player movement speed, keeping felt movement speed (body-lengths per second) constant across zoom levels. The factor lives as an `App` field (not a local variable), since both scene construction and per-frame movement code need to read it.
- **Rationale:** Scaling the whole composition from one shared origin preserves every placement ratio (gaps, wall clearance, walk-behind spacing) exactly, regardless of factor — layout only needs tuning once, at any single factor, decoupling "is the layout correct" from "does it look good at this zoom level." Matches the Photoshop-group-scale mental model explicitly adopted over per-component scaling.
- **Consequences:** Every literal in scene-construction content carries an explicit `* multiplying_factor` — a real, visible verbosity cost, accepted as a consequence of static Rust content (D-C) rather than data-driven loading, where the multiply would live in one place. Revisit if/when content moves to serde/RON (Phase 1+). The viewport itself does not auto-scale with the factor — "let content drive size" (already decided) means viewport bumps remain a separate, deliberate call once content visibly outgrows the current window.

### ADR-043: Debug collider overlay via a dedicated shader, not a stretched texture
- **Context:** M3 needed a way to visualize (normally invisible) wall and entity colliders during placement tuning. A first attempt used a small texture with a baked-in border, stretched over each collider's world-space size — but stretching means border pixel-width scales with rect size, giving visibly uneven borders on non-square rects. This overlay is intended as permanent, reusable engine machinery (Unity/Godot-style gizmo drawing), not a throwaway M3 debugging trick, so the imprecision was worth fixing properly rather than accepting.
- **Decision:** A second, dedicated render pipeline (`debug_shader.wgsl`) draws fill + border directly from each fragment's UV coordinate, with no texture involved. A per-draw uniform (`DebugRectUniform`: fill color, border color, border thickness) is computed fresh for each rect — border thickness is specified once in pixels and converted to UV space **per axis, per draw** (`px / rect_width`, `px / rect_height`), since UV space is stretched independently per axis and a single shared thickness value would produce a mismatched border on any non-square rect. The corner case requires no special handling: a fragment near two edges simultaneously satisfies two of the four border conditions at once, and the existing OR already covers it.
- **Rationale:** True constant-pixel-width borders, at any rect size, are only achievable by computing the border test against real dimensions at draw time — a texture's fixed pixel grid cannot represent this regardless of resolution. The pipeline reuses the existing quad geometry and `Vertex` layout entirely; only the shader and its bind group layout differ from the textured pipeline (group 0 holds `DebugRectUniform` instead of texture+sampler; group 1's transform binding is shared unchanged between both pipelines).
- **Consequences:** A new uniform buffer + bind group is built per debug rect, per frame, while the overlay is active — an accepted, deliberate cost at slice-scale rect counts (a dozen or so), not the "cheap, rebuild every frame" territory ADR-032 justified for the transform buffer alone. Revisit (e.g., batch into one buffer) only if rect count or overlay-on frame cost becomes measurably relevant (village-scale content, Phase 1+). A third debug color (for interactable entities, e.g. the door) is deferred until an `Entity` field distinguishing interactables actually exists — coloring decisions follow data, not the reverse.

### ADR-044: Scene stays a concrete struct for M4; trait + stack deferred to M6
- **Context:** DERIVATION's E4 specifies "scene trait, stack (push battle over overworld)" — but M4 only needs a lateral room-to-room swap, not a push-with-preserved-underlying-state. Building the trait+stack now means designing it from one data point (lateral swap) and guessing whether that shape also serves M6's push case.
- **Decision:** M4 implements transition as a single swap-slot: `App` holds one concrete `Scene`; transitioning replaces its value outright. No `Scene` trait, no `Box<dyn Scene>`, no stack. The trait + stack are deferred to M6, when the battle-over-overworld push provides a second real, concrete shape to design against.
- **Rationale:** Same reasoning as ADR-035 (input abstraction) and ADR-041 (camera mode) — defer generalization until a second concrete consumer exists to teach the correct shape, rather than anticipating it from Beat 6's spec alone. A swap-slot is strictly simpler than and safely upgrades to a stack-of-one later.
- **Consequences:** M6 is now explicitly scoped to include designing `Scene` as a trait with push/pop semantics, informed by two real consumers (overworld, battle) rather than one. Revisit only at M6, not before.

### ADR-045: Room transitions are automatic zone-triggered, no button prompt
- **Context:** Two viable models existed for room-to-room transition: button-gated ("Press X to exit," consistent with the existing interact-verb design) vs. automatic walk-through (GBA-era RPGs, Mega Man Battle Network).
- **Decision:** Room transitions fire automatically on proximity alone — walking into a doorway triggers the transition, no button press required.
- **Rationale:** Matches the developer's explicit MMBN-lineage instinct (ADR-009's design lineage) and reads smoother in the isometric-styled presentation; opens the door to creative exit visuals later (a shape sticking out of the room boundary marking the exit, Mega Man-style) without requiring separate button-prompt UI.
- **Consequences:** Establishes a second trigger flavor distinct from the existing interact verb (see ADR-046). Creative exit-visual treatment is left open, not designed yet.

### ADR-046: Two trigger flavors; trigger dispatch as single-variant enum
- **Context:** ADR-045 created a proximity-only trigger, functionally different from the existing interact trigger (proximity + facing + button, E8). Separately, a "trigger points to an arbitrary function" dispatch mechanism was floated (brainstorm) and evaluated against building infrastructure for trigger kinds that don't exist yet.
- **Decision:** Two trigger flavors coexist: **interact triggers** (existing — proximity + facing + button; examine, talk, pick up) and **zone triggers** (new — proximity alone; scene transitions, and later, ambient effects). Both share the same underlying `aabb_overlap` check; only the firing condition and the resulting action differ. Trigger dispatch is a single-variant enum — `TriggerKind::Warp { target_scene, target_warp_id }` — not a generic callback/function-pointer mechanism. `Trigger` wraps the existing `Rect` (offset + half-size), adding meaning rather than new geometry.
- **Rationale:** Same defer-until-second-consumer instinct as ADR-025/035/041/044 — a generic dispatch shape designed from one use case (warp) risks guessing wrong about what a second trigger kind (cutscene start? music zone? M5 clue trigger? M6 encounter trigger) actually needs. A `Rect`-wrapping enum costs nothing to extend when that second case arrives.
- **Consequences:** Not every `Rect` in a scene is a collider — `Scene` now conceptually separates solid geometry (walls, `Vec<Rect>`) from trigger geometry (`Vec<Trigger>`), never merged into one list. `TriggerKind` gains variants (and possibly a real dispatch shape) only once a second concrete trigger kind is being built, not before.

### ADR-047: Warp pairs, not per-scene spawn points
- **Context:** A transitioning player needs to land somewhere sensible in the destination scene — not its arbitrary default spawn, and specifically at the point corresponding to the door they used. The originally proposed model (named `SpawnPoint`s per scene, keyed by "came from X") requires every scene to anticipate every scene that might lead into it — an N² relationship as door count grows.
- **Decision:** Adopt Pokémon-style warp pairs: a `Trigger` with `TriggerKind::Warp` carries a `(target_scene: SceneId, target_warp_id: WarpId)` pointer to its partner warp. Warps are placed as scene content, wired together as pairs; no scene needs to know about any other scene's structure beyond the pair it's directly connected to. `SceneId` is a plain enum (`Bedroom`, `Hallway`, …) per D-C's static-content approach; `WarpId` is a small unique-per-scene identifier.
- **Rationale:** Scales flat — N doors is N warp pairs, not a combinatorial spawn-point table — and matches a proven genre pattern (Pokémon, MMBN) for exactly this problem. Simpler than the spawn-point alternative it replaces, not just different.
- **Consequences:** Every scene construction function needs a way to place warp triggers as content, same as it already places entities and walls. `SceneId`/`WarpId` become the addressing scheme for cross-scene references, extending the indices-over-references principle (ADR-025, ADR-036) to scene/warp identity.

### ADR-048: Scene persistence via the flag store; GPU resources lazy-unloaded and reloaded
- **Context:** Two competing concerns: keeping every scene resident in memory (safe for narrative persistence — taken items, defeated enemies stay gone — but a real, avoidable GPU/memory cost at any scale beyond the slice) vs. fully lazy loading (cheap, but naively re-running scene construction on revisit would respawn everything, Zelda-dungeon-style — explicitly the wrong feel for this game).
- **Decision:** Separate the two concerns instead of trading one for the other. GPU-heavy resources (`TextureStore`, bind groups, entities) are fully lazy: unloaded on scene exit, reconstructed fresh via the scene's construction function on re-entry. Narrative state (item taken, enemy defeated) is not stored on the scene at all — it lives in ADR-020's existing flag store, owned above individual scenes (at `App` level), which persists across transitions. Scene construction functions read relevant flags while placing entities and deactivate (ADR-037: never remove, only deactivate) anything the flags say should already be gone.
- **Rationale:** The two concerns don't actually need to move together — GPU footprint is legitimately expensive and safe to discard; the state worth remembering is a handful of booleans. Reusing ADR-020's flag store means zero new persistence machinery — a defeated enemy or a taken item is the same kind of fact as `knows_about_masks`, just consumed by scene construction instead of dialogue conditions. Reconstruction is deterministic given the flags, avoiding the Zelda-respawn problem without keeping scenes resident.
- **Consequences:** Scene construction functions (`new_bedroom`, `new_hallway`, …) grow a dependency on flag-store state, not just `multiplying_factor` and device/queue as today. The flag store's lifetime now spans the whole `App`, not any single scene — it must be constructed before the first scene and survive every subsequent transition.

### ADR-049: Aseprite native file loading — parked, not scheduled
- **Context:** Raised as a workflow-friction question (skip the PNG-export step, decode `.aseprite` directly). Evaluated as genuinely feasible via the `asefile` crate, fitting the existing loader machinery (`TextureStore`/`Texture::from_bytes` pattern) with no changes to `Entity`, `Scene`, or `TextureStore`'s shape for the single-frame case.
- **Decision:** Not built now. Deferred until PNG-export friction becomes a real, recurring workflow cost rather than a curiosity — explicitly not scheduled for any current milestone.
- **Rationale:** Zero dependency from M4 (scene trait/stack — now deferred per ADR-044 — text rendering, dialogue) or any milestone through at least M6. The full-featured version (pulling animation frames/layers directly via Aseprite's frame tags, e.g. future walk-cycles) is a genuinely bigger feature than a format swap and reopens frame/layer-selection questions not yet faced — worth its own design pass if/when pursued, not a silent default.
- **Consequences:** Recorded here specifically so the parked state is discoverable in a future session rather than re-litigated from scratch. If picked up later, scope must be declared explicitly as either "single-frame format swap" or "multi-frame/layer animation loading" — the two have very different costs.

### ADR-050: Aseprite native file loading — implemented (supersedes ADR-049)
- **Context:** ADR-049 parked native `.aseprite` loading, deferring it until PNG-export friction became a real, recurring cost rather than a curiosity. That threshold was reached during M4 scene-content work.
- **Decision:** `TextureStore::load` dispatches on file extension: `.aseprite`/`.ase` files decode via a new `Texture::from_aseprite` (using the `asefile` crate), everything else continues through the existing `Texture::from_bytes` path. Both converge on the same `from_image` GPU-upload code — no changes to `Texture`, `TextureStore`'s shape, or any downstream consumer. `from_aseprite` takes frame 0, fully composited across all visible layers (matching what manual PNG export already produced), and logs a warning naming the frame count when a loaded file has more than one frame, so a multi-frame file loaded as a static texture is never silently wrong.
- **Rationale:** `asefile` was chosen over the alternative `aseprite-reader` crate after comparing maintenance signals: ~10x the downloads, actively versioned within the last ~2 years vs. ~4, and no coupling to a game engine (Bevy) this project doesn't use. Neither crate is authored by Aseprite itself, despite both crates' descriptions reading that way at a glance — the phrase refers to the file *format's* origin, not the crate's authorship.
- **Consequences:** Content authored in `.aseprite` no longer requires a manual PNG export step before `cargo run` picks it up — confirmed end-to-end (edited a scene's `.aseprite` source directly, reloaded, updated art appeared with no export). Existing PNG-sourced assets are unaffected and continue to load via the original path; migrating them to `.aseprite` sources is optional, not required by this change. `asefile` is now a project dependency.

### ADR-051: Warp identity as named strings; per-trigger reentry suppression; Scene self-identity
- **Context:** Building the actual door/warp mechanic surfaced three
  related gaps the original Trigger design (ADR-046, ADR-047) hadn't
  covered: (1) nothing prevented a trigger from re-firing the instant
  the player spawned into the destination scene, still standing on the
  arrival trigger; (2) nothing let a resolved warp find its named
  partner trigger inside a freshly, lazily constructed destination
  scene's trigger list — TriggerKind::Warp carried a target_warp_id but
  no trigger declared its *own* warp_id to be matched against; (3) a
  human-readable debug label (e.g. for a future "Home:door ->
  Outside:door" print) either had to be duplicated across both ends of
  a warp pair (rejected — a stale-copy risk) or resolved through a
  separately maintained name registry (rejected — a decoupled,
  forgettable second file for what should be one authoring step).
- **Decision:** `WarpId` (engine/ids.rs) changes from a `u32` newtype
  to `pub struct WarpId(pub &'static str)` — amending ADR-047's
  original concrete type for WarpId, whose scene/pairing design is
  otherwise unchanged. `TriggerKind::Warp` gains its own `warp_id`
  field alongside `target_scene`/`target_warp_id`. `Trigger` gains
  `recently_used: bool`, cleared once the player's center leaves that
  specific trigger's rect (not a scene- or App-wide flag). `Scene`
  gains `pub id: SceneId`.
- **Rationale:** `WarpId` was always matched associatively (find the
  trigger whose id equals X) never indexed into a Vec — unlike
  `TextureId`, which genuinely needs to be numeric because it indexes
  `TextureStore` directly. Treating `WarpId` as needing numeric
  consistency with `TextureId` was a pattern-matched-on-the-wrong-
  similarity mistake, corrected this session. A string identifier
  authored once, at a trigger's own construction site, serves as both
  the routing key and the debug label with a single source of truth —
  no separate name registry, no field that can drift out of sync with
  its counterpart. Per-trigger (not global) reentry suppression means
  only the specific door just used is briefly inert, so a future scene
  with two doors close together is correct by construction rather than
  by a case someone has to remember to handle later.
- **Consequences:** A typo'd `WarpId("doo")` still compiles and fails
  silently at runtime (no compiler exhaustiveness check, same
  limitation the prior `u32` form had) — accepted, see ADR-052.
  `SceneId` variants (currently `Bedroom`/`Hallway` in code) still need
  renaming to `Home`/`Outside` to match `new_home`/`new_outside` — not
  yet done as of this entry, first step of the next implementation
  session.

### ADR-052: Startup warp-pair validation — proposed, explicitly deferred
- **Context:** ADR-051's `WarpId` as a bare string has no compiler-
  enforced exhaustiveness — a typo'd warp id compiles cleanly and fails
  silently at runtime (the door simply does nothing), discoverable only
  by manually walking through that exact doorway during play.
- **Decision:** Not built now. A validation pass — at startup or as a
  test, constructing every scene and confirming every `Warp` trigger's
  `(target_scene, target_warp_id)` resolves to a real trigger somewhere
  — is proposed but explicitly deferred until warp count is large
  enough (a populated village, multiple houses) that manual playtesting
  stops being a reliable way to catch this class of typo.
- **Rationale:** At current content volume (two scenes, one warp pair),
  the validator would cost real effort to catch a mistake trivially
  caught by playing the game once. Matches the project's established
  pattern (ADR-025, 035, 041, 044, 046, 049) of deferring machinery
  until the problem it solves is real rather than anticipated.
- **Consequences:** Recorded here specifically so it's discoverable
  later rather than re-derived from scratch once warp count actually
  makes it worth building.

### ADR-053: App state restructured to Option<AppState>, resumed()-driven
- **Context:** Earlier scaffolding had drifted window/renderer/scene
  construction out of resumed() and into run(), called before
  event_loop.run_app() started — using EventLoop::create_window
  directly, which triggered a deprecation warning
  ("use ActiveEventLoop::create_window instead") and departed from the
  winit ApplicationHandler lifecycle pattern this project committed to
  at M1 (window creation via ActiveEventLoop, inside resumed()). The
  original motivation was avoiding Option<Renderer>/Option<Scene>
  unwrap ceremony scattered through every field access.
- **Decision:** App holds a single `state: Option<AppState>`, where
  AppState bundles every field that only exists once a window does
  (window, renderer, scenes, current_scene, input state, settings).
  resumed() is guarded (`if self.state.is_some() { return }`) and is
  the sole place AppState is constructed, using
  ActiveEventLoop::create_window. Every other handler starts with
  `let Some(state) = &mut self.state else { return }`, then works with
  bare, non-Option fields for the rest of the function body.
- **Rationale:** Preserves the actual goal (no unwrap ceremony in the
  hot path) while keeping resumed() as the correct, guarded entry
  point for GPU/window setup — the guard costs one let-else per
  handler, materially less than five separate per-field unwraps, and
  remains correct if resumed() ever fires more than once (a real,
  documented possibility on some platforms, e.g. Android's
  window-reclaim-on-background, even though this project's
  desktop-only target makes that case unlikely to ever fire in
  practice).
- **Consequences:** Resolves the EventLoop::create_window deprecation
  warning. A parked idea from the same discussion — a persistent,
  always-in-memory "top-level" struct holding cross-scene resources —
  was evaluated and found to already be satisfied by AppState itself;
  no further abstraction is needed until a concrete second consumer
  (e.g. the flag store, ADR-020/048) actually requires one.

### ADR-054: Warp spawn position decoupled from trigger detection geometry
- **Context:** Implementing spawn positioning (Trigger's rect.center,
  via the destination trigger found in Scene::activate_warp) landed
  the player exactly on the trigger's center on every arrival — visibly
  wrong (standing on top of the door sprite). Separately, before this
  positioning existed at all, testing surfaced that a scene's player
  position was left stale across repeat visits, producing
  visit-count- and direction-dependent spawn locations — a consequence
  of scenes being cached rather than reconstructed on re-entry (see
  ADR-048; not yet resolved, see Current State).
- **Decision:** TriggerKind::Warp gains a `spawn_offset: Vec2` field.
  Scene::activate_warp returns `trigger.rect.center + spawn_offset`
  rather than the bare center; the caller writes this into the
  player's position unconditionally on every warp arrival.
- **Rationale:** A trigger's rect already serves a specific, different
  purpose (how large an overlap counts as "arrived") from where a
  player should visually land — conflating them (e.g. by moving or
  resizing the trigger rect itself) would fix one case at the expense
  of the other, since detection and arrival need independent tuning
  per door, in a direction specific to that scene's geometry. Writing
  the position unconditionally on every arrival — not just once, at
  scene construction — is what actually fixes the stale-position bug,
  independent of what value is written.
- **Consequences:** Every Warp trigger must now specify a spawn_offset
  explicitly at construction (no default) — a small, deliberate
  authoring cost per door. The stale-position bug this fix incidentally
  resolved is a symptom of the still-open scenes-cached-forever
  question (ADR-048); spawn_offset does not resolve that question, it
  only ensures position is freshly written regardless of how it's
  answered.

### ADR-055: Main menu and pause menu — deferred, blocked on text rendering
- **Context:** With scene transitions now working between two real
  scenes, the lack of any menu (the game currently starts directly in
  Home and exits only via Escape or killing the process) was raised as
  a gap worth closing before more scenes accumulate.
- **Decision:** Not built now. No beat, milestone, or prior ADR ever
  scoped a main menu into the vertical slice — DERIVATION's slice
  exclusions explicitly name pause-menu tabs as out of scope, and a
  main menu was never mentioned at all. Both remain deferred until
  text rendering (E6) exists, since a menu's minimum viable form
  (selectable text options) depends on the same subsystem M4's dialogue
  work already requires.
- **Rationale:** Building placeholder menu art now would either fake
  real text or duplicate work text rendering is about to make trivial;
  better to build both together. A real open question — whether a menu
  screen is a Scene in the current sense (which assumes a player,
  walls, entities) or a distinct concept — is also better answered once
  there's an actual text-rendering-capable scene to design it against,
  rather than guessed at now.
- **Consequences:** Recorded here so the need is discoverable and not
  rediscovered from scratch once text rendering lands — at that point,
  a main menu (and pause menu) become natural, low-risk additions to
  design alongside dialogue's own text needs.

### ADR-056: Pre-rendered bitmap font atlas over runtime glyph rasterization
- **Context:** Text rendering needed a way to get glyphs from a font onto the GPU. Two broad approaches exist: pre-rendered bitmap atlas (one texture, every glyph baked in, sliced by fixed or metadata-driven coordinates) vs. runtime rasterization of vector fonts (e.g. via fontdue/ab_glyph, parsing .ttf and rasterizing to a dynamic atlas at load or first-use).
- **Decision:** Pre-rendered bitmap atlas. Specifically, a hand-arranged uniform grid (Good Neighbors font, CC0, sourced from OpenGameArt) with no accompanying metadata file — glyph position computed by arithmetic from an explicit character-to-cell lookup table, not read from a packed atlas's `.fnt`/similar format.
- **Rationale:** The project's committed visual identity (GBA/SNES-era pixel art, ADR-008) doesn't call for arbitrary fonts, sizes, or localization into scripts a fixed bitmap couldn't cover — runtime rasterization solves problems this game doesn't have, the same reasoning pattern behind every other deferred-abstraction ADR this project has made (025, 035, 041, 044, 046). A uniform grid specifically (over the font's originally-released packed/variable-width layout) was chosen to avoid writing and maintaining a `.fnt`-format parser — a real, avoidable subsystem — in exchange for a small, one-time authoring cost (redrawing the font into a fixed grid) and slightly higher texture memory (empty padding around narrow glyphs).
- **Consequences:** Every glyph occupies an identical cell regardless of its actual width — correct, expected monospace behavior, not a bug (visibly confirmed via the F3 test string's wider gap after narrow characters like ','). Adding a font with proportional/variable-width glyphs later, or supporting a script this grid doesn't cover, would need new work; not anticipated or designed for now.

### ADR-057: Renderer::render() split into acquire/render_scene/render_text/present
- **Context:** Text needs to draw as a second, later pass into the exact same on-screen frame the scene just drew into. The prior single `render()` method acquired a swapchain frame, drew the scene, and presented it as one atomic unit — calling it twice per game-frame (once for the scene, once for text) would acquire two *different* swapchain buffers (the surface is double/triple-buffered), causing one buffer to show the scene without text and the next to show text without the scene, flickering between them.
- **Decision:** `Renderer::render()` is replaced by four methods: `acquire_frame` (returns a `Frame` bundling the acquired `SurfaceTexture` and its `TextureView`, or `None` for transient not-ready cases), `render_scene` and `render_text` (each take `&Frame` and draw into it, order-dependent — text draws after and paints over the scene), and `present_frame` (consumes the `Frame`, presents it). The caller (`platform.rs`) orchestrates: acquire once, draw scene, draw text (when there's text to draw), present once.
- **Rationale:** Splitting frame *acquisition* from frame *drawing* is the standard fix for "multiple draw passes, one presented frame" — draw calls become composable (any number of passes can target one `Frame`) without each needing its own acquire/present pair. `render_scene`/`render_text` no longer return `Result`, since the only fallible step (surface acquisition) now lives solely in `acquire_frame` — drawing into an already-acquired frame can't itself fail in any case this codebase currently handles.
- **Consequences:** Any future draw pass (e.g. a pause menu overlay, once built) follows the same pattern — take `&Frame`, draw into it, let the caller sequence and present. `platform.rs`'s `RedrawRequested` handler is now the single place frame ordering is decided.

### ADR-058: Text rendering is screen-space only
- **Context:** The existing sprite transform chain (`projection * camera_view * model`, ADR-032) shifts every drawn position by the camera's current offset — correct for anything that exists *in* the game world, but dialogue/narrator text is a UI element that should stay fixed to the window regardless of where the camera is looking or which CameraMode (ADR-041/per-scene) is active.
- **Decision:** `render_text` composes its transform as `projection * model` only — no `camera_view` term. Text coordinates are always relative to the screen, never the game world.
- **Rationale:** This is the real, current need (Beat 2's narrator/dialogue text); a hypothetical future need for world-anchored text (e.g. a floating damage number over an enemy) would be a genuinely different consumer with different requirements, worth its own method or mode flag if and when it's real — not something to design speculatively now, matching this project's consistent pattern.
- **Consequences:** `render_text` cannot currently be used to draw text that should move with the camera. If that need arises, it would need a second code path or a mode parameter, not a change to this one.

### ADR-059: Facing as a four-way enum, not a raw Vec2
- **Context:** Interact triggers (the bed's directional examine requirement) needed a way to check "is the player looking the right way." A raw Vec2 (e.g. the last nonzero movement vector, normalized) would work mathematically but requires memorizing which vector means which direction at every call and content-authoring site.
- **Decision:** `Entity.facing: Direction`, a plain `enum { Up, Down, Left, Right }`. `Direction::from_movement(Vec2) -> Option<Direction>` picks the dominant axis of a movement vector (diagonal input has no corresponding sprite direction), returning `None` for zero movement so callers preserve the entity's last facing while idle rather than resetting to a default.
- **Rationale:** Matches how directional pixel-art sprites actually work — separate up/down/left/right frame sets, not continuous angles — so this is also the data shape a future animation system will want, not just a readability convenience now. `required_facing: &[Direction]` reads directly as intent (`&[Direction::Right]`) at every trigger's construction site, rather than a vector a reader has to mentally decode.
- **Consequences:** Facing is inherently coarse (four directions, no diagonals) — acceptable and expected for this game's visual style; would need revisiting only if 8-directional sprites were ever adopted, not currently planned.

### ADR-060: Multi-approach interactables use one Trigger per side, not one Trigger with a facing list
- **Context:** The necklace (later, once built) needs to be interactable from two opposite sides — approach from north facing south, or approach from south facing north — but not from east or west. An initial design put both acceptable directions in one Trigger's `required_facing` list, sharing one Rect for both approaches.
- **Decision:** A shared Rect with a multi-entry `required_facing` list is only correct when *every point in that Rect* has the same correct facing (true for the bed's single-approach case). For a genuinely two-sided object, two separate `Trigger`s are used instead — one per approach zone, each with its own single-direction `required_facing` — sharing one `prompt_entity`/`prompt_texture` so the visual icon is one object even though detection is two.
- **Rationale:** A single Rect with `&[Down, Up]` cannot distinguish "standing north of the object, facing south (correct)" from "standing south of it, facing south (incorrect, facing away)" — both satisfy `required_facing.contains(&player.facing)` if the box straddles both zones. Correct facing is a property of *where the player is relative to the object*, not a property of one shared box.
- **Consequences:** `Scene::update_interact_prompts` must OR proximity across every trigger sharing a `prompt_entity` (via a `HashMap<EntityId, bool>` keyed by the shared entity, not a per-trigger overwrite) — a naive last-checked-trigger-wins loop would incorrectly hide the icon depending on trigger iteration order. `required_facing` as a slice remains correct and useful for its actual case (one box, one-or-more equally-valid facings within that single box) — this ADR narrows when to reach for it, it doesn't deprecate it.

### ADR-061: Mouse-position debug readout shows authoring-space, not raw world-space
- **Context:** `Renderer::screen_to_world` correctly converts a screen pixel to world-space coordinates. Displayed directly, this produced a real false alarm: a trigger authored as `Vec2::new(94.0, 40.0) * multiplying_factor` in source read back as `(469, 200)` under the mouse readout — briefly suspected as a bug before recognizing that world-space coordinates have always included `multiplying_factor`'s scale (ADR-042); the raw literal `(94, 40)` was never a world-space value to begin with, only the pre-scale input to one.
- **Decision:** The F2 debug readout divides `world_pos` by `multiplying_factor` before display, so the number shown matches what a developer would actually type into a `Vec2::new(...)` literal at a scene's construction site.
- **Rationale:** The tool's purpose is to speed up hand-placing content — it's more useful reporting "what to type" than "the true final scaled value," since the former is the number actually compared against source code during authoring. `screen_to_world` itself is unchanged (still returns true world-space, needed elsewhere); the division is display-only, at the one call site that renders this specific debug text.
- **Consequences:** Recorded explicitly so the same false alarm doesn't recur — any raw literal in scene-construction code is pre-scale/authoring-space; any position read from a live `Entity`/`Rect`/mouse-readout is post-scale/world-space; the two are only numerically equal when `multiplying_factor == 1.0`.

---

## 5. Current State & Open Questions

### Where the project stands (August 2026, v4)

- ✅ Design phase, derivation pass, naming (Phases 0–9, unchanged).
- ✅ **M1 (The Toolchain and Window):** winit 0.30 integrated, window,
  input logging, delta timing, ControlFlow::Poll.
- ✅ **M2 (The Wall):** full wgpu pipeline — surface/device/queue,
  WGSL shader, textured quad, bind groups, transform uniform, 2D
  coordinate system (ADR-031), transform chain (ADR-032), entity
  position-as-center convention (ADR-033), held-state input, working
  camera-follow. Definition of done met in full.
- ✅ **M3 (The Room):** entity model, scene composition, texture
  store, AABB collision with sequential per-axis resolution, y-sort
  by baseline, full renderer rewrite (multi-entity draw, per-draw
  submission), multiplying_factor whole-composition scaling, and
  permanent debug tooling (collider overlay, input logging) — all
  built and proven against tuned, real bedroom content. Definition
  of done met in full.
- ⬜ **M4 (The Voice)** ← in progress. Design phase (Phase 12)
  resolved scene transitions, trigger/warp identity, and scene
  persistence before content work began: concrete `Scene` swap-slot,
  trait+stack deferred to M6 (ADR-044); automatic zone-triggered door
  transitions, no button prompt (ADR-045); two trigger flavors
  (interact vs. zone), single-variant dispatch enum (ADR-046);
  Pokémon-style warp pairs over per-scene spawn points (ADR-047);
  scene persistence via the existing flag store (ADR-020) with lazy
  GPU unload/reload (ADR-048); native Aseprite loading, parked at
  ADR-049, implemented mid-milestone once PNG-export friction became
  real (ADR-050); warp identity as a named-string WarpId with
  per-trigger reentry suppression and Scene self-identity (ADR-051);
  startup warp-pair validation proposed and deferred (ADR-052).
  `SceneId` variant rename (`Bedroom`/`Hallway` → `Home`/`Outside`)
  still pending in code. 
- Remaining per DERIVATION §5: door/trigger
  content in `new_home`, `new_outside` placeholder scene, the
  transition-handling code in `platform.rs`, text rendering, dialogue
  machinery (E6), audio (E7), examine-bed narrator text, typewriter +
  blips, inner-monologue frame. Scene-transition mechanism (trigger firing, warp-pair resolution,
    reentry suppression, spawn positioning) is now fully implemented and
    verified working end to end between Home and Outside — see ADR-053,
    054. Camera currently uses only the static-anchor mode (ADR-041);
    Outside needs a follow-camera, since its content already exceeds a
    single static screen — CameraMode (static vs. follow, chosen per
    scene) is the immediate next task. Scenes-cached-forever vs.
    ADR-048's lazy-unload-and-rebuild-from-flags remains an open,
    undecided architectural question — current behavior (Vec<Scene>,
    reused once visited) has not been reconciled with ADR-048 either
    way. Main menu / pause menu deferred per ADR-055, blocked on text
    rendering.
- Code written so far: entity/scene/asset layer (now with
  `Collider`/`Trigger` types and `engine/ids.rs` for shared
  identifiers), collision, y-sort, full renderer, debug tooling,
  native Aseprite texture loading alongside PNG — one scene
  (`new_home`, née `new_bedroom`) as content. No scene transitions,
  no second scene, no dialogue, text rendering, or audio exist yet.
- Text rendering foundation now built and verified (ADR-056–058):
  screen-space bitmap font pipeline, F3 debug toggle confirms correct
  glyph lookup/UV sampling/spacing end to end. Still not built:
  dialogue machinery (typewriter, registers, blip audio, advance/skip
  input), the dialogue panel/avatar frame, and Beat 2's actual
  narrator-text content.
- Interact triggers (facing-gated, proximity + button) now built and
  content-tested against a real beat (the bed's lore-drop examine
  text) — see Phase 14, ADR-059–061. Debug tooling extended: world-
  space mouse readout, center-position crosshair markers, trigger
  color-coding by kind. Both text rendering (~83 glyphs) and the
  expanded debug overlay now measurably tank frame rate (per-primitive
  buffer/bind-group/draw-call cost, ADR-043's originally-accepted cost
  crossed its stated revisit threshold) — batching is the identified
  fix for both, not yet built, tracked as an open item.

### Open questions

1. **Tuning knobs (playtest, not debate):** grid width, tick length,
   lockout duration, telegraph visuals — unchanged, still Phase 1+.
2. **Writing pending:** unchanged from v3.
3. **Narrator identity:** parked, unchanged.
4. **ECS / serde re-evaluation:** unchanged, Phase 1.
5. **Input abstraction (ADR-035):** deferred until a second
   input-consuming system exists — watch for the natural trigger at
   M4 (dialogue advance) or M5 (menu nav); don't build early.
6. **Engine name; open-source license:** deferred, unchanged.
7. **New:** current single-sprite scaffolding on `Renderer`
   (`sprite_position`, `camera_position`, `transform_buffer`) needs to
   migrate to an entity model in M3 — not a question of *whether*, but
   the concrete shape is M3's problem to solve, not to anticipate here.
8. **Draw-call batching (text and debug overlay):** both `render_text`
   (one draw call per glyph) and the debug-rect overlay (one draw call
   per rect, now tripled by center markers) pay real, now-measured
   per-primitive GPU cost — confirmed via frame-rate drops (~60fps to
   ~18fps for text at real dialogue-length strings; F1 currently
   unusable with center markers on). Fix identified for both: one
   shared vertex/index buffer, one draw call per whole batch, UV/color
   data baked per-vertex instead of via a per-primitive uniform. Not
   yet built — next concrete task once content/log work settles.

### Parked — Combat design: Z-axis attacks & essence economy

Sketched in a design session (pre-M4), not mechanically finalized. Not
an ADR — nothing here is settled.

- **Z-axis attacks bypass 2D combat rules.** Vertical/"downward" attacks
  ignore lane-blocking and telegraphs entirely — a 2D enemy has no
  sensory access to a Z-axis threat, so it cannot be telegraphed to
  them. Resolves against a column regardless of lane occupancy
  (multi-target), vs. normal 2D attacks which stop at the first body
  in a lane.
- **Player never becomes a "4D" being.** The tree may grant abilities
  *flavored* as reaching beyond 3D, but the player's own arc stays
  strictly 2D→3D. Cultists can structurally never ascend at all —
  their identity is the rejection of a dimension, which forecloses
  the move permanently (rhymes with ADR-006's reject/inherit
  juxtaposition).
- **Essence-absorption cutscene, structure sketched:** consensual
  prompt (E-press on a downed enemy, not automatic pickup) → narrator
  explains via conservation-of-energy framing → explicit in-fiction
  warning against over-accumulation → tech tree UI partially unlocks
  (early nodes only).
- **Open / unresolved:** concrete failure state for hoarding unspent
  z-essence (mechanic undefined — soft cap? damage over time? hard
  block?); dialogue lines unwritten; register assignment (narrator vs.
  inner monologue vs. plain shown text) undecided per-line.
- **Reserve idea, explicitly not scoped:** a tier beyond 3D as a
  genuine player state (as opposed to a tree-granted flavored
  ability) — tonally risky (pulls toward cosmic scale, away from the
  sibling story), parked alongside ADR-007's replacement-sacrifice
  reserve. Not designed, not built into any node list.
- **Likely narrative payoff (unconfirmed):** the climax may resolve
  by the player transferring accumulated z-essence *back* to the
  sister rather than keeping it — ties the resource sink to the
  ending and forecloses an endgame power-fantasy loophole by
  construction. Not committed; flagged as the leading candidate.

### Parked — Documentation: a screenshot-narrated project history

Raised when auditing the screenshot gap across Phases 12–13. `PROJECT_LOG.md`'s existing convention (name a screenshot in prose — e.g. "docs/screenshots/ now covers m3-01 through m3-06") serves the log's actual purpose well: a technical, ADR-anchored continuity record for resuming work across sessions. It gives a casual GitHub visitor no rendered images and no narrative thread connecting them, however.

A separate document — screenshots embedded inline, written for an outside reader rather than a resuming collaborator, telling the project's visual progression from blank window to current state — would serve a genuinely different audience (portfolio viewers, casual repo browsers) than PROJECT_LOG.md does. Not built now: mixing the two purposes would dilute both. Worth building whenever there's a real audience for it (nearing a shareable/portfolio moment), not speculatively now.

### Parked — Interact triggers: separate notice-radius vs. interact-radius

Raised while designing TriggerKind::Interact (M4, the necklace/nightstand case). A single Rect, tested via point_in_rect, currently governs both "show the interact prompt icon" (proximity only) and "the interaction may fire" (proximity + facing + button) — confirmed as mathematically sufficient for every case designed so far (point-in-rect and aabb-overlap were shown to be the same test, just parameterized by rect size — not a source of the "notice vs. touch" distinction on their own).

A genuinely different, currently unbuilt idea: two separately-sized rects per interactable — a wider one governing "the player can tell something's here" (icon visibility) and a tighter one governing "the player may actually interact" (e.g. a glowing altar visible from across a room, but only interactable up close). Not built now — no current beat needs it, and building it speculatively risks guessing radii against no real content. Revisit when a real case demands the distinction; if none ever does, this parked idea should simply be removed rather than built for its own sake.

### Next session agenda (Milestone Chat #3: The Room / M3)

1. Design the `Scene` trait and scene stack (E4) — M3's `Scene` is
   currently a concrete struct; M4 needs push/pop semantics (battle-
   over-overworld is the eventual target per Beat 6, but M4 itself
   only needs room-to-room transition, not a stack depth beyond one).
   Decide what "transition" means concretely: unload/load, spawn
   points, whether the old scene's state persists.
2. Text rendering (E2/E6 boundary) — nothing in the renderer draws
   text yet at all. This is likely the milestone's own "wgpu wall"
   moment: a new, real subsystem (font atlas or similar), budget
   accordingly rather than assume it's small.
3. Dialogue machinery (E6): typewriter reveal, advance/skip input
   (the natural second consumer for ADR-035's deferred Action/InputMap
   abstraction — watch for whether this milestone is where that
   finally gets built, or whether it's still prematurely early).
4. Three-register visual design (ADR-021): narrator (no avatar), inner
   monologue (avatar + distinct frame), NPC (avatar + standard frame)
   — M4's slice only needs inner monologue + narrator (NPC dialogue
   isn't until M5's village).
5. Audio (E7): first blip sounds, tied to typewriter reveal per
   character/register.
6. Content: Beat 2's actual scene (the empty house / bedroom
   transition), examine-triggered narrator text on the sister's bed —
   this is where E8's interaction/trigger system (proximity + facing +
   button, stubbed as a console log in M3) gets its first real
   payoff: console log becomes actual dialogue text.

---
