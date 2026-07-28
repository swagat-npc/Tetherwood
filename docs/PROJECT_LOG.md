# Project Log — Tetherwood
### Game: *Tetherwood* · Act I / vertical slice: *"The Morning She Was Gone"* · Engine: unnamed (deliberate)

**Document type:** Living project record (docs-as-code)
**Started:** July 2026
**Revision:** v3
**Status:** Design phase complete → Toolchain setup & M0/M1 next. Companion document: `docs/DERIVATION.md` (feature inventory, engine split, Rust map, milestones, 45-day plan).
**Maintenance model:** Single canonical file at `docs/PROJECT_LOG.md`, versioned with git. Updated when decisions accumulate, not on a timer.

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
- **Consequences:** Developer owns the update ritual. **Versioning is git commits on one canonical file** — filename suffixes (_v3) are a pre-repo stopgap only. Log revs when decisions accumulate, not on a timer.

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

---

## 5. Current State & Open Questions

### Where the project stands (July 2026, v3)

- ✅ Charter, principles, feasibility.
- ✅ Systems identity; premise; story arc; thesis mechanic; setting.
- ✅ Combat model; progression; first act (beats v2); voice/context/texture/UI staging.
- ✅ **Derivation pass** (`docs/DERIVATION.md`): features, engine split E1–E11, Rust concept map, milestones M0–M7, 45-day slice plan.
- ✅ **Naming:** Tetherwood / "The Morning She Was Gone" (Act I).
- ✅ Design phase **closed**. All gates to code are open.
- ⬜ Toolchain: rustup + MSVC build tools + rust-analyzer (Windows). ← next
- ⬜ Repo init + docs commits (Log v1 → v2 → v3 → DERIVATION) + `cargo new tetherwood` with module skeleton (days 1–2).
- ⬜ M0 ∥ M1 (days 3–7): Rust baseline alongside the first window.
- ⬜ Code written so far: zero. This is the last revision for which that is true.

### Open questions

1. **Tuning knobs (playtest, not debate):** grid width (~4), tick length (~0.5s), lockout duration, telegraph visual treatment.
2. **Writing pending:** initiate pre-fight line (direction settled); clue-chain dialogue (vendor, shopkeeper, child) + flag names; meta-NPC joke lines; narrator T2 winks; flatten-effect v1 fidelity.
3. **Narrator identity:** parked; revisit no earlier than Phase 2 writing.
4. **ECS re-evaluation:** Phase 1, with slice experience in hand (per ADR-025).
5. **serde/RON adoption:** Phase 1 thickening, when content volume hurts (per ADR-027).
6. **Engine name; open-source license:** deferred until the engine crate boundary exists.

### Next session agenda (Milestone Chat: The Toolchain / M1)

1. rustup on Windows (accept the MSVC build tools prompt), VS Code + rust-analyzer.
2. Repo init; commit sequence: `chore(repo): initialize repository with docs structure` → the three log commits → derivation commit → `chore(repo): initialize rust project with module skeleton`.
3. M1 begins: winit window titled **Tetherwood**, clear color, ESC quits, WASD logs. First real Rust fight expected: the event-loop closure (`move`, `'static`).
4. Log v4: not scheduled. It arrives when code forces decisions worth recording — likely mid-M1 or at the M2 wall.

---
