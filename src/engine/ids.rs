//! Small, Copy identifier newtypes shared across modules that would
//! otherwise need to depend on each other to define them. Kept
//! deliberately minimal — this exists because Entity and Scene both
//! need these types and neither should own the other's concept
//! (SceneId/WarpId are needed by Trigger, which lives in entity.rs,
//! but conceptually belong to scene identity, not entity identity).

/// Identifies a scene by name, independent of whether that scene is
/// currently loaded. Used by warps (Trigger::Warp) to name a
/// destination scene that may not exist in memory yet (ADR-048: scenes
/// are lazily constructed on entry, not kept resident).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneId {
    Home,
    Outside,
}

/// Identifies a single warp point within one scene, and doubles as a
/// human-readable debug label — both jobs from one authored value, no
/// separate name registry to keep in sync (ADR-051). Uniqueness only
/// needs to hold within one scene, since a warp is always addressed
/// as (SceneId, WarpId) together.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WarpId(pub &'static str);

/// Identifies a texture within a scene-scoped TextureStore (ADR-036).
/// Simple newtype wrapper around a usize index, made distinct to avoid accidental mixup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureId(pub usize);
