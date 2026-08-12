struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@group(0) @binding(0) var t_atlas: texture_2d<f32>;
@group(0) @binding(1) var s_atlas: sampler;
@group(1) @binding(0) var<uniform> transform: mat4x4<f32>;
@group(2) @binding(0) var<uniform> glyph: GlyphUniform;

struct GlyphUniform {
    uv_offset: vec2<f32>,
    uv_scale: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = transform * vec4<f32>(in.position, 1.0);
    // Remap the quad's existing 0..1 corners onto just this glyph's
    // sub-rectangle of the shared atlas texture.
    out.tex_coords = in.tex_coords * glyph.uv_scale + glyph.uv_offset;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_atlas, s_atlas, in.tex_coords);
}