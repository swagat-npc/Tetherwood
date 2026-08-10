// A WGSL shader has two entry points, same shape as shader.wgsl:
// a vertex shader (runs once per corner of our quad) and a fragment
// shader (runs once per pixel covered by the quad). This shader draws
// NO texture — it computes fill/border color purely from each pixel's
// UV coordinate, per the design we just walked through by hand.

// This mirrors Vertex in renderer.rs — WGSL needs its own matching
// declaration of "what a vertex looks like," since the GPU doesn't
// share Rust's type definitions.
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

// What the vertex shader hands off to the fragment shader.
// clip_position is mandatory (where this vertex lands on screen);
// uv is our own pass-through data, carried alongside it.
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// Same transform uniform (projection * view * model) the textured
// pipeline already uses — reused as-is, bound at group 1 to match
// the existing transform_bind_group_layout.
struct TransformUniform {
    transform: mat4x4<f32>,
}
@group(1) @binding(0) var<uniform> transform: TransformUniform;

// New: per-draw debug-rect parameters. fill_color/border_color are
// vec4 (r,g,b,a). border_thickness holds thickness_x, thickness_y
// in UV units (as derived by hand last message), packed into a vec4
// because uniform buffers want 16-byte-aligned fields — the unused
// z/w components are just padding, ignored in the shader.
struct DebugRectUniform {
    fill_color: vec4<f32>,
    border_color: vec4<f32>,
    border_thickness: vec4<f32>,
}
@group(0) @binding(0) var<uniform> debug_rect: DebugRectUniform;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = transform.transform * vec4<f32>(model.position, 1.0);
    out.uv = model.tex_coords;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tx = debug_rect.border_thickness.x;
    let ty = debug_rect.border_thickness.y;

    let near_left = in.uv.x < tx;
    let near_right = in.uv.x > (1.0 - tx);
    let near_top = in.uv.y < ty;
    let near_bottom = in.uv.y > (1.0 - ty);

    if (near_left || near_right || near_top || near_bottom) {
        return debug_rect.border_color;
    }
    return debug_rect.fill_color;
}
