# Project Log — Tetherwood
### Game: *Tetherwood* · Act I / vertical slice: *"The Morning She Was Gone"* · Engine: unnamed (deliberate)

**Document type:** Living project record (docs-as-code)
**Started:** July 2026
**Revision:** v14
**Status:** M1–M4 complete, M5 (The Village) in progress through Phase 32. A design-only session (Phase 33) settled tile-based scene background authoring, superseding ADR-028's last standing clause (ADR-096) and carving out minimal file-based persistence for tile grids specifically from ADR-027 (ADR-097) — not yet implemented, ready to hand to an M5 implementation session. Companion document: `docs/DERIVATION.md`.
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

### Phase 15 — M4: Draw-Call Batching & Debug Tooling Fixes (completed)

The batching fix flagged as an open item at the end of Phase 14 was
built for both of its two identified targets. Text rendering
(previously one full GPU submission per glyph) now bakes each glyph's
final screen position and atlas UV directly into a shared vertex/index
buffer built once per string, reusing the ordinary sprite pipeline
outright — since baking away the per-glyph UV remap left nothing
text-specific for a dedicated shader to do, text_shader.wgsl and its
pipeline were deleted entirely, not just optimized — see ADR-062.
Debug rects received the same treatment, with one necessary
difference: fill color, border color, and border thickness vary per
rect in ways that can't be baked into shared geometry the way UV
sampling could, so this pass needed a genuinely new vertex format
(DebugVertex) carrying those as per-vertex attributes instead of a
per-draw uniform, and debug_shader.wgsl was rewritten (not deleted)
to read them that way — see ADR-063. Both fixes confirmed by direct
before/after measurement: text went from ~18fps to near-baseline at a
full dialogue-length test string; the debug overlay, which had become
unusable once center-position markers roughly tripled its per-frame
rect count, is legible again.

Two smaller fixes landed alongside this. The font atlas, migrated to
.aseprite mid-session, was briefly loaded via Texture::from_bytes
directly — the PNG/JPEG decoder, silently wrong for Aseprite's binary
format, and a runtime decode failure. Routed instead through
TextureStore's existing extension-dispatch (ADR-050), which required
a new TextureStore::take(id) to extract ownership from an otherwise-
throwaway, single-texture store — explicitly documented as unsound on
any longer-lived store, since removal invalidates every later
TextureId (ADR-036) — see ADR-064. Separately, the mouse-position
debug readout's first version displayed true world-space coordinates,
producing a real false alarm (a trigger authored as (94, 40) read back
as (469, 200)) before recognizing world-space has always included
multiplying_factor's scale; the readout now divides by
multiplying_factor before display, matching what a developer would
actually type in a scene's construction code — recorded earlier this
session as ADR-061.

Two more debug-usability fixes: an on-screen FPS counter (smoothed via
an exponential moving average, since raw per-frame delta was too
noisy to read at a live update rate) gives a real-time readout for
exactly this kind of before/after comparison going forward, and debug
text positions anchored to Renderer::screen_size() rather than
hardcoded coordinates, so they stay correctly placed after a window
resize or maximize rather than silently drifting.

Docs-as-code extended: docs/screenshots/ now covers m4-11 through
m4-13 — the mouse readout staying correctly anchored after a window
resize (m4-11), and a direct before/after pair proving the debug-rect
batching fix, unbatched and near-unusable (m4-12) against batched and
back to near-baseline framerate (m4-13).

### Phase 16 — M4: Dialogue Machinery (completed)

The typewriter/advance/skip/register system named as M4's remaining
core requirement was built in full, scoped to the two registers Beat
2 actually needs (Narrator, InnerMonologue — NPC dialogue stays M5's
job). DialogueState reveals a line character-by-character on a fixed
interval, correctly using char boundaries rather than byte slicing
(a real, if narrowly-avoided, panic risk the moment any line uses a
non-ASCII character like an em dash). Both E and Space advance/skip;
E alone may also start a new interaction when no dialogue is active,
so idly pressing Space while walking never accidentally triggers an
examine.

Dialogue content is authored as a Vec<DialogueLine>, each holding a
Vec<ColoredSpan> rather than a flat &str — chosen specifically to
support per-span coloring (e.g. a single word tinted to read as fear)
as structured Rust data rather than a parsed markup string, keeping
D-C's no-parser discipline intact even as content richness grew — see
ADR-065. Vertex (shared by every sprite and by batched text,
post-ADR-062) gained a tint field for this, multiplied into the
sampled color in shader.wgsl and defaulting to white everywhere else,
so ordinary sprites needed no change — see ADR-066.

A screen-space dialogue panel reuses the batched solid-rect pipeline
built for the F1 debug overlay (ADR-063) — its second real consumer,
generalized from DebugRect/DebugVertex to SolidRect/SolidVertex and
from a debug-only draw path to Renderer::render_solid_rects, callable
with any projection/view pair rather than assuming render_scene's
world-space transform — see ADR-067. The panel's border color reads
the active line's Register, the first real use of Register beyond
bookkeeping and a cheap stand-in for ADR-021's fuller "distinct
frame" requirement until real panel art exists.

A blinking "press to continue" caret (▼, drawn into a previously
empty atlas grid cell) is shown only once the current line is fully
revealed, toggled via its own independent timer so it keeps blinking
continuously rather than resetting per line — implemented as one more
character through the existing batched text path, no new sprite or
draw call.

Docs-as-code extended: docs/screenshots/ now covers m4-14 through
m4-16 and m4-20 — typewriter reveal mid-line (m4-14), the panel
border and per-span text color both visibly distinguishing register
(m4-15), the panel after visual tuning (m4-16), and the blinking
caret (m4-20).

### Phase 17 — M4: Debug Tooling Independence & Facing Visualization (completed)

Dialogue's text scale (DIALOGUE_TEXT_SCALE) and debug text's scale
(DEBUG_TEXT_SCALE) were decoupled — both had been forced to share one
constant, so tuning either affected the other. layout_text/
layout_colored_text became thin wrappers defaulting to
DIALOGUE_TEXT_SCALE around new _scaled variants taking an explicit
scale, the same default-via-wrapper pattern already used for color.
Debug text (FPS counter, mouse position) gained solid background
boxes via a new Renderer::render_text_bg, sized to the glyph list's
own bounding box (text::combined_glyph_info) plus an explicit padding
parameter — reusing render_solid_rects rather than introducing new
drawing machinery, the same generalized pipeline the dialogue panel
already uses.

Entity facing gained two independent visual expressions. A horizontal
flip (negative x-scale in the sprite draw, gated on Direction::Left)
gives the player sprite genuine left/right distinction — Up/Down
remain visually identical, a known, accepted limit of flip-only
direction until real directional art exists (frame-based, per
ADR-050's proven single-frame-swap pattern, extended). Separately, a
new F1 debug marker (push_facing_marker) draws a short line from each
textured entity's position in its current facing direction — its
first version, a plain symmetric bar, was found genuinely ambiguous
(Up and Down were visually indistinguishable, confirmed by placing
two oppositely-facing beds side by side) and was replaced with a
three-segment tapering shape reading unambiguously as a directional
arrow, using nothing but the existing batched-rect pipeline — see
ADR-068.

Docs-as-code extended: docs/screenshots/ now covers m4-17 through
m4-19 — scaled, backed debug text (m4-17), the player sprite's
left/right mirroring (m4-18), and the corrected, unambiguous facing
debug lines (m4-19).

### Phase 18 — M4: Flush Collision, Trigger Restructuring, Necklace Pickup (completed)

Player movement previously stopped short of a blocked collider by
whatever distance the frame's step size happened to leave, rather
than landing flush — visibly worse at higher speed or lower frame
rate, and never actually zero-gap. collider_blocked now returns the
specific obstacle (Option<Rect>, not just whether one exists), and a
blocked step resolves to the exact geometric contact position,
computed fresh from geometry every frame rather than from a
delta-dependent gap — stable regardless of movement speed. Getting
there surfaced and fixed two further bugs: aabb_overlap's strict `<`
judged exact flush contact (newly, frequently reached) as "not
overlapping," letting movement pass clean through the axis that was
supposedly blocking it; and an intermediate gap formula mixed current
and proposed position inconsistently, producing jitter that never
converged — see ADR-069.

TriggerKind's Interact variant split into Dialogue and Toggle — a
direct, dialogue-free texture/collider flip (state inferred from
which texture the entity currently has, ADR-037's existing
Option-as-state pattern) for objects needing to change on every press
with no conversation, distinct from Dialogue's proximity+facing+
button-into-a-conversation flow. Dialogue's prompt_entity/
prompt_texture became Option — not every interactable should show a
"press E" icon forever; this game's design uses prompts only for the
first couple of tutorial interactions. TextureStore::load_aseprite_frame
loads a specific frame of a multi-frame file explicitly, chosen over
Aseprite layers since frames already produce the flat, composited
image a static state needs — see ADR-070.

A new patio door (new_outside) exercises Toggle against real content,
open/closed via two frames of one file. Its first version listed both
Up and Down in one shared trigger's required_facing and could be
opened from the wrong side (standing above, facing away, still
satisfied the check) — the same ADR-060 gap first found on the
necklace, now hit a second time on content this same phase
introduced; fixed by splitting into two triggers, one per approach
zone, sharing all door state and differing only in rect and
required_facing.

DialogueLine gained a narrow, single-purpose consumes_entity: Option
<EntityId> — not a general post-dialogue callback — clearing a
specific entity once a specific line is the dialogue's last and it
closes. Trigger gained a permanent active flag, distinct from the
already-transient recently_used: once consumed, a trigger is
deactivated for the scene's remaining lifetime, checked everywhere
triggers are read (check_triggers, try_interact,
update_interact_prompts, and the F1 debug overlay — all four
independently iterate triggers and all four needed the guard).
Scene::consume_entity clears the target entity, deactivates its
trigger, and separately clears the trigger's own prompt icon — a
distinct entity, easy to miss since a deactivated trigger no longer
participates in prompt-visibility logic at all and would otherwise
leave a stale icon frozen on screen — see ADR-071. The necklace now
removes itself (sprite, collider, prompt, trigger) after its examine
line finishes, Beat 2's first working item-pickup moment.

A Ctrl+R scene-reset shortcut (AppState::reset_scene, reusing
change_scene's build_scene path) rebuilds the current scene from
scratch for testing, checked via the existing held_keys set rather
than winit's separate ModifiersState API.

Docs-as-code extended: docs/screenshots/ now covers m4-21 through
m4-25 — flush, zero-gap contact against a collider (m4-21), the
patio door's two Toggle triggers visible under F1 (m4-22), the door
after opening (m4-23), and a before/after pair confirming the
necklace's full removal after being picked up (m4-24, m4-25).

### Phase 19 — M4: Blip Audio (completed)

The last unbuilt piece of M4's core machinery — blip sounds tied to
typewriter reveal, per DERIVATION's original E7 scope — is in. kira
(0.12) was chosen over rodio after a real comparison: kira is
purpose-built for game audio (tick-synced playback, per-instance
pitch/volume control) where rodio is a general-purpose playback
library that would need that layer hand-built on top — see ADR-072.

DialogueState::tick() reports whether a blip-worthy (non-space)
character was newly revealed, rather than DialogueState knowing
anything about audio itself — the same machinery/content-adjacent
separation Scene already keeps from rendering. Pitch cycles through a
small fixed sequence rather than randomizing, a deliberate rise-and-
fall pattern rather than chatter. Volume is stored and applied via
Decibels, not a linear factor, matching how human loudness perception
is logarithmic — already shaped for a future settings-screen slider
with no rework needed when that's built.

Docs-as-code note: no new screenshots this phase — audio has no
visual signature to capture; confirmed instead by direct listening
(pitch-cycling audible via a temporarily exaggerated test range,
narrator/monologue blips audibly distinct, volume adjustable).

### Phase 20 — M4: Toast Notifications, Wall-Slide Fix, Hand-Built Slider (completed)

Self-expiring, stacked toast notifications (Notification: message,
duration, start_time) replaced the F3 test-string debug toggle,
Ctrl+R scene reset becoming the first real consumer ("Scene (id)
reset", 2 seconds). render_text_with_bg consolidated a
background-then-text pairing every debug text block had been
repeating by hand. Stacking was built newest-shifts-existing-upward
first, then deliberately reverted to positionally-stable
(each notification keeps its on-screen position for its whole
lifetime) after testing showed a shifting stack was harder to track
by eye — a real, tested UX call, not a default.

The pre-existing "sliding along a wall decreases speed" TODO was
fixed: diagonal movement splits speed across both axes by design, but
the free axis was still using its diagonal-reduced share even after
the other axis became fully blocked, rather than being topped up to
the full per-frame speed a purely-cardinal move would get. Two cheap
probes against the unmoved starting position determine each axis's
blocked status before either axis's real magnitude is decided, so a
blocked axis can hand its unused speed budget to the other — fully
symmetric (both x-blocks-boosts-y and y-blocks-boosts-x), confirmed
against cardinal movement, diagonal wall-sliding on both wall
orientations, and a true diagonal corner.

A hand-built Slider (engine/ui.rs) replaced considering an egui
dependency for a single volume knob — evaluated deliberately rather
than dismissed by default, given the developer's own stated future
interest in an inspector panel. update() returns bool and the caller
reads .value directly, rejecting a registered on_change closure after
checking egui's own actual Slider API, which uses the identical
mutate-in-place-and-check-a-flag shape, not a callback — the
project's already-per-frame redraw loop is structurally immediate-
mode already, and a closure-based callback belongs to a different,
retained-mode paradigm the project isn't using — see ADR-073. Volume
is the first real, live-adjustable control wired to blip_volume's
already-Decibels-shaped value. Renderer::screen_projection()
consolidated four separate hand-built orthographic projection
constructions (render_scene, render_dialogue_panel, render_text,
render_text_bg) discovered while wiring the slider's own draw call —
an attempt to instead expose Renderer's SurfaceConfiguration wholesale
was tried and reverted in favor of this narrower getter.

Grid-based spatial partitioning for collision checks — informed by a
technique from a colleague's project, dividing a scene into cells and
quadrant-based neighborhoods so collider_blocked only checks nearby
obstacles rather than every collider in the scene — evaluated and
explicitly parked rather than built, since no current scene is large
enough to make today's linear-scan cost measurable; revisit around
M5's populated village.

Docs-as-code extended: docs/screenshots/ now covers m4-26 — the
volume slider live-adjusting blip_volume.

### Phase 21 — M4: Word-Wrap and Beat 2's Real Dialogue (completed) — M4 CLOSED

The last piece of unbuilt dialogue machinery landed alongside the
milestone's actual content, surfaced by that same content: real prose
(45-70 character lines) exposed that layout_colored_text_scaled had
never had any concept of line width, since every prior string
(placeholder lines, debug text, toasts) happened to be short enough
to never need it. text::wrap_colored_text splits a colored-char
sequence into multiple visual lines at word boundaries only,
processing one word at a time so a break can never land mid-word —
kept as a separate pre-processing step rather than built into
layout_colored_text_scaled itself, so every other caller of that
function needed no changes and no risk of altered behavior.
Renderer::dialogue_text_max_width() derives the panel's usable width
from the same constants dialogue_text_position() already uses, rather
than a separately guessed value.

Beat 2's actual dialogue replaced every placeholder line: the bed (5
lines — rumpled sheets and the storm giving way to real worry) and
the necklace (4 lines, ending on its one deliberate felt-wrongness
beat, "heavier than it should" — every other line in both sequences
stays mundane on purpose, since Beat 2 shouldn't tip its hand yet).
[Name] remains an explicit, unresolved placeholder pending a naming
decision, not an oversight.

This closes M4 (The Voice). DERIVATION's definition of done — Beat 2
playable: room transition; examine bed/necklace → narrator and inner-
monologue text; typewriter + blips; inner-monologue register visually
and aurally distinct — is met in full, verified against real content,
not placeholder text. M4 substantially exceeded DERIVATION's original
E4/E6/E7 scope along the way: the scene-transition/warp system grew
three trigger kinds (Warp, Dialogue, Toggle) rather than one; debug
tooling grew from a collider overlay into a real, if informal,
authoring aid (world-space mouse readout, facing visualization, a
live-adjustable slider); text rendering and the debug-rect overlay
both required a full batching pass once real content and richer
debug visuals exceeded their originally-accepted per-primitive draw
cost.

Docs-as-code extended: docs/screenshots/ now covers m4-27 — a real
Beat 2 line correctly word-wrapped within the dialogue panel.

### Phase 22 — Sister's Departure Re-Lore & Ancestral Protector Lineage (completed)

A design-only session (no code), triggered by re-examining the necklace's
placement (behind the bed, decided during M4 implementation) against
ADR-019's original claim that she never returned home. The two didn't
fit together cleanly, and pulling on that thread produced three real
decisions, recorded below as ADR-074–076.

Resolved: she came home, was woken mid-storm, and left again in a
hurry in the dark — not taken from the room, not a struggle. The
necklace catches on the headboard as an ordinary consequence of a
rushed, lightless departure, not violence. This supersedes ADR-019's
"never came home" claim while keeping everything ADR-019 was actually
for (real panic, natural gate-block routing) intact.

Also resolved: the gate sabotage was never just atmospheric cult
cover — it's premeditated ambush-routing, timed to a path the cult
had already learned she takes. This sharpens "why sabotage the gate
at all" into a concrete answer.

Also resolved, and the session's centerpiece: the sigil has a real
origin. An ancestral line of village protectors, passed to the eldest
child, has maintained protective wards against this specific cult for
generations — explaining, for the first time on causal grounds rather
than "conservation of energy" alone, why the totem can absorb and
convert z-essence (it was built for exactly that), why the sibling
tether functions as more than sentiment (it's a shared bloodline
conduit), and why she has the object at all. Retroactively upgrades
ADR-006's thesis and ADR-017's tree from "z-essence, because physics"
to "z-essence, because this family's entire purpose."

Also resolved: the clue-chain activity previously read as "she was
mixed up with the cult" is reframed as "she was actively working
against them" (ward maintenance, timed to the storm because rain
threatens chalk sigils). This is a stronger version of ADR-005's
premise, not a contradiction of it — she was always meant to be
secretly better than she appeared; now there's a concrete, dangerous
thing she did to make that true, which also strengthens ADR-007's
ritual precondition (unregretted, unknown sacrifice).

No code or shipped dialogue changes as a result of this session — the
Phase 21 bed/necklace lines were already tonally consistent with this
lore by coincidence of good instinct, and remain the canonical text.
This phase exists to make the ground under them solid, and to record
a Beat 4 writing constraint for whenever that dialogue is drafted.

Also parked (not decided, not an ADR): a hierarchy-gated escalation
for cult combat abilities — elemental/environmental battle effects
(wind, rain, sandstorm) usable only by named cult leadership with
preparation time, never rank-and-file initiates, mirroring how the
game already reveals cult capability gradually (ADR-016). Recorded in
full under Section 5's parked ideas.

### Phase 23 — Structural Refactor & Reorganization (completed)

A dedicated session, flagged at M4's close (Phase 21) and explicitly
deferred until now, addressing accumulated structural debt before M5
begins: `platform.rs`'s CPU/GPU-mixed renderer, a single giant
`RedrawRequested` handler, and scattered debug/editor tooling, plus
several items that surfaced mid-session.

**Renderer split into CPU/GPU/draw submodules.** `renderer.rs`'s
single file was split into `renderer/mesh.rs` (pure CPU vertex-
building: `Vertex`, `SolidVertex`, mesh construction), `renderer/
gpu.rs` (`Frame`/`Renderer` struct definitions and GPU lifecycle:
`new`, `acquire_frame`, `present_frame`, `resize`, coordinate-space
queries), and `renderer/draw.rs` (the `impl Renderer` draw-call
methods). `renderer.rs` itself became a thin facade — `mod`
declarations plus `pub use gpu::{Frame, Renderer}; pub use
mesh::SolidRect;` — see ADR-077.

**Debug tooling consolidated** under `engine/debug/` — F1/F3 overlay
code and toast notifications first, the volume slider and HUD drawing
folded in later in the same session — rather than staying scattered
across `renderer.rs`/`platform.rs` as each was added ad hoc.

**Scene-building content extracted from Scene itself.**
`Scene::new_home`/`Scene::new_outside` — genuinely game content (which
furniture, which warps, which dialogue triggers), not engine machinery
— moved to `game/scenes/home.rs`/`outside.rs` as `pub fn build(...)`,
leaving `Scene` holding only mechanics any scene needs — the
machinery/content test (§2.2) applied to the engine's own internals,
not just game features — see ADR-078.

**ADR-035's input abstraction resolved.** `InputState` (engine) holds
raw held-key state behind `press`/`release`/`is_held`; `Action`
(`game/actions.rs`) is the semantic, press-triggered enum a game maps
raw input onto. Movement stays a direct `InputState` read
(`resolve_movement`), deliberately excluded from `Action`, since it's
continuous held-state, not a discrete press — see ADR-079.

**`RedrawRequested` broken up.** The single event-loop match arm
handling dialogue ticking, player movement, and HUD drawing split into
`tick_dialogue`/`update_player`/`draw_hud` methods on `AppState`;
`DialogueState` itself moved out to its own file (`platform/
dialogue.rs`, later `app/dialogue.rs`).

**`engine/ids.rs` eliminated.** The file grouping `EntityId`/
`TextureId`/`WarpId`/`SceneId` together — built specifically to break
a circular dependency between `entity.rs` and `scene.rs` — was judged,
on reflection, not idiomatic Rust (types grouped by kind rather than
by the struct they belong to). Each type moved next to its owner;
`Trigger`/`TriggerKind`/`Background` moved from `entity.rs` to
`scene.rs` in the same pass, following the same ownership test —
`entity.rs` now imports nothing from `scene.rs` at all, a real fix
rather than a relocated version of the same coupling — see ADR-080.

**Files reorganized by owning/consuming module**, a second flat-
`engine/`-directory problem raised independently of the above: shaders
moved into `renderer/shaders/`, `text.rs`/`texture.rs` into
`renderer/`, `input.rs` into `app/` (formerly `platform/`; `platform.rs`
itself renamed to `app.rs` to match its actual contents and read
better at the call site), and `ui.rs` into `debug/` once its only
current use — a `show_debug_info`-gated volume slider — showed it
wasn't the general-purpose widget its name implied — see ADR-081.

**`Scene`'s `impl` block split out**, mirroring `draw.rs` holding
`Renderer`'s implementation separately from `renderer.rs`, once
`scene.rs` reached the same rough size (~470 lines) that had motivated
the renderer split — see ADR-082.

**Debug HUD drawing extracted** from `AppState::draw_hud` into
`engine/debug/hud.rs`, each function taking only the specific data it
draws rather than `AppState` itself, mirroring `debug::overlay::
build_debug_rects` already taking just `&Scene` — see ADR-083.

**Crate-wide import convention settled**, generalizing what was
already the de facto pattern in `renderer/`: `super::` imports first,
then `crate::` imports, then external crates and same-module relative
imports together, all alphabetized, with a blank line only between
`mod` declarations and the `use` block.

Every change in this phase was verified via `cargo check`/`cargo run`
and committed independently at each step. No gameplay behavior changed
at any point — this phase is purely structural.

### Phase 24 — M5 Opening: Spatial Partitioning & Debug Grid Visualization (completed)

M5's first concrete work, reframed mid-session from "pre-M5
foundational work" to M5 itself — a spatial index is infrastructure
the village genuinely needs, not a prerequisite bolted on before it.

Collision broad-phase moved from a linear scan over every wall and
entity to a sparse spatial grid. `SpatialGrid` (`engine/grid.rs`)
buckets `CollisionHandle`s (`Wall(WallId)` or `Entity(EntityId)`, both
Copy newtypes per ADR-025/036's indices-over-references pattern) into
a `HashMap<(i32, i32), Vec<CollisionHandle>>` keyed by cell
coordinate — sparse, so only cells something actually occupies cost
anything. A wall or entity collider wider than one cell is filed
under every cell its bounding box touches (`cells_for_rect`), so a
query from any of those cells finds it. `Scene::build_static_grid`
populates this once per scene load (wired into `AppState::build_scene`,
the single chokepoint both scene changes and Ctrl+R resets already
funnel through) from every wall and every non-player entity's
resolved world-space collider (`Entity::world_collider`, built on the
existing `collider_center` helper). The player is deliberately
excluded — it moves every frame, and a cell entry only stays correct
for content that doesn't; a dynamic grid for movers is a named,
deliberate follow-up, not yet built.

`collider_blocked` now queries `collision_handles_around_position`
(a 3x3-cell neighborhood around the check point, radius tunable)
instead of scanning the whole scene, resolves each candidate handle
back to a real `Rect`, and runs the same `aabb_overlap` test as
before — collision math itself is unchanged, only the candidate set
feeding it shrank.

A real naming pass happened over the course of building this, worth
recording since the end result is the vocabulary the rest of M5 will
build on: `cell_at_position` (world position -> cell, the grid's most
basic operation), `neighboring_cells` (pure geometry — every cell
coordinate within a radius, occupied or not, no lookup), and
`collision_handles_around_cell`/`collision_handles_around_position`
(the collision-specific narrowing built on top of `neighboring_cells`,
returning only cells' actual contents). The split between
`neighboring_cells` (geometry) and the `collision_handles_*` family
(contents) exists specifically because debug visualization and
collision queries turned out to want different things from "adjacent
cells" — debug wants to highlight a neighborhood regardless of
occupancy (pink boxes over empty space are fine), collision only
cares what's actually there. Building this as one function first,
then splitting it once the two real consumers' needs diverged, is
the same "generalize once a second consumer exists, not before"
pattern the project has followed since ADR-025 — just applied at
function-naming scale, not architecture scale.

Debug visualization built on top, gated behind new toggles (see
below): `build_occupied_cells_mesh` (every cell with a stored handle,
faint green) and `build_grid_lines_mesh` (visible-viewport grid lines,
computed from camera bounds each frame rather than the whole grid, to
stay cheap regardless of scene size) both read `Scene.static_grid`
directly with zero new GPU pipeline work — both reuse the existing
batched `SolidRect`/`render_solid_rects` path (ADR-063/067) exactly as
built. `build_player_neighborhood_mesh` highlights the player's
current cell (lighter fill) plus its radius-1 neighbors (stronger
fill, pink), keyed off `collider_center()` rather than raw `position`
— a real, felt bug during development: keying off sprite-center
position caused the highlight to visibly desync from the player's
actual physical footprint, since the sprite's visual center and its
collider center are not the same point on this project's art
(ADR-033 vs. the collider-center convention already used by
`check_triggers`/`update_interact_prompts`). Confirms `collider_center()`,
not `position`, is the right default for anything reasoning about
"where is this entity, physically."

A real, minor artifact was noticed and understood, not fixed: walls
whose edges land at or near a cell boundary can round up into an
extra occupied cell under the debug highlight (floor-based boundary
math, the same category of hairline issue ADR-069 already hit for
collision resolution) — cosmetic only, doesn't affect actual
`aabb_overlap` correctness, not worth chasing further right now.

Debug toggles were consolidated from two loose `AppState` bools into
a `DebugFlags` struct (`engine/debug.rs`) holding six flags, each with
a `toggle_*` method that flips its bool and returns a status string —
feeding directly into the existing notification system (Phase 20)
via a new `AppState::notify(impl Into<String>)` helper, replacing six
near-identical `Notification` literals (and the F1/F3 console-log-only
feedback that existed before). Key bindings deliberately remapped:
F1 (debug info/console logging), F2 (collider/trigger overlay), F3
(the debug screen's master switch — framed as an inspector-in-progress,
not just "the volume slider"), F4 (grid lines), F5 (player
neighborhood), F6 (occupied cells). Grid lines are gated on both F3
and F4 (`show_debug_renderer && show_grid`) by deliberate design —
collider overlay, FPS counter, and mouse-position readout are not yet
nested under the same master switch, a named, tracked follow-up, not
an oversight.

Committed as three separate, scoped commits: the grid + collision
query itself, the debug visualization built on top, and the
`DebugFlags` restructure — kept apart specifically so the
gameplay-affecting change (collision behavior) has a clean, isolated
diff from the two debug-only changes layered after it.

Docs-as-code extended: docs/screenshots/ now covers m5-01 through
m5-04 — the spatial grid's lines against the neutral background
(m5-01), occupied-cell highlighting layered on top (m5-02), the
player-neighborhood highlight (m5-03), and the F-key toggle ->
notification pipeline confirmed end to end (m5-04).

### Phase 25 — Dynamic Grid for the Player (completed)

Closes out the spatial-partitioning arc opened in Phase 24. `Scene`
gained a second `SpatialGrid` (`dynamic_grid`), rebuilt from scratch
at the start of every `try_move_player` call from the player's
current collider — a full rebuild rather than incremental
cell-boundary tracking, since a single mover makes that complexity
unearned right now. `collider_blocked` queries both grids and unions
the results before running `aabb_overlap`.

With exactly one mover (the player, which already excludes itself via
`skip_index`), this is architecturally inert today — no observable
behavior change. Its value is establishing the "snapshot every
mover's position before anyone moves this frame, query both grids
uniformly" pattern correctly now, before a second mover (an NPC)
exists to actually depend on it being right. `STATIC_CELL_SIZE`
renamed to `CELL_SIZE`, now shared by both grids.

### Phase 26 — Isometric Projection: Design & Rendering Foundation (completed)

A dedicated pre-M5 planning session (separate chat) resolved the
isometric-projection question left ambiguous since ADR-028, producing
a clear directive: isometric-ness is render-time-only. All game
logic — entity positions, wall/collider Rects, trigger zones, spawn
points, camera targets — stays in plain orthogonal 2D world space
exactly as M1–M4 built it. The isometric look is produced entirely by
a projection transform (shear + scale) applied at draw time. AABB
collision (ADR-038) is retained unchanged; OBB was evaluated and
rejected as unneeded for any currently-designed content. This
supersedes ADR-028's "isometric is art direction only, straight
orthographic renderer" framing while keeping the rest of ADR-028
(no tilemap engine, hand-authored scenes) intact.

`Renderer::isometric_projection()` implements the shear as a 2x2
matrix (`screenX = (x-y)*K`, `screenY = (x+y)*K*0.5`), with `K` a
placeholder (1.0) deliberately tuned by eye rather than derived —
same "tune by feel" approach as `CELL_SIZE`. An F10 toggle
(`is_isometric` on `AppState`, threaded through `render_scene`) makes
flat/isometric a live, comparable debug view.

A real, corrected design split emerged mid-implementation, after an
initial version (shear folded into `camera_view`, applied to every
sprite's full quad) visibly skewed the player sprite's shape rather
than just repositioning it. The fix, and the actual standing
principle going forward: sprites/background are point-sheared only —
`sprite_camera_view` stays translation-only, applied after shearing
`camera_position` and each entity's position individually, since
isometric sprite art is expected to already look correct from that
angle and should never have its quad reshaped. Debug geometry (grid
lines, occupied cells, collider/trigger overlay) gets the opposite
treatment — the full shear is baked into a separate `debug_view`
matrix, deforming the whole rect, since a flat world-space square
genuinely should render as a diamond from this angle. No changes were
needed to any debug-rect-building code — only which view matrix
`render_solid_rects` receives.

Docs-as-code extended: docs/screenshots/ now covers m5-05 through
m5-07 — the scene in normal/orthographic projection (m5-05), the same
scene isometric (m5-06), and a confirming shot that sprites stay
unskewed while debug collider geometry correctly renders as diamonds
(m5-07).

### Phase 27 — Isometric Movement, Per-Scene Camera Modes, Tunable Debug Grid (completed)

Three related pieces of follow-up work once the isometric projection
itself was proven, plus one real false start worth recording in full
since it produced a genuine, re-derivable lesson.

**Movement — two wrong approaches before the right one.** The first
attempt derived the true mathematical inverse of the projection's
shear submatrix (`to_isometric_direction`) and used it to remap raw
WASD input, on the theory that pressing "up" should always look
straight up on screen. Two real bugs surfaced from this: normalizing
*after* the remap (in `update_player`) discarded the per-axis
weighting the remap depended on, making up/down feel slower than
left/right; and `try_move_player`'s existing diagonal wall-slide
boost, tuned for flat mode's genuine two-key case, double-applied
weighting the remap had already done, causing wall-hugging players to
visibly speed up. Both were fixed (normalize before remap; an
isometric-specific fixed slide constant, `2/sqrt(5)`, derived from the
projection's fixed 1:0.5 axis ratio) — but a third, deeper issue
remained: diagonal movement (two keys held) still drifted at a plain
45 degrees rather than the isometric grid's true (steeper) diagonal
angle, causing a player sliding along a wall to slowly drift into it.
Debug-rect visualization of the player's facing direction, cross-
referenced against manual play of MegaMan Battle Network 6 for a real
isometric-movement reference, resolved the actual design question:
MMBN's scheme is not "always look screen-cardinal," it's the reverse
for one axis — a single key press looks screen-cardinal and moves
along a world-space 45-degree diagonal; two keys together look
grid-diagonal on screen (matching the isometric tile edges) and move
along a single world axis. Verified algebraically that this scheme is
*not* derivable as one linear formula (screen-cardinal singles and
grid-diagonal doubles are mutually exclusive under any
sum-then-transform approach, since the shear is linear) — it is a
deliberate control-scheme/art choice, matching the shape needed for
planned diagonal player sprites, and implemented as
`resolve_isometric_movement`: a direct 8-entry match on the four WASD
keys, each returning a pre-derived unit vector. Because every table
entry is unit-length, the earlier isometric-specific slide-boost
branch, `ISO_SLIDE_FACTOR`, and the `step`/`is_isometric` parameters
threaded through `try_move_player` all proved unnecessary and were
removed — the flat-mode boost logic already handles any unit-vector
direction correctly, isometric included. `to_isometric_direction` and
the formula-inverse approach were deleted outright, not left dangling.
Facing (`Direction::from_movement`) does not yet account for the new
table's diagonal/cardinal split — left as a tracked TODO, deferred
until facing while isometric is a real, felt need (expected once a
second mover, an NPC, exists).

**Per-scene, independently-authored camera modes.** A diamond-shaped
room no longer fits a fixed viewport the way a square room did, so
`CameraMode::Static` needed reconsidering for isometric scenes. After
two rejected designs — mutating a scene's active camera mode on F10
toggle (broke on scene transitions, since a freshly-rebuilt scene had
no way to know the current `is_isometric` state) and unconditionally
forcing `Follow` whenever isometric (wrongly assumed every scene wants
the same isometric behavior) — the shape that stuck: `Scene` stores
two authored `CameraMode`s (`orthographic_camera_mode`,
`isometric_camera_mode`), resolved to one active `current_camera_mode`
via `Scene::resolve_camera_mode`, called identically from `Scene::new`
and from `sync_camera_mode` (the F10 handler), so scene load and the
toggle can never disagree. Each scene authors both independently —
Home stays Static in orthographic / Follow in isometric; Outside stays
Follow in both. `CameraMode` gained `Default` (`Follow`) for scenes
that don't need to distinguish the two.

**Tunable debug grid display size.** Numpad8/Numpad2 increase/decrease
a new `grid_display_cell_size` field in 8px steps (clamped [8, 128]),
each firing a notification. `build_grid_lines_mesh` takes this as a
parameter instead of reading the real `SpatialGrid`'s `cell_size`,
letting the debug overlay be visually resized independently of actual
collision geometry; `build_occupied_cells_mesh`/
`build_player_neighborhood_mesh` are deliberately untouched, since
those show real grid contents, not a reference grid. `DebugFlags`
renamed to `DebugSettings`, since it now holds a tunable value, not
just booleans, and "flags" stopped accurately describing its
contents.

Docs-as-code extended: docs/screenshots/ now covers m5-08 through
m5-09 — increased and decreased debug grid display size. The
movement-table correction and per-scene camera mode work in this
phase have no distinct visual signature beyond what m5-05–m5-09
already show; no additional screenshots taken.

### Phase 28 — Progression Tracking & Trigger-Owned Dialogue Outcomes (completed)

`ProgressionTracker` (`game/progression.rs` — deliberately placed in
`game/`, not `engine/`, since nothing engine-level ever reads or
writes it, only content code does, mirroring `game/dialogue.rs`'s
existing precedent for this kind of `engine`-imports-from-`game`
crossing) is a minimal, in-memory `HashMap<&'static str, bool>` living
on `AppState` for the session's runtime, surviving scene transitions
per ADR-048. Deliberately no persistence to disk — nothing in M5's
actual need requires it, and ADR-027 already defers serde/data-files
until real content volume forces it; an inspector's eventual
save-to-file need was explicitly named as a future justification, not
a present one.

`Entity` gained `active: bool` and a `deactivate()` method
consolidating what `consume_entity` previously did by hand (clear
texture, clear collider) into one call — a real cohesion improvement,
not just a rename, since it replaces four independently-tracked
`Option`-clearing sites with one.

`TriggerKind::Dialogue` gained `sets_flag: Option<&'static str>`
alongside its existing `consumes_entity` — both live on the trigger,
not per-`DialogueLine`, since the trigger is dialogue's actual entry
and exit point; a multi-line conversation would otherwise need the
same value padded onto every line just to reach the last one, exactly
the padding problem ADR-071 already reasoned about once for
`consumes_entity` alone. `InteractResult::Dialogue` and
`DialogueState::new` both thread the value through, replacing the
earlier construct-then-mutate pattern
(`lines.last_mut().consumes_entity = ...`) with a single, fully-formed
construction call — `line_for`'s output stays pure, untouched authored
content.

Scene construction (`home::build`) checks `progression.is_set(...)`
once per relevant entity and calls `deactivate()` up front for
anything already consumed in an earlier visit — the necklace is the
first real content built against this, confirmed end to end (picked
up, walked out, walked back in, stayed gone). `outside::build` takes
`&ProgressionTracker` too, for signature consistency, currently
unused.

Flag-conditioned dialogue *content* (different lines depending on
which flags are already set, not just whether a flag gets set) is
explicitly out of scope for this phase — `line_for` still takes only
an `id`, no `ProgressionTracker` visibility. Named as the next real
step once actual NPC dialogue authoring begins.

Docs-as-code note: no new screenshots this phase — ProgressionTracker
and the trigger-owned dialogue outcomes are confirmed by behavior
(picked up the necklace, walked out, walked back in, stayed gone),
not by anything with a visual signature distinct from Phase 21's
existing necklace-removal screenshots.

### Phase 29 — Village Content Foundation: Village Rename, Fixed cell size and Flag-Aware Dialogue (completed)

CELL_SIZE (grid.rs) was discovered to be authored directly in
world-space units (64.0) with no multiplying_factor applied to
itself — unlike every other authored value in the codebase, which is
written in small "authoring units" and scaled once at construction.
Its real authoring-unit size (64 / 5 = 12.8) never matched what the
number implied, surfaced while taking pixel measurements off the
debug grid for a new sprite. Fixed: CELL_SIZE is now 12.0, in the
same convention as everything else, multiplied by multiplying_factor
once at each SpatialGrid construction site (Scene::new,
build_static_grid, rebuild_dynamic_grid), all three now taking
multiplying_factor as a parameter; try_move_player and render_scene
thread it through for the same reason. grid_display_cell_size's
tunable clamp range was rescaled to match. multiplying_factor itself
stays a plain runtime f32, not a const — a real, named future case
(inspector-editable content) was identified, but the inspector
doesn't exist yet to consume it; premature to build a config struct
around a guess.

outside.rs/SceneId::Outside/the outside asset were renamed to
village.rs/SceneId::Village — a pure rename, no logic change, done
ahead of the first real NPC so new content wouldn't keep
accumulating inside a name still describing the old placeholder
scene.

line_for gained a &ProgressionTracker parameter, letting a dialogue
id branch on which flags are already set rather than always
returning fixed lines — the actual payoff of Phase 28's progression
work. villager_1 (sprite, entity, Dialogue trigger) is the first
content built against this, with two branches depending on
necklace_consumed, confirmed firing correctly both ways in play.

Docs-as-code extended: docs/screenshots/ now covers m5-10 through
m5-11 — villager's two dialogue branches, confirmed distinct
before and after necklace_consumed is set.

### Phase 30 — Facing-Direction Rework, Scene-Builder Extraction, Debug Additions (completed)

A single, direction-agnostic trigger for villager (approachable
from any side) exposed that required_facing.contains(&player.facing)
alone can't distinguish "correctly positioned and facing toward" from
"facing the same absolute direction from the wrong side" — ADR-060's
original problem, previously worked around one trigger per side. Adds
is_facing_toward(player_center, target_center, target_half_size,
player_facing) — a relative, edge-aware check (not just center-to-
center) — combined with reinterpreting the trigger's facing field as
"which side(s) the object presents" rather than "which way the
player must look," related by direction-inversion
(Entity::match_facing_direction). A real, later-caught bug: the first
version of is_facing_toward compared only center-to-center direction,
which let a player standing flush against one side of a wide target
"face toward" it while looking along a perpendicular direction —
fixed by additionally checking the player falls within the target's
extent on the axis perpendicular to the facing direction. Lets the
patio door collapse from two triggers to one, and lets villager_1 be
approached from any of four sides with a single trigger.

try_interact's two branches also turned out to need genuinely
different proximity tests: Dialogue triggers require true flush
contact (player_flush_with, aabb_overlap against the player's actual
collider), Toggle triggers keep a vicinity check (player_near,
point_in_rect), since a toggled object's own collider can disappear
(an open door has none), leaving no flush geometry to test against.
DialogueTriggerSpec's per-NPC trigger_padding was replaced with a
flat +1.0-authoring-unit margin around the target's own collider.

The entity+trigger+prompt authoring helper (spawn_entity, spawn_player,
spawn_dialogue_trigger) was restructured as Scene methods
(engine/scene/builder.rs), not free functions in game/ — nothing in
the helper is actually game-specific, matching the same test that's
kept TextureStore/SpatialGrid in engine/ throughout. Making these
real Scene methods required Scene to exist before its content does,
which the old Scene::new (background/walls/triggers/entities/
player_index all constructor arguments) didn't allow. Scene::new now
takes none of them — all four collections start empty, player_index
is a placeholder until spawn_player sets it (private, no other
mutation path) — and home::build/village::build construct an
(initially empty) Scene up front, populating it directly:
scene.background.push(...)/scene.walls.extend(...) for hand-authored
content (no dedicated methods needed — no texture-loading-plus-ID-
bookkeeping problem to solve there), the three spawn_* methods for
the entity+trigger+prompt pattern repeated across the necklace, bed,
and villager_1.

Adds TriggerId, mirroring WallId/EntityId — the necklace's
deactivation-on-revisit logic needed to reference its own trigger by
index, and .last_mut() was fragile (had already broken once,
silently deactivating the wrong trigger after a reordering). An
intermediate attempt reused EntityId to index triggers, which
compiled but was wrong twice over (index computed against the wrong
Vec's length; EntityId is specifically meant to index Scene.entities).

Adds point_in_range, a shared one-dimensional bounds-check helper
extracted from point_in_rect and is_facing_toward once both were
found computing the same per-axis "is this scalar between a min and
max" check under different variable names.

Adds an F11 debug noclip toggle (DebugSettings.enable_player_collider)
for reaching content placed away from a scene's entry point without
fighting geometry — implemented as an early return in
try_move_player when disabled, not a flag threaded through
collider_blocked's call sites.

### Phase 31 — Render Pipeline Layering: Draw Extraction, HUD Split, Overlay Layer (completed)

First concrete steps of a longer-standing plan to organize rendering
into purpose-grouped passes (background+entities, overlay, debug,
dialogue/UI, debug-info) rather than one large render_scene body plus
a separately-scattered draw_hud. A depth buffer was considered and
deliberately not pursued — nothing in the game needs per-pixel depth
resolution within a tier, only draw-order control between a handful
of tiers, which purpose-grouped passes already provide without the
real new GPU surface (depth texture, pipeline depth-stencil state) a
depth buffer would need.

draw_background_and_entities extracted from render_scene as its own
callable method — the main y-sorted entity draw loop, unchanged in
behavior, verified by visual comparison before/after. render_scene
itself reduced to orchestration.

The former draw_hud (three unrelated things bundled: notifications,
dialogue, debug info, and the volume slider's *input handling*
tangled into what was nominally a draw call) split into
update_debug_ui (slider input handling only, called once per frame
alongside update_player, not from inside a draw path), draw_ui
(dialogue — permanent, player-facing UI, expected to grow toward a
real HUD: health, inventory, map), and draw_debug_info
(notifications, FPS, mouse position, the slider's draw call).
engine/debug/hud.rs renamed to info.rs, freeing "hud" to mean an
actual future player-facing HUD rather than collide with debug-only
text. draw_slider moved off info.rs entirely, now Slider::draw in
ui.rs — the widget owns its own update/build_rects/draw, the pattern
future inspector widgets should follow.

Entity gains is_overlay_layer: bool. Entities marked true (currently
only prompt icons) are excluded from the main y-sorted draw loop and
drawn in a dedicated second pass afterward, always on top regardless
of y-sort or camera position. Fixes a real, visible bug: villager_1's
prompt icon could be occluded by the player's sprite when approached
from above. Both passes share submit_sprite_draw, a helper extracted
from what was one large per-draw block, parameterized by whether that
draw is allowed to clear the screen — built as a deliberate
copy-paste first, verified working, then deduplicated, per this
project's usual incremental approach to nontrivial refactors.

update_interact_prompts switched from point_in_rect to aabb_overlap
against the player's real collider — the same flush-vs-point mismatch
already fixed for try_interact once tight, collider-sized triggers
made a center-only check inconsistent. PROMPT_MARGIN bumped from 5.0
to 15.0 authoring units, a felt adjustment now that the prompt
renders reliably on top.

Still open, not part of this phase: relocating draw_background_and_
entities/debug-rect-building/draw_ui/draw_debug_info into dedicated
files under renderer/layers/ (the file-organization half of the
layering plan); batching multiple differently-textured sprites into
fewer draw calls (needs a texture atlas or similar — a real step up
in complexity, deferred until draw count is a measured cost, per
ADR-040's original deferral).

Docs-as-code extended: docs/screenshots/ now covers m5-12. The
prompt icon rendering reliably on top of the player sprite via the
new overlay pass, the concrete case that motivated is_overlay_layer.

### Phase 32 — Render Layer Relocation & AppState Reorganization (completed)

Two purely structural commits closing out the render-layering plan
and cleaning up AppState's growing impl block, both verified
behavior-identical before/after.

render_scene deleted outright. draw_background_and_entities and
submit_sprite_draw (Phase 31) move to engine/renderer/layers/
entities.rs; the debug-rect building + render_solid_rects call
(previously inline in render_scene) becomes its own method,
draw_debug_geometry, in engine/renderer/layers/debug_geometry.rs.
draw_ui and update_debug_ui/draw_debug_info (AppState methods, not
Renderer methods, since they read AppState-only fields) move to
engine/app/layers/ui.rs and engine/app/layers/debug_info.rs,
mirroring the same one-file-per-layer structure on the AppState
side. RedrawRequested now calls all 5 layers directly and in order —
draw_background_and_entities, draw_debug_geometry, draw_ui,
update_debug_ui, draw_debug_info — as one visible sequence, rather
than through an intermediate render_scene wrapper.

AppState's impl block, grown large across M5's session, split by
behavior: engine/app/player.rs (update_player), engine/app/
scene_lifecycle.rs (build_scene/change_scene/reset_scene), and a
second impl AppState block appended to the existing engine/app/
dialogue.rs (tick_dialogue/play_blip/BLIP_PITCH_STEPS) — dialogue
audio stays with dialogue rather than getting a separate "audio"
file, since blip timing is directly driven by dialogue's character-
reveal ticking, not a general-purpose audio concern. A new
tick_frame_timing extracts the delta/smoothed-fps/frame-count
bookkeeping that previously lived inline in RedrawRequested; notify
and tick_frame_timing both stay directly on AppState in app.rs —
small, general utilities not judged worth their own file.

No screenshots this phase — both commits are pure reorganization
with no visual or behavioral signature distinct from what M5's
existing screenshots already show.

### Phase 33 — Tile-Based Scene Authoring: Design Session (completed, design-only, not yet implemented)

A design-only session, no code, triggered by revisiting ADR-028's
still-standing content-authoring clause in light of real isometric
pixel art research done since it was written. Distinguished carefully
from ADR-087's earlier, separate correction of ADR-028's projection-
math half — this session concerns the other half entirely: how scene
backgrounds get authored, not what camera angle renders them.

Settled: scenes move from one hand-painted background image each to
a small reusable isometric tile set (~10–12 pieces), assembled per
scene. Collision, entities, and furniture are explicitly unaffected —
this is a background-visuals-only change; walls stay hand-placed
`Rect`s, furniture stays hand-placed, nothing becomes tile-snapped.

Also settled, after real deliberation across several exchanges: this
warrants building the interactive tile-placement tool now, not
deferring it further — click-to-paint against live tile art is judged
the "real consumer" the standing inspector-deferral policy was
waiting for. Initial back-and-forth explored a copy-paste-to-source
workflow (paint live, print a `const` array to hand-transcribe) as a
way to get the ergonomic win without touching ADR-027's file-format
deferral; this was ultimately rejected in favor of a real minimal
file format once it became clear a hand-rolled parser for one fixed,
trivial shape (whitespace-separated integers, one row per line) is
*less* code than the transcription workflow, not more — and doesn't
actually reopen ADR-027, which was about deferring a *generic*
serialization solution for complex, evolving schemas, not about files
as a category. Recorded as ADR-096 (the format-of-backgrounds
decision) and ADR-097 (the format-of-persistence decision), kept
separate since they're independently reversible claims.

Explicitly deferred, not designed: furniture-via-click-placement
(acknowledged as a natural future extension of the same tool);
mouse-picking (screen-to-world, the inverse of ADR-087's projection)
is identified as a hard dependency for the painter to function at all,
but not yet written.

### Phase 34 — Debug/Inspector Fixes, Tile-Editor Cursor Highlight, TTF Text Support (completed)

Three related but separately-committed units of work, done in service
of Phase 33's tile-authoring plan (an inspector capable of readable
text was a real, named prerequisite for the eventual tile painter).

**Bug-fix pass**, prompted by a direct request to scrutinize a batch
of self-implemented changes before building further on top of them:
the tile-editor cursor was being set every frame regardless of
whether the mode actually changed (moved to fire only on the F8
toggle); dead frame-timing code and a stale, self-referential TODO
comment were removed; `SpatialGrid::cell_center_world` was extracted
after the same cell-to-world formula turned up independently a
fourth time (occupied cells, player neighborhood, and now cursor
highlight); and `InspectorSection` was given its own computed
`bounds: Rect`, so the volume slider derives its position from its
section's real bounds instead of independent hand-tuned offsets that
only coincidentally lined up — the same single-source-of-truth defect
already caught several times this session, here in new territory
(UI layout instead of world geometry).

**Tile-editor cursor highlight**: an F8-toggled mode showing a custom
cursor and highlighting whichever grid cell the mouse currently hovers
over (`build_cursor_highlight_mesh`), reusing the newly-shared
`cell_center_world` helper and the existing batched debug-rect path —
no new GPU work.

**TTF text support**, added *alongside* the existing bitmap font, not
replacing it — dialogue keeps its established blocky look; the
inspector's section titles needed something thinner. `fontdue`
rasterizes every printable ASCII glyph once at `Renderer::new` time;
a hand-rolled shelf-packer places them into one RGBA atlas texture,
recording each glyph's UV rect, advance width, and baseline offset.
Verified in two separate, deliberately isolated steps before any
draw path existed: first, a single rasterized glyph uploaded and
drawn as an ordinary sprite, proving the fontdue-to-GPU-texture chain
works at all; second, the full packed atlas dumped to a PNG and
visually inspected for overlap or corruption, proving the packing
math independent of any rendering code. Only after both were
confirmed correct was the real path built: `PositionedTTFGlyph`,
`build_ttf_text_mesh`, `render_ttf_text`, and `layout_ttf_text` are
new, deliberately parallel siblings to the bitmap-font path, not
a shared abstraction — TTF glyphs are proportionally spaced (a
per-glyph advance width) where the bitmap font is fixed-pitch, and
forcing one shape to cover both would have complicated the simpler,
working bitmap path for a need it never had. `Inspector::draw_section_
titles` confirms the result: mixed-case, mixed-descender text renders
on a visibly consistent baseline. `TTFFont` (an earlier bundled-struct
shape, superseded once the atlas texture and glyph map became
separate `Renderer` fields) and `TextureStore::insert_raw` (a
throwaway helper from the single-glyph verification step) were
removed once confirmed unused.

Docs-as-code extended: docs/screenshots/ now covers m5-13 through
m5-15 — the tile-editor cursor highlight (m5-13), the corrected
inspector section/slider layout (m5-14), and TTF section titles
rendering with a consistent baseline (m5-15).

Still fully open, unblocked by this phase but not advanced by it:
screen-to-world mouse picking (the tile painter's actual hard
dependency, per ADR-097), the tile painter itself, the tile-pixel-
size/isometric-`K` pairing (real tile art now exists — a 32x32 cube
pair — but the final scale hasn't been tuned against it), and cube-
tile depth-sorting (the real design consequence of committing to
full-cube rather than flat-top floor tiles, not yet designed).

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

### ADR-062: Text rendering batched into one draw call; dedicated text pipeline removed
- **Context:** render_text issued one full GPU submission (buffer allocation, bind group, command encoder, submit) per glyph, via a per-draw GlyphUniform remapping a shared unit quad's UV onto one atlas sub-rectangle each time. At real dialogue-length strings (~83 characters) this measurably cost ~40fps, dropping the game from ~60fps to ~18fps whenever text was on screen.
- **Decision:** build_text_mesh computes every glyph's final screen-space corner positions and atlas UV directly, once, on the CPU, producing one vertex+index buffer for an entire string. render_text uploads that once and issues a single draw_indexed call regardless of string length. Since baking UV directly into vertex data eliminated the only reason text needed its own shader (the per-draw remap), text_shader.wgsl, text_pipeline, GlyphUniform, and glyph_bind_group_layout are deleted; render_text now draws through the existing sprite pipeline (shader.wgsl), pointed at the glyph atlas bind group instead of a scene texture.
- **Rationale:** Per-primitive GPU submission cost (not the fragment/vertex math itself) was the actual bottleneck, per the same category of cost ADR-043 already named for debug rects — batching removes the submission count, not any rendering logic. That the fix also deleted a whole shader/pipeline pair, rather than just speeding up the existing one, was a direct consequence of the per-primitive *data* (UV offset) being fully bakeable into geometry, unlike debug rects' per-primitive *color/thickness* (see ADR-063, which could not take this same shortcut).
- **Consequences:** No typewriter/partial-reveal hook was added in this pass (deferred until the dialogue manager actually needs it); draw_indexed(0..N) on this same batched buffer is the identified extension point when that's built — reducing revealed character count needs no buffer rebuild, only a smaller index range.

### ADR-063: Debug-rect overlay batched via a dedicated per-vertex format
- **Context:** The debug-collider overlay (ADR-043) paid the same per-rect GPU submission cost as text, at "a dozen or so" rects — explicitly accepted at that scale, explicitly flagged for revisit if rect count grew. Adding center-position crosshair markers (one per wall, collider, and trigger, plus one at the world origin) roughly tripled per-frame rect count, crossing that threshold — F1 became unusable.
- **Decision:** A new DebugVertex format carries fill_color, border_color, and border_thickness as per-vertex attributes (duplicated identically across a rect's 4 corners) instead of a per-draw DebugRectUniform. debug_shader.wgsl's border-math fragment logic is unchanged — only its inputs move from a uniform to interpolated vertex attributes. build_debug_mesh batches every debug rect for the frame into one shared vertex/index buffer, drawn with a single draw_indexed call; debug_bind_group_layout is removed, since the debug pipeline's only remaining binding is the shared transform uniform.
- **Rationale:** Unlike text's UV offset (a single value fully determining what to sample), a rect's color/thickness values are inputs to the fragment shader's own computation, not something bakeable into shared geometry alone — this is why debug-rect batching needed a genuinely new vertex format rather than reusing an unmodified existing pipeline the way text could. The underlying fix (move per-primitive data from a uniform to per-vertex attributes, collapse many draws into one) is the same principle as ADR-062, applied to a case where the data being varied is richer.
- **Consequences:** Confirmed via direct before/after screenshots with the smoothed on-screen FPS counter — frame rate returned to near-baseline with center markers enabled. Any future debug-visualization addition (e.g. a facing-direction indicator) should default to this same batched-mesh pattern rather than reintroducing a per-primitive draw loop.

### ADR-064: TextureStore::take, explicitly unsound outside a throwaway store
- **Context:** The font atlas asset was migrated to .aseprite mid-session but briefly loaded via Texture::from_bytes directly, bypassing TextureStore::load's existing extension-based dispatch (ADR-050) — a runtime decode failure, since from_bytes's PNG/JPEG decoder has no knowledge of Aseprite's binary format. Reusing TextureStore::load's dispatch instead required a way to extract an owned Texture back out, since TextureStore::get only borrows and the store built for this one-off load doesn't outlive Renderer::new.
- **Decision:** TextureStore::take(id) removes and returns ownership of a texture via Vec::swap_remove, documented explicitly as sound only for a store about to be discarded with nothing else holding an index into it.
- **Rationale:** swap_remove was chosen over remove for both correctness-adjacent honesty and (at this scale, immaterial) efficiency: on a genuinely throwaway single-texture store there is nothing "behind" the removed element to shift either way, but swap_remove's O(1), no-reordering-of-the-rest semantics better signal "I don't care what happens to this collection afterward" than remove's order-preserving intent. On any store with more than one entry and a longer lifetime, either method would invalidate later TextureIds (ADR-036's stable-index guarantee) — take() must never be called on a real scene's TextureStore.
- **Consequences:** The doc comment on take() carries this warning directly, since nothing in the type system currently prevents misuse on a longer-lived store — a future correctness gap, accepted at this scale (one call site, unlikely to be reused incorrectly) rather than building an enforcement mechanism now.

### ADR-065: Dialogue content authored as structured colored spans, not markup
- **Context:** Per-word/phrase text coloring (e.g. tinting "dangerous" red within an otherwise white line) needed a way to associate color with a sub-range of a line's text.
- **Decision:** DialogueLine holds Vec<ColoredSpan> (text + color per span) rather than a flat &str with inline markup syntax.
- **Rationale:** A markup syntax parsed at runtime is a real, if small, parser — unjustified machinery at current content volume, and inconsistent with D-C's static-Rust-data approach (ADR-027). Structured spans get the same capability as plain, explicit Rust values, authored directly in game::dialogue::line_for.
- **Consequences:** More verbose to author than inline markup would be (each colored sub-run is its own explicit struct literal) — an accepted, deliberate cost given content volume remains small.

### ADR-066: Shared Vertex format gains a tint field
- **Context:** Per-span dialogue coloring needed a way to multiply a color into each glyph's sampled texture. Text already shares the ordinary sprite pipeline (ADR-062) with no per-vertex color concept.
- **Decision:** Vertex (used by every sprite and by batched text alike) gains a tint: [f32; 4] field, multiplied into the sampled color in shader.wgsl's fragment shader. Every existing static VERTICES entry sets it to white (a no-op multiply), so ordinary sprites are unaffected.
- **Rationale:** Extending the one shared vertex format, rather than building a second, text-only tinted variant, avoids re-diverging text from sprites right after ADR-062 unified them — the cost (one more vec4 per vertex, unused by sprites) is trivial next to reintroducing a parallel pipeline.
- **Consequences:** Any future sprite-tinting need (a damage flash, a fade) already has the capability available for free.

### ADR-067: Debug-rect pipeline generalized into a general solid-rect primitive
- **Context:** The dialogue panel needed a solid-colored background rectangle — mechanically identical to what the F1 debug overlay's batched-rect pipeline (ADR-063) already draws, just for a permanent, player-facing purpose rather than a dev toggle.
- **Decision:** DebugRect/DebugVertex renamed to SolidRect/SolidVertex; the draw path generalized into Renderer::render_solid_rects, taking an explicit projection/view pair rather than assuming render_scene's world-space transform — letting the F1 overlay (world-space) and the dialogue panel (screen-space, matching render_text's ADR-058 convention) share one mechanism correctly.
- **Rationale:** The debug overlay was the pipeline's first consumer; the dialogue panel is its second, and per this project's established pattern, a second real consumer is exactly when a name/API should generalize beyond its original single-purpose framing — not before.
- **Consequences:** Any future solid-color UI need (a health bar background, a menu panel) has a ready, batched, tested primitive to build on.

### ADR-068: Facing debug marker redesigned from a symmetric line to a tapering arrow
- **Context:** push_facing_marker's first version drew one rect centered on a point offset in the facing direction — visually symmetric, so Up and Down (and Left and Right) produced identical-looking lines, confirmed ambiguous by placing two oppositely-facing beds side by side.
- **Decision:** Three rects along the facing axis, decreasing in thickness toward the tip (5.0 -> 3.0 -> 1.5 px), producing a triangle-like silhouette that unambiguously points away from center — using nothing but the existing batched solid-rect pipeline, no new geometry primitive (e.g. a true triangle) built.
- **Rationale:** A single rect cannot encode direction at all, only axis — no comparison or math fix addresses this, only an asymmetric shape can. Three tapering segments achieve that using only the rectangle primitive already available, avoiding new pipeline work for a debug-only visual.
- **Consequences:** A genuine triangle/arrowhead primitive remains a possible future upgrade if more debug/editor visuals need it, not built now since the tapering-rect approximation already reads clearly.

### ADR-069: Collision resolution lands the player flush against colliders via exact geometry
- **Context:** A blocked movement step was previously rejected outright rather than partially resolved, so the player's actual stopping distance from an obstacle depended on wherever they happened to be the previous frame — never exactly flush, worse at higher speed.
- **Decision:** collider_blocked returns Option<Rect> (the specific blocking obstacle). A blocked axis resolves to the exact position where the player's collider edge touches the obstacle's edge, computed directly from both rects' geometry each frame — not from delta or a cached gap value.
- **Rationale:** The correct resting position is a fixed geometric fact independent of movement speed; computing it directly (rather than approximating via partial steps or accumulated deltas) is both simpler and immune to the frame-rate-dependent jitter an intermediate delta-based attempt produced. Required aabb_overlap's boundary comparison to change from strict `<` to `<=`, since exact flush contact (the new, common resting state) must correctly register as "still touching," not "just cleared."
- **Consequences:** Confirmed against straight approaches from all four directions, diagonal sliding along a flat wall, and diagonal movement into a true inside corner.

### ADR-070: TriggerKind split into Warp, Dialogue, and Toggle
- **Context:** All non-Warp interactions had shared one Interact variant (proximity+facing+button starting a dialogue). A door needing to open/close on every press, with no conversation, didn't fit that shape.
- **Decision:** Interact renamed to Dialogue; a new Toggle variant flips a target entity's texture and collider directly, with no dialogue involved. State is inferred from which texture the entity currently holds (ADR-037's Option-as-state pattern), not tracked as a separate flag. Dialogue's prompt_entity/prompt_texture become Option, since this game's design shows "press E" prompts only for the first couple of tutorial interactions, never afterward.
- **Rationale:** A door's toggle and a conversation's start are genuinely different actions with different data needs (Toggle has no id/prompt/facing-list-per-side content at all); forcing both through one variant would mean irrelevant fields on every construction site. TextureStore::load_aseprite_frame (explicit frame selection) was added alongside this, since Toggle's two visual states are two frames of one file.
- **Consequences:** try_interact's return type became InteractResult, an enum the caller matches on to dispatch correctly — any future third interaction kind gets its own arm here, not a generalized callback (still deliberately not built, per ADR-046).

### ADR-071: Permanent trigger deactivation and entity consumption for one-shot pickups
- **Context:** The necklace needed to remove itself (sprite, collider, prompt icon, and its own trigger) after its examine dialogue finished — a one-way, permanent state change, distinct from Toggle's bidirectional flip and from recently_used's transient, self-re-arming suppression.
- **Decision:** DialogueLine gains consumes_entity: Option<EntityId> — narrow and single-purpose, naming exactly which entity to remove when that specific line is the dialogue's last and it closes, not a general post-dialogue hook. Trigger gains a permanent active: bool, checked in every function that reads triggers (check_triggers, try_interact, update_interact_prompts, and the F1 debug-rect overlay). Scene::consume_entity clears the target entity's texture/collider, deactivates the owning trigger, and separately clears that trigger's own prompt-icon entity — a distinct entity from the one being consumed, whose visibility would otherwise freeze at whatever state it held the instant the trigger deactivated, since a deactivated trigger no longer participates in update_interact_prompts at all.
- **Rationale:** A generic post-dialogue callback mechanism was considered and rejected (same reasoning as ADR-046's earlier trigger-dispatch decision) — one narrow field solving the one real case in front of the project, not speculative machinery for cases that don't exist yet.
- **Consequences:** All four trigger-reading call sites needed the active guard independently; missing any one of them (as initially happened for the debug overlay) leaves a stale, non-functional trigger still visibly or functionally present. Confirmed via a before/after screenshot pair showing the necklace's complete removal.

### ADR-072: kira chosen over rodio for game audio
- **Context:** M4's blip system needed a Rust audio crate. kira had been listed in the project's tech stack table since Phase 0, uncontested; worth verifying rather than treating as a settled default given every other dependency choice this session (asefile, the font atlas) was actually compared against alternatives first.
- **Decision:** kira, confirmed as the right choice on reflection rather than replaced.
- **Rationale:** kira is purpose-built for game audio — tick/clock-synced playback, per-instance pitch and volume control via Tween/Value types — exactly the shape blip-syncing needs. rodio is a general-purpose playback library (decode, play, pause, loop) with no equivalent game-specific timing/modulation layer; using it would mean building that layer by hand on top. No third serious contender was found in the Rust audio ecosystem for this use case.
- **Consequences:** Confirmed via kira's own documented examples (Tween-based playback rate changes, Clock-based timed playback) matching the project's actual need closely enough that no workaround or hand-rolled timing layer was required.

### ADR-073: Slider rejects a callback field in favor of direct value access
- **Context:** Autocomplete suggested `on_change: Option<Box<dyn Fn(f32)>>` on the hand-built Slider widget, the standard callback/event-hook pattern for a general-purpose, reusable UI library. The developer's stated goal (a decoupled Slider usable for a future inspector panel) made it worth genuinely evaluating rather than defaulting to whichever was simpler for today's single volume-knob use case.
- **Decision:** No callback field. `Slider::update(mouse_pos, mouse_down) -> bool` returns whether the value changed this call; the caller reads `slider.value` directly, in the same frame, at the same call site.
- **Rationale:** Checked against egui's actual, real Slider API — `ui.add(egui::Slider::new(&mut value, range))` — which uses the identical mutate-in-place-and-check-a-response-flag shape, not a registered closure. Callbacks are the retained-mode UI pattern (persistent widgets, external code reacting to events fired later); this project's render loop already rebuilds and redraws everything every frame, which is structurally immediate-mode already. The callback version wasn't a more general form of the same design — it was a different paradigm, borrowed from a different kind of UI framework, that would need to be undone rather than extended if a real inspector is built later.
- **Consequences:** Confirms the project's "well-structured version costs the same, so build it" standing correction (per the developer's explicit request to be called out when this applies, not just when a simpler version is being over-built) — the bool-return version is not the compromise here, it's the version already matching what a real future inspector would want.

### ADR-074: She returned home; the storm woke her; gate sabotage is premeditated ambush-routing (supersedes ADR-019)
- **Context:** ADR-019 stated she never came home through the storm. Necklace placement, decided during M4 implementation (behind the bed, caught on the headboard), only makes sense if she *was* in that room that night — direct tension with the original claim. Separately, ADR-019 left "why sabotage the gate" underspecified beyond generic cover.
- **Decision:** She came home, was woken by the storm itself partway through the night, and left again in a hurry — no lights (relying on lightning flashes), grabbing a cloak, not noticing the necklace slip loose and catch on the headboard. No struggle, no intruder in the house; the room stays undisturbed except for that one small, ordinary object out of place. Separately: the main gate's storm damage is real, but the cult deliberately finishes the sabotage afterward, that same night, because they'd already learned this is the route she takes — forcing her onto the cemetery path specifically to stage the ambush there, not merely to slow later pursuit.
- **Rationale:** Preserves everything ADR-019 actually needed (real panic from an empty-but-recently-occupied room; natural, rail-free routing for the player toward the back exit) while fixing a real plausibility gap: a struggle in a shared bedroom is unstageable without the player character improbably sleeping through it or unbuilt magic explaining why he didn't (ADR-023's show-then-tell logic explicitly reserves that kind of reveal for later). A hurried, self-caused departure needs no such patch. Turning the sabotage into deliberate ambush-routing (rather than incidental cover) answers a "why would they bother" question the original ADR left open, and reads as competent antagonists exploiting a real coincidence rather than antagonists with unstated weather control.
- **Consequences:** Beat 2's already-shipped dialogue (Phase 21) required no changes — "rumpled sheets," "the storm... woke her," and the necklace's felt-wrongness line already fit this reading. Beat 3's flat-plank foreshadowing (ADR-019) is unaffected — the plank still marks the cult's finishing touch on the gate. Any future Beat 4/5 writing must not imply she was taken from the house.

### ADR-075: Ancestral protector lineage — the sigil's true origin
- **Context:** The sigil's ability to absorb and convert z-essence (ADR-017) was previously justified only by in-fiction physics ("energy can't be destroyed, has nowhere else to go"), which explains the *currency* but not why this specific object is built to receive it, nor why the sibling tether (ADR-019/023's justification for the plank and the map) is more than a metaphor.
- **Decision:** The protagonist's family is a lineage of village protectors, predating the current cult conflict, whose role has been standing against exactly this kind of dimensional predation. The role — and the sigil that channels it — passes to the eldest child of each generation, a specific inherited family tradition rather than a general claim about gender. The sister carries it now; wards placed and maintained around the village (the chalk, candles — Beat 4's existing clue content) are this duty in practice, needing periodic renewal, which the storm's timing threatened (rain washes chalk sigils). The totem's ability to absorb z-essence isn't found magic — it was made for this. The sibling tether that lets the player feel the plank's wrongness (ADR-019) and later use the cult's own map (ADR-023) is this same bloodline conduit, not sentiment alone. Cultists are cast as this lineage's opposite number — reject vs. protect a dimension — extending ADR-006's existing reject/inherit juxtaposition rather than adding a new one.
- **Rationale:** Converts several previously separate "just because" justifications (why the tether works, why the totem does what it does, why she has it) into one mechanism with a shared cause. Framing the inheritance as this-specific-family's tradition (not a general claim about women) keeps the intended message — a chosen lineage of selfless duty — without the framing reading as a claim about gender broadly.
- **Consequences:** Implementation note: the necklace examined in Beat 2 (`necklace_examine`) *is* the ADR-018 keepsake/sigil — Beat 1's originally-planned separate keepsake-examine interactable was never built as its own thing; this pickup consolidates it. No content changes required. Opens, not answers, a question worth holding for later writing: why the founding ancestor chose eldest-child inheritance specifically. The cult's status as an old, patient threat (not a recent upstart) is now implied and should inform any future cult-history writing.

### ADR-076: Sister's clue-chain activity reframed as opposition, not entanglement
- **Context:** Phase 3's arc and Beat 4's clue content (villagers seeing her "scurry," buying chalk/candles/a scarf) were originally ambiguous enough to read as her being drawn toward or investigating the cult out of curiosity — workable, but weaker than available alternatives once ADR-075 established she has an active protective duty.
- **Decision:** She was maintaining protective wards against the cult, not fraternizing with or investigating it. The same observed behaviors (odd purchases, secretive movement) now read as ward-maintenance in progress, timed urgently against the storm.
- **Rationale:** "She was quietly investigating something spooky" makes her an unlucky bystander; "she was doing dangerous, secret work to protect others, alone, and told no one" makes her exactly the kind of unregretted, unknown sacrifice the ritual requires (ADR-007), and gives the cult a sharper motive (neutralizing a real threat, not grabbing a convenient target) that fits the acknowledgment-ladder enemy dialogue design (ADR-016) better than incidental victimhood.
- **Consequences:** Beat 4's actual dialogue is still unwritten (per DERIVATION, Phase 1 scope) — this is a framing constraint for whenever it's drafted, not a content change today. What the candles/chalk/scarf are *specifically for* (ward materials, ward locations) remains open and does not need resolving now.

### ADR-077: Renderer split into mesh/gpu/draw submodules
- **Context:** `renderer.rs` mixed pure CPU vertex-building math with GPU resource lifetime (`Frame`/`Renderer` init, frame acquire/present) and the actual per-frame draw-call methods, all in one file.
- **Decision:** Three submodules under `renderer/`: `mesh.rs` (CPU-only geometry building, no `wgpu` handles touched), `gpu.rs` (`Frame`/`Renderer` struct definitions and GPU lifecycle), `draw.rs` (the `impl Renderer` draw-call methods). `renderer.rs` becomes a thin facade — `mod` declarations plus `pub use gpu::{Frame, Renderer}; pub use mesh::SolidRect;`.
- **Rationale:** CPU math and GPU state have genuinely different failure modes and different things a reader needs to know to safely change them; separating them makes each file's job legible from its name alone. Draw-call methods stayed `pub fn`, not `pub(super)`, since they're called from `app.rs`, a sibling of `renderer` rather than a descendant of it — Rust module privacy only extends to descendants of the declaring module.
- **Consequences:** `include_wgsl!` paths in `gpu.rs` needed updating (macro paths resolve relative to the file they're written in, not the crate root) when the shaders later moved into `renderer/shaders/` (ADR-081).

### ADR-078: Scene-building content extracted from Scene into game/scenes
- **Context:** `Scene::new_home`/`Scene::new_outside` — which furniture, which warps, which dialogue triggers exist in each scene — lived as methods on the engine's own `Scene` type, mixing reusable scene mechanics with Beat-1-specific authored content.
- **Decision:** Both moved to `game/scenes/home.rs`/`outside.rs` as free functions (`pub fn build(device, queue, multiplying_factor) -> Result<Scene>`), leaving `Scene` holding only mechanics any scene needs.
- **Rationale:** Direct application of the project's machinery/content test (§2.2) to the engine's own internals: could a different game reuse `Scene`'s collision and trigger-resolution code unedited? Yes. Could it reuse `new_home`'s furniture placement? No — that's this game's content.
- **Consequences:** `game/scenes.rs` becomes the project's second real `src/game/` content module (after `game/dialogue.rs`, Phase 14) — the pattern later scenes (M5's village) should follow rather than growing `Scene` itself.

### ADR-079: Input abstraction resolved — InputState (engine) vs. Action (game) (resolves ADR-035)
- **Context:** ADR-035 flagged raw `KeyCode` handling inline in `App` as temporary debt, deferred until a second input-consuming system existed to teach the right abstraction shape.
- **Decision:** `InputState` (engine) holds a `HashSet<KeyCode>` of currently-held keys behind `press`/`release`/`is_held`. `Action` (`game/actions.rs`) is a small enum (`Interact`, `AdvanceOrSkip`) covering discrete, press-triggered things a player can do; `resolve_key_press(code, dialogue_active) -> Option<Action>` maps a raw press to one, contextually. Movement stays a direct `resolve_movement(&InputState) -> Vec2` reading raw WASD state, deliberately excluded from `Action`, since it's continuous held-state, not a discrete press.
- **Rationale:** Dialogue's advance/skip input was the second real consumer ADR-035 was waiting on. Splitting raw engine-level input from game-level semantic actions keeps `engine` ignorant of what E or Space *mean* to this specific game, matching ADR-026's engine/game module boundary.
- **Consequences:** F1/F3/Escape/Ctrl+R dev-tooling bindings stayed in `app.rs` directly, not routed through `Action` — they're legitimately engine-level (any game built on this engine would want the same debug toggles), not game content.

### ADR-080: ids.rs eliminated; identifiers and Trigger/Background relocated to their owning module
- **Context:** `engine/ids.rs` grouped `EntityId`, `TextureId`, `WarpId`, `SceneId` together, specifically to break a circular dependency: `Trigger` (living in `entity.rs`) needed `SceneId`/`WarpId`, which conceptually belong to scene identity, not entity identity. On review, grouping types by *kind* rather than by the struct each belongs to was judged not idiomatic Rust.
- **Decision:** `EntityId` and `WarpId` moved into `entity.rs`; `TextureId` moved into `texture.rs`. `Trigger`, `TriggerKind`, `WarpId`, and `Background` all moved out of `entity.rs` into `scene.rs` — not because `Trigger` resembles `Collider` (a genuinely different mechanism: `Trigger` uses a point-in-rect test against the player's center, `Collider` uses full AABB-vs-AABB overlap), but because `Scene` is what actually owns a `Vec<Trigger>`/`Vec<Background>`, while `Entity` never held a reference to either. `SceneId` moved into `scene.rs` alongside them.
- **Rationale:** The deciding test, applied consistently: a type lives wherever a `Vec<T>`/`Option<T>` of it is actually held as a struct field, not wherever it happens to be referenced from or resembles a neighboring type. This resolves the original circular-dependency concern more honestly than `ids.rs` did — `entity.rs` ends with zero imports from `scene.rs`, a genuine one-directional dependency, rather than a neutral third file hiding a coupling that was real either way.
- **Consequences:** `TextureId`/`Texture`/`TextureStore` were briefly marked `pub(crate)` (an autocomplete suggestion, after a `pub(super)` attempt failed since the real callers — `entity.rs`, `scene.rs`, `game/scenes/*.rs` — aren't descendants of `renderer`) before reverting to plain `pub`, matching every other type in the crate; `pub(crate)` vs. `pub` is behaviorally identical in a single, unpublished binary crate (ADR-026), so adopting it project-wide is a separate, deliberate style decision for later, not one to make by accident in one file.

### ADR-081: Engine files reorganized by owning/consuming module
- **Context:** Flat `engine/` mixed thin mod-declaration facades (`debug.rs`, `renderer.rs`, `platform.rs`) with substantial single-purpose files (`entity.rs`, `input.rs`, `text.rs`, `texture.rs`, `ui.rs`) and two loose `.wgsl` shader files, with no organization beyond "lives directly under `engine/`."
- **Decision:** `shader.wgsl`/`debug_shader.wgsl` moved into `renderer/shaders/` (their sole consumer, `gpu.rs`, is one directory away). `text.rs` moved into `renderer/` (used only by `renderer/mesh.rs`/`draw.rs`). `texture.rs` moved into `renderer/` too, after weighing `Scene`'s ownership of the `TextureStore` field against `texture.rs`'s content being entirely wgpu resource plumbing that only `gpu.rs` does anything mechanical with — the latter won out. `input.rs` moved into `platform/` (its only holder is `AppState`), and `platform.rs`/`platform/` renamed to `app.rs`/`app/` to match its actual contents (`App`/`AppState`) and read better at the call site (`engine::run` vs. `engine::platform::run`). `ui.rs` moved into `debug/` once its only current use — a `show_debug_info`-gated volume slider, sitting in the same conditional block as the FPS counter and mouse-position debug readout — showed it wasn't the general-purpose widget its name implied; a `TODO` marks the eventual rename to `inspector.rs` once it holds more than one widget, deliberately not renamed now (nothing to generalize yet, per ADR-025).
- **Rationale:** A folder should exist because more than one file genuinely belongs together, not for its own sake — `entity.rs`/`scene.rs` stayed flat despite sitting beside folder-having siblings, since neither had a second file to share a folder with yet.
- **Consequences:** `engine.rs`, `renderer.rs`, and `app.rs` needed their `mod`/`pub mod` visibility widened in a few places (`pub mod app`, `pub mod renderer`) specifically because `game/`-side content now reaches across the `engine`/`game` boundary into `renderer::texture` and `app::input` — a real, necessary widening, not overexposure, since privacy otherwise already covered every same-crate case that mattered.

### ADR-082: Scene's impl block split into scene/mechanics.rs
- **Context:** `scene.rs`, once it absorbed `Trigger`/`TriggerKind`/`Background`/`SceneId` (ADR-080), reached ~470 lines — comparable to `platform.rs`/`app.rs` before its own `dialogue`/`input` extraction — with type definitions and the dozen-method `impl Scene` block both in one file.
- **Decision:** `impl Scene { ... }` (`player`, `collider_blocked`, `check_triggers`, `try_interact`, `update_interact_prompts`, `activate_warp`, `try_move_player`, `toggle_entity`, `consume_entity`) moved to `scene/mechanics.rs`, importing what it needs from `scene.rs` via `super::`. `scene.rs` keeps the type definitions and the `Scene` struct itself.
- **Rationale:** Directly mirrors `draw.rs` holding `Renderer`'s implementation separately from `renderer.rs`'s type/facade role — the same split applied to `Scene` once it grew to the size that motivated the renderer split in the first place.
- **Consequences:** `entity.rs` did not receive the same treatment — at 110 lines, with no seam as clear as "types vs. a large impl block," forcing a folder onto it would organize for symmetry's sake rather than a real reason.

### ADR-083: Debug HUD drawing extracted from app.rs into debug/hud.rs
- **Context:** `AppState::draw_hud` mixed three unrelated concerns: the toast-notification renderer (not gated by `show_debug_info`, but exclusively fed by the Ctrl+R dev-reset action today), the dialogue panel (core gameplay HUD, unrelated to debug), and the actually `show_debug_info`-gated FPS counter, mouse position readout, and volume slider.
- **Decision:** The notification, FPS-counter, mouse-position, and slider-drawing logic moved to new functions in `engine/debug/hud.rs`, each taking only the specific data it draws (`&mut Vec<Notification>`, `smoothed_fps: f32`, `&[SolidRect]`) rather than `&mut AppState`. `app.rs`'s `draw_hud` stays the orchestrator, calling these and making the actual `self.renderer.render_*` calls itself. The volume slider's mouse-driven value update stayed in `app.rs`, kept separate from its own drawing.
- **Rationale:** Mirrors `debug::overlay::build_debug_rects` already taking just `&Scene` rather than reaching into caller state — `debug` stays a leaf module every other part of the engine can call into, rather than needing to know `AppState`'s shape, the same reasoning behind `entity.rs` not depending on `scene.rs` (ADR-080).
- **Consequences:** Notification rendering's move into `debug/` is provisional on its current single producer being dev-tooling; a real gameplay toast system would be the second-consumer moment (per ADR-025) to reconsider whether it still belongs there.

### ADR-084: Spatial grid for collision broad-phase, static grid only for now
- **Context:** `collider_blocked` scanned every wall and every entity
  in the scene on every collision check, twice per frame per moving
  entity (once per axis) — fine at M3/M4 content scale, named as a
  parked concern once M5's populated village would make the cost
  measurable (Phase 20's parked item).
- **Decision:** A sparse `SpatialGrid` (`HashMap<(i32,i32), Vec<CollisionHandle>>`)
  buckets walls and non-player entity colliders by cell at scene-load
  time. `CollisionHandle` is a `Copy` enum (`Wall(WallId)` |
  `Entity(EntityId)`), extending ADR-025/036's indices-over-references
  principle rather than storing borrowed rects. Collision checks query
  a fixed-radius neighborhood (`collision_handles_around_position`)
  instead of the whole scene, then run the same `aabb_overlap` test
  against only those candidates. The player is excluded from this
  grid — it's the one thing in the scene that moves every frame, and
  a static, build-once structure can't stay correct for that. A
  dynamic grid for movers is scoped as a deliberate, separate
  follow-up, not built in this pass.
- **Rationale:** Matches the parked item's own stated trigger
  (M5's village) and its own proposed mechanism (grid cells, `usize`/
  typed-index membership) closely. Splitting static (build-once) from
  dynamic (updated-on-move) rather than one combined grid avoids
  paying re-bucketing cost for content that never moves, and is
  already the correct shape for the player alone today, not just a
  hedge against future NPCs.
- **Consequences:** `Scene.walls` moved from `Vec<Rect>`'s originally
  documented shape (ADR-038) to `Vec<Collider>` at some point between
  ADR-038 and this pass — noted here since `WallId` indexes into that
  Vec and the drift wasn't otherwise recorded. `Scene` gained its
  first non-`pub` field (`static_grid`) and, as a direct consequence,
  its first real constructor (`Scene::new`) — every `Ok(Scene { ... })`
  struct-literal construction (`home::build`, `outside::build`) had to
  change, since private fields can't be set via struct literal from
  outside the struct's own module. The dynamic grid, quadrant-narrowed
  neighbor queries (vs. today's flat radius), and scene draw-call
  batching (Phase 20's other parked item, independently relevant here
  since debug visualization adds more per-frame rects) remain open,
  named follow-ups.

### ADR-085: DebugFlags consolidates debug toggles; debug screen has a master switch
- **Context:** Debug display state had grown from two independent
  `AppState` bools (`show_colliders`, `show_debug_info`) to needing
  six, once grid-line/occupied-cell/player-neighborhood visualization
  each wanted their own toggle — the point ADR-081's `ui.rs` TODO had
  already named as the trigger for treating debug tooling as a real,
  growing surface rather than one-off flags.
- **Decision:** A `DebugFlags` struct groups all six flags off
  `AppState` as one `debug` field. Each flag has a `toggle_*(&mut self)
  -> &'static str` method — flips the bool, returns a human-readable
  status string — rather than `AppState` flipping bools directly and
  separately constructing feedback text at each call site. One flag,
  `show_debug_renderer`, is treated as the debug screen's master
  switch (bound to F3): grid-line visualization checks
  `show_debug_renderer && show_grid` rather than `show_grid` alone.
  Collider overlay, FPS counter, and mouse-position readout are not
  yet nested under this same master switch — a deliberate, named gap,
  not an inconsistency.
- **Rationale:** Six near-identical `Notification` construction sites
  in the F-key handler collapsed to two lines each via a shared
  `AppState::notify(impl Into<String>)` helper, itself accepting
  `impl Into<String>` (not just `&str`) specifically so both plain
  string-literal toggle messages and `format!`-built messages (e.g.
  the existing scene-reset notification) work through one signature
  without call-site borrowing gymnastics. The master-switch framing
  treats the debug screen as an inspector-in-progress (per ADR-081's
  parked `inspector.rs` rename) rather than a flat list of unrelated
  toggles — grid visualization is the first thing built with that
  hierarchy in mind from the start, rather than needing to be
  retrofitted into it later.
- **Consequences:** Key bindings changed from the established F1
  (colliders) / F3 (debug info) layout to F1 (debug info) / F2
  (colliders) / F3 (debug screen master) / F4 (grid lines) / F5
  (player neighborhood) / F6 (occupied cells) — a deliberate breaking
  change to existing muscle memory, not incidental drift. Nesting
  colliders/FPS/mouse-position under the same master switch grid
  visualization now uses is a named, tracked follow-up. A hotkey-
  reference panel and a possible second, smaller-legible font
  (bitmap-atlas redraw vs. a genuine TTF/OTF rasterization pipeline —
  the latter would reopen ADR-056's scope, not just extend it) are
  both parked as real, near-term needs once `ui.rs` becomes the
  inspector ADR-081 already anticipated — not built in this pass.

### ADR-086: Dynamic grid for the player, merged with static grid at query time
- **Context:** `static_grid` (ADR-084) can't represent movers — the
  player needed its own grid, rebuilt as it moves, without inventing
  a second collision-query code path.
- **Decision:** `Scene.dynamic_grid`, a second `SpatialGrid`, rebuilt
  from scratch at the start of every `try_move_player` call from the
  player's current collider. `collider_blocked` queries both grids
  and unions the results before `aabb_overlap`.
- **Rationale:** A full per-frame rebuild, not incremental
  cell-boundary tracking, since a single mover makes incremental
  bookkeeping unearned complexity right now — the same "build the
  version that fits today's actual need" instinct as every prior
  deferred-generalization ADR.
- **Consequences:** Architecturally inert today (the player already
  excludes itself via `skip_index`) but establishes the
  "snapshot-then-query-uniformly" pattern correctly before a second
  mover (an NPC) exists to depend on it. `STATIC_CELL_SIZE` renamed
  `CELL_SIZE`, shared by both grids.

### ADR-087: Isometric projection is render-time-only; point-shear for sprites, shape-shear for debug geometry (supersedes ADR-028's flat-renderer framing)
- **Context:** ADR-028 committed to a straight orthographic renderer,
  deferring true isometric projection indefinitely. A dedicated
  planning session revisited this ahead of M5 village content, since
  the project's actual visual identity target was isometric from
  Phase 1's original concept discovery.
- **Decision:** All game logic (entity positions, colliders, triggers,
  spawn points, camera targets) stays plain orthogonal 2D world
  space, unchanged. The isometric look is produced entirely by a
  render-time projection transform. AABB collision (ADR-038) is
  retained unchanged; OBB evaluated and rejected as unneeded.
  Sprites/background are point-sheared only (anchor position
  transformed, quad shape untouched) since isometric art is expected
  to already look correct from that angle; debug geometry (grid
  lines, colliders) is shape-sheared (the whole rect deformed), since
  a flat world-space square genuinely renders as a diamond from this
  angle. An F10 toggle makes both projections live-comparable.
- **Rationale:** The point-vs-shape split was arrived at after an
  initial version (shear folded uniformly into `camera_view`)
  visibly skewed sprite shapes — sprites need placement to move, not
  their geometry to deform; debug geometry needs the opposite,
  since it's showing real world-space shapes from an oblique angle,
  not pre-drawn art.
- **Consequences:** `K` (the shear's scale factor) is a placeholder,
  tuned by eye rather than derived, matching `CELL_SIZE`'s precedent.
  Every other part of ADR-028 (no tilemap engine, hand-authored
  scenes) remains in force.

### ADR-088: Isometric movement as a hand-authored 8-direction table, not the projection's formula inverse
- **Context:** The natural first approach — remap raw WASD input via
  the true mathematical inverse of the isometric shear — produced
  movement where every key combination looked screen-cardinal.
  Manual comparison against MegaMan Battle Network 6 (the project's
  actual isometric-overworld reference) showed this was the wrong
  scheme entirely: MMBN's single-key presses look screen-cardinal
  and move world-diagonal; two keys together look grid-diagonal and
  move along one world axis.
- **Decision:** `resolve_isometric_movement` is a direct match on the
  four WASD keys, returning one of 8 pre-derived unit vectors — not
  a formula applied uniformly to all 8 cases.
- **Rationale:** Verified algebraically that the desired scheme is
  not achievable as one linear transform: the shear's linearity
  forces "single-key screen-cardinal" and "two-key screen-diagonal"
  to be mutually exclusive under any sum-then-transform approach.
  This is a deliberate control-scheme/art choice (matching diagonal
  player sprites planned for later), not a formula gap.
- **Consequences:** Every table entry is unit-length, so
  `try_move_player`'s existing flat-mode wall-slide boost needed no
  isometric-specific branch after all — a whole category of
  compensation code (`ISO_SLIDE_FACTOR`, the `step`/`is_isometric`
  parameters threaded through collision resolution) built while
  chasing the wrong approach was removed outright, not left dangling.
  Facing (`Direction::from_movement`) does not yet account for the
  table's diagonal/cardinal split — deferred until a second mover
  makes it a real, felt need.

### ADR-089: Per-scene, independently-authored orthographic and isometric camera modes
- **Context:** A diamond-shaped room (isometric) doesn't fit a fixed
  viewport the way a square room (orthographic) does, so a scene
  authored `Static` for flat mode may need `Follow` in isometric mode
  — but not every scene agrees (Outside wants `Follow` in both).
- **Decision:** `Scene` stores two authored `CameraMode`s
  (`orthographic_camera_mode`, `isometric_camera_mode`), resolved to
  one active mode via `Scene::resolve_camera_mode` — called
  identically at construction and on the F10 toggle
  (`sync_camera_mode`), so the two can never disagree.
- **Rationale:** Two rejected designs preceded this: mutating the
  active mode only on F10 toggle broke across scene transitions
  (a freshly-built scene had no way to know the current isometric
  state); unconditionally forcing `Follow` whenever isometric wrongly
  assumed every scene wants identical behavior. Storing both authored
  values and resolving fresh from a single function, callable from
  both triggers, closes both gaps at once.
- **Consequences:** `CameraMode` gained `Default` (`Follow`) for
  scenes that don't need to distinguish the two cases explicitly.

### ADR-090: Debug grid display size decoupled from real CELL_SIZE; DebugFlags renamed DebugSettings
- **Context:** Wanted to visually inspect the debug grid at different
  granularities without changing actual collision-grid behavior.
- **Decision:** `grid_display_cell_size`, tunable live via
  Numpad8/Numpad2 (8px steps, clamped [8,128]), passed as a parameter
  into `build_grid_lines_mesh` instead of reading the real
  `SpatialGrid`'s `cell_size`. `DebugFlags` renamed `DebugSettings`,
  since a tunable value no longer fits "flags" as an accurate name.
- **Rationale:** `build_occupied_cells_mesh`/
  `build_player_neighborhood_mesh` deliberately still read the real
  grid's `cell_size`, since those show actual grid contents; only the
  line overlay is a pure visual reference and safe to decouple.
- **Consequences:** Collision itself (`SpatialGrid::insert`/query,
  `CELL_SIZE`) is completely unaffected by this display setting.

### ADR-091: ProgressionTracker (game-scoped) and trigger-owned dialogue outcomes
- **Context:** The necklace needed to stay picked-up across a scene
  revisit — nothing survives scene reconstruction except what's
  explicitly re-consulted at build time (ADR-048) — and
  `consumes_entity` living per-`DialogueLine` (ADR-071/Phase 18)
  meant a multi-line conversation would need it padded onto every
  line just to reach the last one.
- **Decision:** `ProgressionTracker` (`game/progression.rs`) is a
  minimal in-memory `HashMap<&'static str, bool>` on `AppState`,
  surviving scene transitions. Deliberately placed in `game/`, not
  `engine/`, since only content code ever reads/writes it — mirroring
  `game/dialogue.rs`'s existing precedent for `engine`-imports-from-
  `game`. `TriggerKind::Dialogue` gains `sets_flag: Option<&'static
  str>` alongside `consumes_entity`, both now the trigger's
  responsibility, not the dialogue line's — the trigger is dialogue's
  actual entry and exit point. `Entity` gains `active: bool` and
  `deactivate()`, consolidating what `consume_entity` did by hand.
- **Rationale:** No persistence to disk — ADR-027 already defers
  serde/data-files until real content volume forces it; nothing in
  M5's actual need (one-time pickups, NPC dialogue conditions)
  requires surviving a process restart, only a scene transition
  within one.
- **Consequences:** Scene construction (`home::build`) checks
  `progression.is_set(...)` and calls `deactivate()` up front for
  anything already consumed. Flag-conditioned dialogue *content*
  (different lines depending on flags already set, not just whether a
  flag gets set) is explicitly out of scope — `line_for` still takes
  only an `id` — named as the next real step once NPC dialogue
  authoring begins.

### ADR-092: CELL_SIZE authored in content units; multiplying_factor stays a runtime value, not a const
- **Context:** CELL_SIZE was authored directly in world-space (post-
  scale) units, the only authored constant in the codebase not
  following the "small authoring unit, scaled once at construction"
  convention every wall/entity position already used.
- **Decision:** CELL_SIZE is now 12.0, in authoring units, multiplied
  by multiplying_factor once at each SpatialGrid construction site.
  multiplying_factor itself remains a plain runtime f32 (not a
  const), passed as a parameter, per the existing TODO's own
  eventual config-struct direction.
- **Rationale:** A real, named future need for multiplying_factor
  to be runtime/inspector-editable was identified — but the
  inspector doesn't exist yet, so a config struct would be designed
  against a guess. CELL_SIZE has no equivalent live need for
  runtime mutability; a well-named const is sufficient for a future
  engine-reuser to find and change in source.
- **Consequences:** grid_display_cell_size's tunable clamp range
  was rescaled to match the new authoring-unit space.

### ADR-093: Scene construction precedes content; entity/trigger authoring lives on Scene itself
- **Context:** The entity+trigger+prompt pattern (texture load,
  entity push, prompt entity push, trigger rect math) was repeated
  across the necklace, bed, and villager_1 — three real instances,
  the signal to extract per this project's standing discipline.
  Extracting it as Scene methods required Scene to exist before any
  content did, which Scene::new's old constructor-argument shape
  (background/walls/triggers/entities/player_index all passed in)
  didn't allow.
- **Decision:** Scene::new takes none of the four content
  collections — all start empty; player_index is a placeholder set
  only via the new spawn_player method. home::build/village::build
  construct an (initially empty) Scene up front and populate it
  directly via scene.background.push(...)/scene.walls.extend(...)
  (hand-authored, no dedicated method needed) and
  scene.spawn_entity(...)/spawn_player(...)/spawn_dialogue_trigger(...)
  (engine/scene/builder.rs) for the repeated pattern.
- **Rationale:** The helper is kept in engine/, not game/, since
  nothing in it is actually game-specific — every string/number
  arrives as a caller parameter, the same test that's kept
  TextureStore/SpatialGrid in engine/ throughout.
- **Consequences:** Adds TriggerId (mirroring WallId/EntityId) after
  a `.last_mut()`-based deactivation reference broke once (wrong
  trigger deactivated after a reordering) and an intermediate
  EntityId-reused-for-triggers attempt proved wrong on two counts
  (wrong Vec's length, wrong newtype for the collection).

### ADR-094: Trigger facing reinterpreted as the object's presented side; flush-contact vs. vicinity interact checks (supersedes part of ADR-060)
- **Context:** A single, any-side-approachable trigger (villager_1)
  exposed that required_facing.contains(&player.facing) alone can't
  tell "correctly positioned, facing toward" from "facing the same
  absolute direction from the wrong side" — the exact problem
  ADR-060 solved by requiring one trigger per approach side.
- **Decision:** The trigger's facing field now means "which side(s)
  the object presents," inverted via Entity::match_facing_direction
  against the player's actual facing. is_facing_toward checks both
  that the target is on the correct side (delta sign) and that the
  player falls within the target's extent on the perpendicular axis
  — the second check fixes a real bug found after the first,
  center-only version shipped (a player flush against one side of a
  wide target could "face toward" it while looking perpendicular to
  it). try_interact's two trigger kinds turned out to need different
  proximity tests: Dialogue requires true flush contact
  (player_flush_with, aabb_overlap against the player's real
  collider); Toggle keeps a vicinity check (player_near,
  point_in_rect), since a toggled object's collider can vanish (an
  open door has none) leaving nothing to be flush against.
- **Rationale:** One trigger per any-side-approachable object,
  instead of one per side, and a correctness fix a purely
  center-based check couldn't provide.
- **Consequences:** Every pre-existing single-approach trigger
  (bed, necklace, both patio-door toggles, later merged into one)
  had its facing value inverted to the new meaning — not just
  renamed — verified by hand and in play. DialogueTriggerSpec's
  per-NPC trigger_padding replaced with a flat, tight +1.0-unit
  margin. Adds point_in_range, a shared bounds-check extracted once
  point_in_rect and is_facing_toward were found computing the same
  per-axis logic independently.

### ADR-095: Render passes split by purpose; is_overlay_layer for occlusion-proof sprites; no depth buffer
- **Context:** render_scene's entity loop and the former draw_hud
  (notifications, dialogue, debug info, and the volume slider's
  input handling, all bundled) needed disentangling before a
  purpose-grouped, file-per-layer render structure was achievable. A
  real bug (villager_1's prompt icon occluded by the player's sprite
  when approached from above) needed a layering fix, not a
  collision/proximity fix.
- **Decision:** draw_background_and_entities extracted as its own
  method; draw_hud split into update_debug_ui (input handling,
  called once per frame, not from a draw path), draw_ui (dialogue,
  the seed of a future real HUD), and draw_debug_info (notifications,
  FPS, mouse position, slider draw). Entity gains is_overlay_layer:
  bool — true entities draw in a dedicated pass after every normal
  entity, always on top regardless of y-sort. A depth buffer was
  considered and rejected: nothing in the game needs per-pixel depth
  resolution within a tier, only draw-order control between a
  handful of tiers, which purpose-grouped passes already provide
  without the real new GPU surface (depth texture, pipeline
  depth-stencil state) a depth buffer requires.
- **Rationale:** Both new draw passes share submit_sprite_draw, a
  helper extracted from one large per-draw block, parameterized by
  clear-vs-load — built as a deliberate copy-paste first, verified
  working, then deduplicated, this project's standard approach to
  nontrivial refactors.
- **Consequences:** engine/debug/hud.rs renamed info.rs (frees "hud"
  for actual future player-facing UI); draw_slider relocated onto
  Slider::draw, the pattern future inspector widgets should follow.
  update_interact_prompts also switched to aabb_overlap (from
  point_in_rect), the same flush-vs-point fix as ADR-094, applied to
  prompt visibility rather than interaction. Relocating the five
  layers into dedicated renderer/layers/ files, and batching
  multiple differently-textured sprites into fewer draw calls
  (needs a texture atlas or equivalent), remain open, named
  follow-ups.

### ADR-096: Tile-based scene backgrounds (supersedes ADR-028's remaining hand-authored-background clause)
- **Context:** ADR-028 committed to one hand-painted background image per
  scene, reasoned independently of camera projection (a content-authoring-
  scale argument, not an isometric-vs-flat one). ADR-087 later corrected
  ADR-028's separate projection-math claim but explicitly left this clause
  standing. Revisited once real isometric pixel art research made a small
  reusable tile set a well-understood option, which it wasn't when ADR-028
  was written (self-assessed inexperience with isometric art at the time).
- **Decision:** Scene backgrounds are assembled from a small set of
  reusable tile pieces (~10–12: floor, edge, corner variants) placed on a
  grid, not one large image per scene. Collision and entity placement are
  entirely unaffected: walls remain hand-placed `Rect`s in world space
  (ADR-038), furniture/NPCs remain individually hand-authored and hand-
  placed, none of it tile-snapped. The tile-ID-to-texture palette is a
  small `const` array in source; only the per-scene layout externalizes
  (see ADR-097). Tile pixel dimensions and the isometric shear constant
  `K` (ADR-087) are authored together and tuned by eye, per the precedent
  ADR-087/ADR-092 already established for `K` and `CELL_SIZE`.
- **Rationale:** Meets Principle 5's stated exception (charter 2.3): the
  correct future shape is now known and cheap to build, where it wasn't
  at ADR-028's time. A correctly-shaded isometric floor costs materially
  more art labor per scene than a flat one would have, making tile reuse
  pay for itself sooner than ADR-028's original content-volume argument
  assumed. Renderer benefit alongside the art one: background stops being
  a special-cased large texture and becomes more draws through the same
  textured-quad path already used for furniture.
- **Consequences:** Requires an authoring tool to be usable in practice
  (ADR-097) — the format decision alone doesn't solve the "don't want to
  hand-type indices" problem this was raised to fix. `DERIVATION.md`'s
  D-D entry is now stale on this point and is left unedited, per this
  project's standing convention that DERIVATION is a dated, closed
  artifact — the ADR chain (ADR-028 → ADR-087 → ADR-096) is authoritative.

### ADR-097: Minimal file-based persistence for tile grids only (narrow carve-out from ADR-027)
- **Context:** ADR-096 requires layouts to live somewhere editable
  without recompiling. A copy-paste-to-source workflow (paint live in an
  in-memory tool, print a `const` array, hand-transcribe into
  `game/scenes.rs`) was considered first, specifically to avoid touching
  ADR-027's deferral of file-based content. Rejected: a hand-rolled parser
  for this one fixed, trivial shape is less code than the transcription
  step it was trying to avoid, and doesn't implicate the general-purpose
  problem ADR-027 was actually deferring.
- **Decision:** Tile grids are saved as plain text (whitespace-separated
  small integers, one row per line — no schema, no serde, no RON) and
  read once by the owning scene's constructor, at the same moment it
  already loads its textures — the existing lazy-construction, asset-
  loading pattern (E10) applied to one more asset kind, not a new loading
  phase. No file present ⇒ nothing painted ⇒ the scene's void shows
  through, following directly from the existing no-camera-clamping/void-
  is-the-look decision rather than needing a special-cased empty state.
  Malformed files (wrong row length, out-of-range tile ID) panic loudly,
  naming the file and the specific problem, rather than defaulting
  silently — appropriate for a solo-dev tool where the developer is both
  the only user and the one debugging failures.
- **Rationale:** ADR-027 deferred a *generic* serialization solution for
  schemas that are complex, varied, or still evolving (NPCs, dialogue,
  enemy stats). A tile grid is the opposite: one fixed, fully-known shape
  that will never need to grow more complex. Hand-rolling a parser for it
  is the correct application of the judgment ADR-027 asked for later,
  applied now to a case that actually qualifies — not a reopening of the
  broader question. Every other content category stays static Rust data,
  unchanged.
- **Consequences:** This is the producer/consumer half of ADR-096's
  authoring tool; the tool itself (toggle paint mode, click to place from
  a keyboard-selected tile brush, live render, save-to-file) is scoped
  but not yet built. Named hard dependency, not yet written: screen-to-
  world mouse picking, the inverse of ADR-087's projection transform — a
  different consumer than ADR-088 (which deliberately rejected the
  shear's literal inverse for movement, for control-feel reasons; no
  conflict, since picking has no equivalent feel constraint). Save-file
  naming/location convention not yet decided. Undo, multi-layer, and
  drag-paint explicitly deferred past a first version. Furniture-via-
  click-placement acknowledged as a natural extension, not designed.

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
- ✅ **M4 (The Voice):** closed at Phase 21. Scene transitions (three
  trigger kinds — Warp, Dialogue, Toggle), lazy scene reload, camera
  modes, batched text and debug-rect rendering, full dialogue
  machinery (typewriter, advance/skip, two registers, per-span color,
  word-wrap), blip audio, and Beat 2's real bed/necklace content —
  all built, verified, and confirmed against real content rather than
  placeholders. Grid-based spatial partitioning for collision remains
  parked, revisit at M5 content scale.
- ✅ **Structural refactor & reorganization (Phase 23):** the pass
  deferred at M4's close is done — renderer split by concern
  (mesh/gpu/draw), scene content extracted to `game/scenes`, ADR-035's
  input abstraction resolved, `RedrawRequested` broken up, `ids.rs`
  eliminated in favor of ownership-based type placement, engine files
  reorganized by owning module, `Scene`'s impl split out, `platform.rs`
  renamed to `app.rs`, debug HUD drawing extracted, and import
  ordering made consistent crate-wide (ADR-077–083). No gameplay
  behavior changed. **M5 (The Village) starts next.**
- ✅ **M5 opening (Phases 24–28):** spatial partitioning (static +
  dynamic grid, collision broad-phase), full isometric projection
  foundation (render-time-only shear, point-shear/shape-shear split,
  the 8-direction hand-authored movement table, per-scene camera
  modes), debug tooling (tunable grid display, DebugSettings), and
  ProgressionTracker with trigger-owned dialogue outcomes — all
  built, verified, and committed. **Still open:** facing during
  isometric movement (deferred to the first NPC), flag-conditioned
  dialogue *content* (`line_for` doesn't yet read
  `ProgressionTracker`), collider/FPS/mouse-position debug views not
  yet nested under the same master switch as grid visualization.
  **Next:** `outside.rs` renamed to reflect real village content;
  flag-aware dialogue; ambient NPCs, clue chain, guard/sword per
  DERIVATION's M5 scope.
- ✅ **Village content foundation & render-pipeline layering
  (Phases 29-31):** CELL_SIZE authoring-unit fix, village rename,
  flag-aware dialogue, villager_1 as the first NPC; facing-direction
  rework (edge-aware is_facing_toward, flush-vs-vicinity interact
  checks, TriggerId); Scene-construction restructure (spawn_entity/
  spawn_player/spawn_dialogue_trigger as real Scene methods);
  render_scene/draw_hud split into purpose-grouped draw methods;
  is_overlay_layer fixing prompt-icon occlusion. **Still open:**
  facing during isometric movement (deferred to a moving NPC),
  relocating the five draw layers into renderer/layers/ files,
  batching multiple differently-textured sprites into fewer draw
  calls, nesting collider/FPS/mouse-position debug views under the
  same master switch as grid visualization. **Next:** the layering
  file-relocation, then remaining M5 content — the flag-conditioned
  clue chain, additional ambient NPCs, the guard/sword moment.
- ✅ **Render layer relocation & AppState reorganization (Phase 32):**
  render_scene removed, all 5 draw layers now live in their own
  files (engine/renderer/layers/, engine/app/layers/), called
  directly and in sequence from RedrawRequested. AppState's impl
  block split by behavior (player.rs, scene_lifecycle.rs, dialogue.rs
  now also holding dialogue-audio methods); tick_frame_timing
  extracted. **Still open:** facing during isometric movement
  (deferred to a moving NPC — none exist yet), nesting collider/FPS/
  mouse-position debug views under grid visualization's master
  switch, batching multiple differently-textured sprites into fewer
  draw calls (deferred until draw count is a measured problem).
  **Next:** actual M5 content authoring — the flag-conditioned clue
  chain (villager → vendor → child), remaining ambient NPCs, the
  guard/sword moment. The necklace/bed remain hand-authored, not yet
  converted to spawn_entity/spawn_dialogue_trigger — a small,
  optional cleanup, not blocking content work.
- ✅ **Tile-based scene authoring — design session (Phase 33):** decided,
  not yet implemented. Backgrounds move from one hand-painted image per
  scene to a small reusable tile set (ADR-096); layouts persist via a
  minimal hand-rolled text format, read once at scene construction
  (ADR-097) — a deliberate, narrow exception to ADR-027, not a reopening
  of it. **Next:** implement the file format, then the interactive
  painter (paint mode, click-to-place, save) — the painter needs screen-
  to-world mouse picking, which doesn't exist yet and is the likely first
  real subtask.

### Open questions

1. **Tuning knobs (playtest, not debate):** grid width, tick length,
   lockout duration, telegraph visuals — unchanged, still Phase 1+.
2. **Writing pending:** unchanged from v3.
3. **Narrator identity:** parked, unchanged.
4. **ECS / serde re-evaluation:** unchanged, Phase 1.
5. **Input abstraction (ADR-035):** resolved in Phase 23 — see
   ADR-079. `InputState` (engine) vs. `Action` (game), dialogue
   advance/skip was the second consumer.
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

### Parked — Grid-based spatial partitioning for collision checks

Raised while extending try_move_player's flush-collision resolution — collider_blocked currently checks every wall and every entity's collider unconditionally, twice per frame per moving entity (once per axis), regardless of distance from the mover. Fine at current content scale (a dozen-ish colliders per scene), but scales linearly with total collider count, not with how many are actually nearby.

A coarse-grid spatial partition (informed by a similar approach in a colleague's bullet-hell project) would divide a scene into fixed-size cells, further split into quadrants around the moving entity's current cell, and only check colliders in the mover's cell plus the 3 adjacent cells toward whichever quadrant the entity sits in — reducing most checks to a small, roughly-constant neighborhood rather than the whole scene. Not built now — no current scene is close to large enough for this to matter, and building it against today's small, hand-placed content risks guessing at cell size and boundary handling without real data to tune against. Revisit once a scene's collider count is large enough to make the cost concretely measurable (M5's populated village is the likely trigger).

### Parked — Combat design: elemental/environmental battle effects & cult power hierarchy

Sketched alongside ADR-075's lineage work, not mechanically designed.
Not an ADR.

- **Elemental/environmental effects are hierarchy-gated, not universal.**
  Rank-and-file initiates never use them (consistent with ADR-016 —
  they're mundane by design). Named cult leadership ("executives," to
  borrow the framing this was pitched in) can invoke a single
  environmental effect, and only with prior preparation time — not a
  standing ability, a rare and costly working. Full-scale weather
  events (the storm that enabled the kidnapping) are reserved for
  cult leadership acting with significant lead time, not something
  available casually — this keeps ADR-016's escalation-by-reveal
  design intact rather than undercutting it with an off-screen show
  of force before the game even starts.
- **Escalation mirrors ADR-016's existing tier logic, one level up:**
  early leadership fights introduce one mild environmental modifier
  (e.g. wind nudging the player toward the front row, aggravating the
  existing back-row-camping tension ADR-014 already names); later
  fights intensify the same category rather than introducing unrelated
  ones (wind becomes a visibility-obscuring sandstorm, etc.).
- **Hard constraint, not optional:** whatever obscures vision must
  never obscure telegraph tiles themselves (ADR-015's fairness
  contract is non-negotiable) — the environment gets worse, the
  warning stays legible, possibly via a redundant audio cue under
  heavy-effect conditions. Telegraph windows may legitimately shrink
  as a difficulty lever (paired with the loss of an enemy's own visible
  wind-up animation once "whose turn is it" is no longer readable at a
  glance), but must never disappear.
- **Explicitly out of slice, out of Phase 1.** This is new engine
  surface (per-battle environmental state, movement/visibility
  modifiers) with no beat currently demanding it. Revisit only once
  boss/executive-tier encounters are actually being designed.

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
