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
    Bedroom,
    Hallway,
}

/// Identifies a single warp point within one scene. Only needs to be
/// unique within that scene — not globally — since a warp is always
/// addressed as (SceneId, WarpId) together. Newtype over u32, matching
/// TextureId's pattern (engine/texture.rs): prevents an arbitrary
/// number from being passed where a warp identity is expected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WarpId(pub u32);

/// Identifies a texture within a scene-scoped TextureStore (ADR-036).
/// Simple newtype wrapper around a usize index, made distinct to avoid accidental mixup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureId(pub usize);
