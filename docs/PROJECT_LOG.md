# Project Log — Rust Game & Engine Project
### Working Title: *"The Morning She Was Gone"* (game) / Engine TBD

**Document type:** Living project record (docs-as-code)
**Started:** July 2026
**Status:** Concept phase complete → Vertical slice design in progress
**Maintenance model:** Updated at the end of each working session where decisions are made. Lives in `docs/` in the project repository, versioned with git.

---

## 1. How to Read This Document

This log has four layers:

1. **Project Charter** — the stable "constitution" of the project. Changes rarely.
2. **Phase Log** — chronological record of what was worked on and concluded, phase by phase.
3. **Decision Log (ADRs)** — every significant decision, with context, rationale, and consequences. This is the most valuable section long-term.
4. **Current State & Open Questions** — snapshot of where the project stands right now.

When updating: append to the Phase Log, add new ADRs (never delete or rewrite old ones — supersede them with a new ADR that references the old one), and rewrite section 5 to reflect current reality.

---

## 2. Project Charter

### 2.1 Mission

Build a playable game in Rust, and extract a reusable, eventually open-source game engine from the process — in that order of priority.

### 2.2 The Governing Principle

> "I am building a game in Rust, and extracting a reusable game engine from the process."
> — NOT: "I am building a general-purpose game engine, and eventually I will make a game."

Every engine feature must exist because the game demanded it. The flow is:

```
Game needs feature → Implement → Use in game → Learn from real usage
→ Refactor/generalize where appropriate → Promote reusable parts into engine
```

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

### 2.4 Developer Profile (context for all learning decisions)

- 6+ years professional software development; M.Sc. Informatics (TU Munich).
- Strong: TypeScript/Angular, C#/.NET, full-stack development, general architecture.
- Beginner: Rust, game engine architecture, graphics programming.
- Implication: explanations skip general programming concepts, but never assume Rust idioms or graphics knowledge.

### 2.5 Candidate Technology Stack (not committed)

| Concern | Candidate |
|---|---|
| Language / build | Rust + Cargo |
| Windowing | winit |
| Graphics | wgpu |
| Math | glam |
| ECS | Bevy ECS or hecs (evaluation pending) |
| Audio | kira |
| Physics | Rapier (may be unnecessary for grid combat) |
| Serialization | serde |

Initial foundation candidate: **Rust + Cargo + winit + wgpu + glam**. Dependencies are added when the project needs them, not before.

---

## 3. Phase Log

### Phase 0 — Project Framing (completed)

**Input:** Initial project context document.

**Work done:** Established the project's identity, constraints, feasibility assessment, and the ten-step blueprint (concept discovery → scope → engine scope → tech evaluation → architecture → Rust roadmap → dev roadmap → open-source strategy → risk analysis → 30-day plan).

**Key conclusions:**
- Project is feasible if scope is controlled.
- 2D first; 3D only if the project naturally evolves there.
- The engine emerges from the game, never the reverse.
- First major task: game concept discovery.

### Phase 1 — Game Concept Discovery: Systems Identity (completed)

**Work done:** Explored what kind of game this wants to be, starting from mechanical/visual instincts rather than story.

**Key conclusions:**
- **Visual identity:** 2D isometric overworld designed to feel like a 3D world.
- **Exploration:** scene-based (exteriors, interiors, transitions between small interconnected areas).
- **Narrative interaction:** NPC dialogue with typewriter text effect and character-specific text blips.
- **Combat:** on encounter, the game transitions to a dedicated top-down battle scene — real-time grid-based combat, inspired by Mega Man Battle Network's *design philosophy* (relationship between overworld and combat), not a mechanical copy.
- **Structural loop:** Home → explore → talk → discover goal → travel → encounter → grid battle → continue → reach goal.

### Phase 2 — Game Concept Discovery: Premise (completed)

**Work done:** Generated and evaluated five candidate premises, all designed to support the established loop: The Last Delivery, The Village Festival, The Missing Person, The Pilgrimage, The Strange Morning. Each was rated on scope, exploration, dialogue, combat, and atmosphere.

**Key conclusions:**
- All five premises were viable; The Pilgrimage was the initial structural recommendation.
- Final selection (developer's synthesis): **The Missing Person as the frame, with The Last Delivery's "not who they seem" twist inverted** — the missing sister is not secretly worse than believed, but secretly *better*: she made hidden, unregretted sacrifices for the protagonist's benefit.

### Phase 3 — Story & Theme Synthesis (completed)

**Work done:** Developed the full narrative arc and discovered the game's thesis mechanic.

**The story (summary):**
Protagonist wakes; sister is missing. Investigation loop through villagers produces clues → next town → she was involved with unknown people → they resist snooping → confrontations escalate → the group is a cult → the cult intends to sacrifice her → the cult's magic is **dimensional flattening**: sacrificing the z-axis for power (2D beings need not contend with a third dimension). Once the ritual is understood, a countdown begins; tone shifts from investigation to escalating non-stop combat. At the altar: the ritual requires a sacrifice who has made unregretted, *unknown* sacrifices for a loved one, expecting nothing. **By explaining the ritual, the cult reveals the sister's hidden sacrifices to the protagonist — breaking the ritual's own precondition.** The villains defeat themselves by monologuing. The bond deepens; the game ends.

**Key conclusions:**
- **The z-axis flattening is the game's thesis, not a joke.** It makes the isometric→flat-grid combat transition *diegetic*: entering battle means being pulled into the cult's magic. Visual identity, combat system, and story climax are one idea (high economy for a solo dev).
- Progression possibility: early battles feel *done to* the player (world folds against your will); late game, the player fights comfortably in the flattened dimension or triggers it themselves.
- Held-in-reserve idea: by journey's end, the *protagonist* may qualify as a ritual candidate (unknown sacrifices for the sister) — potential final-act stakes. Not committed.
- Setting: pre-modern / medieval-ish fantasy. Rationale: walking pace makes village-to-village structure earnest; excludes GPS/vehicles that would trivialize travel; fits Game Boy-era ambition.
- Emotional presentation: still frames + music during revelations (high emotional payoff, low production cost).

**Scope warnings issued and accepted:**
- Inventory "analysis" system flagged as a hidden subsystem (UI + interactions + content). Deferred; clues via dialogue and found notes for v1.
- Combat escalation via "more HP + more goons" flagged as content-expensive/repetitive. Budget: ~4–6 enemy behavior patterns total; escalate by *combining* behaviors in the arena.
- Scene count to be fixed early and defended (~6 exteriors + interiors); "one more village" identified as a primary scope trap.
- No party members — confirmed. Solo protagonist vs. multiple enemies, consistent with MMBN lineage and dramatically simpler.

### Phase 4 — Methodology: Design-First Development (completed)

**Work done:** Established the development methodology connecting design to Rust learning.

**Key conclusions:**
- **Design precedes code so that code has something to serve.** The storyboarded vertical slice becomes a requirements document; requirements convert Rust learning from textbook mode into problem→solution mode.
- Exception: a **minimal Rust baseline** (~1–2 weeks, deliberately shallow) is required before touching winit/wgpu, or compiler errors will be unreadable. Scope: ownership, borrowing, structs, enums, match, traits — at "I know these exist and roughly why" depth. Tools: Rustlings and/or fast first-half pass of the Rust Book. Everything deeper is learned on demand.
- The pipeline:

```
Storyboard the slice
→ Derive feature list ("what must exist to be playable")
→ Derive engine requirements ("what infrastructure those features need")
→ Map to Rust concepts ("what I'll be forced to learn, in what order")
→ Minimal Rust baseline (1–2 weeks, shallow)
→ Build milestone 1, learn what it demands
→ Repeat
```

### Phase 5 — Vertical Slice Storyboard v1 (in progress — awaiting reaction pass)

**Work done:** Drafted the 7-beat vertical slice "The Morning She Was Gone" (~10 min of gameplay), each beat annotated with the engine systems it demands.

**The beats (v1 draft):**

| # | Beat | Story content | Systems demanded |
|---|---|---|---|
| 1 | Wake up | Bedroom; walk; examine bed + sister's keepsake (emotional seed) | Window, game loop, isometric interior rendering, player movement, collision, interact verb |
| 2 | The empty house | Her room untouched; first internal monologue | Room→room scene transition, typewriter dialogue, audio blips, trigger system |
| 3 | The village, three voices | 3 NPCs, 3 clues, distinct blip-voices; child's clue gated on talking to the other two | Exterior scene, NPC entities, interaction radius, per-character dialogue audio, dialogue state/flags |
| 4 | Leaving | North exit; neighbor's soft gate + practice item (narrative only, no inventory UI) | Scene exit conditions, flag check |
| 5 | The road, and the fold | Hooded figure; two unsettling lines; **the world flattens** — the identity moment | Encounter trigger, battle scene transition, flatten effect (v1 = simple wipe acceptable) |
| 6 | First battle | Small grid (~6×4); one enemy, one telegraphed pattern; real-time; lose = retry, no death spiral | Grid logic, real-time grid input/movement, enemy behavior state machine, attack/hitbox/timing, health, win/lose |
| 7 | The clue, and the hook | World un-flattens; figure gone; chalk symbol scrap matches shopkeep's clue; "she's headed somewhere"; end | Return transition, closing dialogue, end screen (deliberately no new systems) |

**Deliberate slice exclusions:** inventory UI, multiple simultaneous enemies, abilities, second town, cult content, save system.

**Status:** Awaiting developer's reaction pass (beats, tone, gating, clue design).

---

## 4. Decision Log (ADR Format)

> Format: each ADR records Context (the situation), Decision (what was chosen), Rationale (why), and Consequences (what this commits us to, including downsides). ADRs are immutable — a changed decision gets a *new* ADR that supersedes the old one.

### ADR-001: Rust as the project language
- **Context:** Developer wants to learn systems programming; evaluating languages for a game+engine project.
- **Decision:** Rust, exclusively.
- **Rationale:** Learning goal in itself; performance, memory safety, ownership model, strong types, zero-cost abstractions suit engine work; rich ecosystem (winit/wgpu/etc.).
- **Consequences:** Steeper early learning curve; compiler fights expected and treated as the learning mechanism. No engine framework shortcuts (see ADR-003).

### ADR-002: Game-first, engine-extracted
- **Context:** Risk of the classic failure mode — endless engine-building, no game.
- **Decision:** The game is the primary product; the engine is extracted from proven, game-driven code.
- **Rationale:** Concrete requirements prevent speculative architecture; finishing something playable is a core goal.
- **Consequences:** Early engine code may be game-stained and need later refactoring for reuse. Accepted cost.

### ADR-003: Build on low-level libraries, not a full framework (e.g., not Bevy-the-framework)
- **Context:** Bevy would ship a game faster but would do the engine-architecture learning *for* us.
- **Decision:** Compose low-level crates (winit, wgpu, glam, etc.); study frameworks like Bevy for architectural inspiration only. (Bevy ECS as a *library* remains under evaluation — see Open Questions.)
- **Rationale:** Understanding engine internals is a primary educational goal of the project.
- **Consequences:** Slower initial progress; more surface area to learn; greater ownership of the result.

### ADR-004: 2D first
- **Context:** 3D is more impressive but multiplies scope (math, assets, rendering complexity).
- **Decision:** First game is 2D. Architecture should avoid *unnecessarily* blocking future 3D, but not at the cost of present simplicity.
- **Rationale:** Scope control; 2D already covers every fundamental (loop, rendering, input, ECS, assets, audio, scenes).
- **Consequences:** Portfolio impact relies on distinctiveness rather than 3D flash — mitigated by ADR-008.

### ADR-005: Premise — Missing Person frame with inverted "not who they seem" twist
- **Context:** Five candidate premises evaluated (Phase 2).
- **Decision:** Sister is missing; protagonist investigates. Twist inversion: she is revealed as secretly *more* loving/sacrificing than known, not secretly villainous.
- **Rationale:** Strongest player motivation ("you'd drop everything for this"); gives dialogue/exploration systems real purpose (investigation); the inversion lands the emotional theme (hidden sacrifice, deepened bond).
- **Consequences:** Dialogue and clue systems become load-bearing; their quality directly carries the game.

### ADR-006: The z-axis flattening as diegetic thesis mechanic
- **Context:** Needed a justification for the isometric→flat-grid combat transition; developer proposed cult magic that sacrifices the z-axis for power (initially framed as a meta joke).
- **Decision:** Canonize it as the game's thesis. Entering combat = being pulled into the cult's flattened dimension. The transition is plot, not UI convention.
- **Rationale:** One idea unifies visual identity, combat system, and story climax — maximum economy for a solo developer. Also enables a progression arc told through the camera (early: flattening is done *to* you; late: you master or invoke it).
- **Consequences:** The flatten transition effect becomes a first-class feature that must exist even in the vertical slice (crude v1 acceptable). The ritual's internal logic must stay consistent.

### ADR-007: The cult's exposition breaks its own ritual
- **Context:** The ritual requires a sacrifice whose own sacrifices are *unknown* to their beneficiary; the altar scene requires the cult to explain this.
- **Decision:** The explanation itself disqualifies the sister as a sacrifice — the reveal is the resolution.
- **Rationale:** Elegant, thematically resonant (knowledge of love defeats exploitation of love), and rhymes with ADR-006's genre-aware wit.
- **Consequences:** The altar scene's writing carries the entire climax; must be crafted carefully. Reserve idea (uncommitted): the protagonist may qualify as a replacement candidate — potential final-battle stakes.

### ADR-008: Pre-modern / medieval-ish fantasy setting
- **Context:** Needed a setting where village-to-village travel on foot is natural.
- **Decision:** Old-world, non-modern setting; no GPS, vehicles, or instant communication.
- **Rationale:** Walking pace makes the journey structure earnest; removes "why not take a flight?" plot-hole criticism; suits Game Boy-era tone and still-frame emotional presentation.
- **Consequences:** Art direction and audio must carry "olden village" atmosphere; anachronisms must be policed in writing.

### ADR-009: No party members
- **Context:** Party systems multiply AI, UI, balance, and narrative complexity.
- **Decision:** Solo protagonist, potentially versus multiple enemies.
- **Rationale:** Dramatic simplification; true to MMBN lineage (lone fighter vs. groups).
- **Consequences:** Combat depth must come from movement, positioning, timing, and enemy behavior combinations rather than team composition.

### ADR-010: Combat escalation via behavior combination, not stat inflation
- **Context:** "More HP + more goons" escalation flagged as content-expensive and repetitive.
- **Decision:** Budget of roughly 4–6 enemy behavior patterns for the full game; difficulty escalates by combining behaviors in the arena.
- **Rationale:** Two simple patterns intersecting is a new puzzle at near-zero content cost; matches MMBN's pattern-based design.
- **Consequences:** Enemy behaviors must be designed as composable from the start.

### ADR-011: Inventory analysis system deferred from v1
- **Context:** "Analyze dropped items in inventory" sounds small but implies UI, interactions, and per-item content.
- **Decision:** Not in the vertical slice, not in v1. Clues delivered via dialogue and found notes/still frames.
- **Rationale:** Nothing in the core loop breaks without it; classic hidden-subsystem scope trap.
- **Consequences:** If later added, may be promoted as engine infrastructure (inventory) + game content (items) separately. A future ADR would cover it.

### ADR-012: Design-first pipeline with minimal Rust baseline
- **Context:** Choosing between "learn Rust from a book first" and "learn purely on demand."
- **Decision:** Storyboard → feature list → engine requirements → Rust-concept map → 1–2 week shallow Rust baseline (ownership, borrowing, structs, enums, match, traits) → build and learn on demand.
- **Rationale:** Problems make learning stick; but zero baseline makes compiler errors unreadable. Baseline = vocabulary, not mastery.
- **Consequences:** The vertical slice storyboard is a blocking dependency for the coding phase — by design.

### ADR-013: Documentation as docs-as-code with ADRs
- **Context:** Developer wants a start-to-end project record, maintained during (not after) the project; assistant cannot persist work between sessions.
- **Decision:** This document lives in the project repo (`docs/`), versioned with git. Updated at the end of any session with real decisions ("update the project log"). Decisions recorded as immutable ADRs; changed decisions get superseding ADRs.
- **Rationale:** Git guarantees continuity; ADRs answer "why did we do it this way?" — the question post-hoc documentation always fails to answer.
- **Consequences:** Developer owns the update ritual; the assistant produces deltas or full revisions on request.

---

## 5. Current State & Open Questions

### Where the project stands (July 2026)

- ✅ Project charter, principles, feasibility — settled.
- ✅ Game systems identity — settled.
- ✅ Premise, story arc, thesis mechanic, setting — settled.
- ✅ Development methodology — settled.
- 🔄 Vertical slice storyboard v1 — drafted, **awaiting reaction pass**.
- ⬜ Feature list & engine requirements derivation — blocked on storyboard.
- ⬜ Rust concept map & 30-day plan — blocked on derivation.
- ⬜ Any code — intentionally not started.

### Open questions (to resolve, roughly in order)

1. **Grid combat model:** MMBN-style split grid (player owns one half) vs. free movement on a shared grid. Shapes combat code fundamentally; decide during storyboard reaction or slice design.
2. **Storyboard reaction pass:** beats, tone, the soft gate in Beat 4, clue design in Beat 7.
3. **ECS evaluation:** Bevy ECS as a library vs. hecs vs. simpler game-object model for the slice — decide when Beat-level requirements are derived, not before.
4. **Flatten-effect v1 fidelity:** minimum acceptable version of the identity transition for the slice.
5. **Engine name:** unclaimed; no urgency.
6. **Open-source license:** deferred until the engine crate boundary exists.

### Next session agenda

1. Storyboard reaction pass (Beat-by-beat).
2. Decide the grid combat model (Open Question 1).
3. Begin derivation: beats → feature list → engine systems → Rust concepts.

---
