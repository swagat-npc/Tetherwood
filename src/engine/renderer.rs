use anyhow::Result;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::engine::entity::TriggerKind;
use crate::engine::ids::TextureId;
use crate::engine::scene::Scene;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

struct DebugRect {
    position: glam::Vec2,
    size: glam::Vec2,
    fill_color: [f32; 4],
    border_color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct DebugVertex {
    position: [f32; 3],
    /// 0..1 position within this specific rect — same role tex_coords
    /// played before (which corner of the unit quad this is), just
    /// renamed since there's no texture being sampled here. The
    /// fragment shader still needs this to know "how close to an
    /// edge am I" for the border test.
    local_uv: [f32; 2],
    fill_color: [f32; 4],
    border_color: [f32; 4],
    /// Per-axis thickness in local_uv space — same calculation
    /// ADR-043 already does (pixels / rect_width, pixels / rect_height),
    /// just baked into each vertex now instead of a per-draw uniform.
    border_thickness: [f32; 2],
}

impl DebugVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<DebugVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3, // position
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2, // local_uv
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3 + 2]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4, // fill_color
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3 + 2 + 4]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4, // border_color
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3 + 2 + 4 + 4]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x2, // border_thickness
                },
            ],
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [0.0, 1.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [1.0, 0.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
];

const INDICES: &[u16] = &[0, 1, 2, 0, 3, 2];

/// Builds the model matrix for a sprite: scale a unit quad to `size`,
/// then translate so `position` lands at the sprite's center (ADR-033).
fn model_matrix(position: glam::Vec2, size: glam::Vec2) -> glam::Mat4 {
    let half_size = size / 2.0;
    let scale = glam::Mat4::from_scale(size.extend(1.0));
    let translate = glam::Mat4::from_translation((position - half_size).extend(0.0));
    translate * scale
}

fn push_center_marker(debug_rects: &mut Vec<DebugRect>, center: glam::Vec2, scale: f32) {
    const ARM_LENGTH: f32 = 8.0;
    const THICKNESS: f32 = 2.0;
    const X_COLOR: [f32; 4] = [1.0, 0.15, 0.15, 1.0]; // X-Axis
    const Y_COLOR: [f32; 4] = [0.15, 1.0, 0.15, 1.0]; // Y-Axis

    debug_rects.push(DebugRect {
        position: center,
        size: glam::Vec2::new(ARM_LENGTH * scale, THICKNESS * scale),
        fill_color: X_COLOR,
        border_color: X_COLOR,
    });
    debug_rects.push(DebugRect {
        position: center,
        size: glam::Vec2::new(THICKNESS * scale, ARM_LENGTH * scale),
        fill_color: Y_COLOR,
        border_color: Y_COLOR,
    });
}

/// Builds one vertex+index buffer for an entire string — 4 vertices
/// and 6 indices per glyph, with each glyph's final screen position
/// and atlas UV baked directly into its vertices. Unlike the earlier
/// per-glyph-uniform approach, no per-draw remapping is needed: the
/// vertex data already is the answer, so the whole string draws in
/// one draw call instead of one per glyph.
fn build_text_mesh(glyphs: &[crate::engine::text::PositionedGlyph]) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::with_capacity(glyphs.len() * 4);
    let mut indices = Vec::with_capacity(glyphs.len() * 6);

    for glyph in glyphs {
        let (uv_min, uv_max) = crate::engine::text::glyph_uv(glyph.cell.0, glyph.cell.1);
        let top_left = glyph.position;
        let bottom_right = glyph.position + crate::engine::text::GLYPH_SIZE;

        let base = vertices.len() as u16;
        vertices.push(Vertex {
            position: [top_left.x, top_left.y, 0.0],
            tex_coords: [uv_min.x, uv_min.y],
        });
        vertices.push(Vertex {
            position: [top_left.x, bottom_right.y, 0.0],
            tex_coords: [uv_min.x, uv_max.y],
        });
        vertices.push(Vertex {
            position: [bottom_right.x, bottom_right.y, 0.0],
            tex_coords: [uv_max.x, uv_max.y],
        });
        vertices.push(Vertex {
            position: [bottom_right.x, top_left.y, 0.0],
            tex_coords: [uv_max.x, uv_min.y],
        });

        // Same 0,1,2,0,3,2 winding as the existing static VERTICES/
        // INDICES pair, just offset per glyph via `base`.
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 3, base + 2]);
    }

    (vertices, indices)
}

fn build_debug_mesh(rects: &[DebugRect]) -> (Vec<DebugVertex>, Vec<u16>) {
    const BORDER_PX: f32 = 3.0;

    let mut vertices = Vec::with_capacity(rects.len() * 4);
    let mut indices = Vec::with_capacity(rects.len() * 6);
    for rect in rects {
        let half_size = rect.size / 2.0;
        let top_left = rect.position - half_size;
        let bottom_right = rect.position + half_size;
        let thickness = [BORDER_PX / rect.size.x, BORDER_PX / rect.size.y];

        let base = vertices.len() as u16;
        vertices.push(DebugVertex {
            position: [top_left.x, top_left.y, 0.0],
            local_uv: [0.0, 0.0],
            fill_color: rect.fill_color,
            border_color: rect.border_color,
            border_thickness: thickness,
        });
        vertices.push(DebugVertex {
            position: [top_left.x, bottom_right.y, 0.0],
            local_uv: [0.0, 1.0],
            fill_color: rect.fill_color,
            border_color: rect.border_color,
            border_thickness: thickness,
        });
        vertices.push(DebugVertex {
            position: [bottom_right.x, bottom_right.y, 0.0],
            local_uv: [1.0, 1.0],
            fill_color: rect.fill_color,
            border_color: rect.border_color,
            border_thickness: thickness,
        });
        vertices.push(DebugVertex {
            position: [bottom_right.x, top_left.y, 0.0],
            local_uv: [1.0, 0.0],
            fill_color: rect.fill_color,
            border_color: rect.border_color,
            border_thickness: thickness,
        });
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 3, base + 2]);
    }
    (vertices, indices)
}

/// A GPU-acquired frame buffer, ready to be drawn into one or more
/// times before being shown on screen. Bundles the swapchain texture
/// with its view, so render_scene/render_text can share one frame
/// instead of each acquiring (and presenting) their own — which would
/// cause flickering, since the swapchain is double/triple-buffered
/// and two separate acquisitions would land on two different buffers.
pub struct Frame {
    output: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    transform_buffer: wgpu::Buffer,
    transform_bind_group: wgpu::BindGroup,
    num_indices: u32,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    bind_groups: Vec<wgpu::BindGroup>,
    debug_pipeline: wgpu::RenderPipeline,
    // Held only to keep its GPU resources alive for as long as
    // glyph_atlas_bind_group borrows from them — never read again after
    // construction, since the bind group is what render_text actually uses.
    #[allow(dead_code)]
    font_atlas: crate::engine::texture::Texture,
    glyph_atlas_bind_group: wgpu::BindGroup,
    pub camera_position: glam::Vec2,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

        let surface = instance.create_surface(window)?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: true,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Tetherwood Device"),
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let transform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("transform bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render pipeline layout"),
                bind_group_layouts: &[
                    Some(&texture_bind_group_layout),
                    Some(&transform_bind_group_layout),
                ],
                immediate_size: 0,
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(Vertex::desc())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad vertex buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad index buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("transform buffer"),
            contents: bytemuck::cast_slice(&glam::Mat4::IDENTITY.to_cols_array()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("transform bind group"),
            layout: &transform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: transform_buffer.as_entire_binding(),
            }],
        });

        let mut font_store = crate::engine::texture::TextureStore::new();
        let font_atlas_id =
            font_store.load(&device, &queue, "assets/good_neighbors_font.aseprite")?;
        let font_atlas = font_store.take(font_atlas_id);

        let glyph_atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("glyph atlas bind group"),
            layout: &texture_bind_group_layout, // reused — same shape as any sprite texture
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&font_atlas.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&font_atlas.sampler),
                },
            ],
        });

        let debug_shader = device.create_shader_module(wgpu::include_wgsl!("debug_shader.wgsl"));

        let debug_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("debug pipeline layout"),
                bind_group_layouts: &[Some(&transform_bind_group_layout)],
                immediate_size: 0,
            });

        let debug_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("debug rect pipeline"),
            layout: Some(&debug_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &debug_shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(DebugVertex::desc())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &debug_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        let num_indices = INDICES.len() as u32;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };

        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            transform_buffer,
            transform_bind_group,
            num_indices,
            texture_bind_group_layout,
            debug_pipeline,
            font_atlas,
            glyph_atlas_bind_group,
            bind_groups: Vec::new(),
            camera_position: glam::Vec2::ZERO,
        })
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn screen_size(&self) -> glam::Vec2 {
        glam::Vec2::new(self.config.width as f32, self.config.height as f32)
    }

    /// Converts a screen-space pixel position (e.g. CursorMoved's
    /// position — top-left origin, y-down, same axis convention as world
    /// space per ADR-031) into world-space coordinates, by inverting the
    /// same translation render_scene's camera_view applies going the
    /// other direction: world = screen - screen_center + camera_position.
    /// Always reads self.camera_position fresh, so results stay correct
    /// across CameraMode::Follow's per-frame camera movement.
    pub fn screen_to_world(&self, screen_pos: glam::Vec2) -> glam::Vec2 {
        let screen_center = glam::Vec2::new(
            self.config.width as f32 / 2.0,
            self.config.height as f32 / 2.0,
        );
        screen_pos - screen_center + self.camera_position
    }

    /// Acquires the next frame to draw into. Returns Ok(None) for the
    /// same transient cases render() previously handled by silently
    /// returning early (Outdated reconfigures and retries next frame;
    /// Occluded/Timeout/Validation just skip this frame).
    pub fn acquire_frame(&mut self) -> Result<Option<Frame>> {
        if !self.is_surface_configured {
            return Ok(None);
        }

        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t)
            | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => return Ok(None),
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(None);
            }
            wgpu::CurrentSurfaceTexture::Lost => anyhow::bail!("surface lost"),
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        Ok(Some(Frame { output, view }))
    }

    pub fn render_scene(&mut self, frame: &Frame, scene: &Scene, show_colliders: bool) {
        let projection = glam::Mat4::orthographic_rh(
            0.0,
            self.config.width as f32,
            self.config.height as f32,
            0.0,
            -1.0,
            1.0,
        );
        let screen_center = glam::Vec2::new(
            self.config.width as f32 / 2.0,
            self.config.height as f32 / 2.0,
        );

        // camera_position is set per-frame by the caller based on scene.camera_mode (ADR-041).
        let camera_view =
            glam::Mat4::from_translation((screen_center - self.camera_position).extend(0.0));

        // y-sort: entities drawn in ascending order of baseline
        // (bottom edge = position.y + half height), so entities with a
        // lower baseline draw first and correctly end up behind
        // entities with a higher one.
        let mut order: Vec<usize> = (0..scene.entities.len()).collect();
        order.sort_by(|&a, &b| {
            let baseline_a = scene.entities[a].position.y + scene.entities[a].size.y / 2.0;
            let baseline_b = scene.entities[b].position.y + scene.entities[b].size.y / 2.0;
            baseline_a.partial_cmp(&baseline_b).unwrap()
        });

        // Full draw list: background first (always behind, never
        // y-sorted), then entities in sorted order. Each entry is
        // (bind group index, position, size).
        let mut draws: Vec<(usize, glam::Vec2, glam::Vec2)> = Vec::new();
        for bg in &scene.background {
            draws.push((bg.texture.0, bg.position, bg.size));
        }

        for &idx in &order {
            let entity = &scene.entities[idx];
            if let Some(texture_id) = entity.texture_id {
                draws.push((texture_id.0, entity.position, entity.size));
            }
        }

        let mut debug_rects: Vec<DebugRect> = Vec::new();
        if show_colliders {
            const WALL_FILL: [f32; 4] = [1.0, 0.0, 0.0, 0.15];
            const WALL_BORDER: [f32; 4] = [0.7, 0.0, 0.0, 0.9];
            const ENTITY_FILL: [f32; 4] = [0.0, 0.4, 1.0, 0.15];
            const ENTITY_BORDER: [f32; 4] = [0.0, 0.2, 0.8, 0.9];
            const TRIGGER_FILL: [f32; 4] = [0.0, 1.0, 0.0, 0.15];
            const TRIGGER_BORDER: [f32; 4] = [0.0, 0.7, 0.0, 0.9];
            const INTERACT_FILL: [f32; 4] = [1.0, 1.0, 0.0, 0.15];
            const INTERACT_BORDER: [f32; 4] = [0.7, 0.7, 0.0, 0.9];

            push_center_marker(&mut debug_rects, glam::Vec2::ZERO, 1.0);

            for wall in &scene.walls {
                debug_rects.push(DebugRect {
                    position: wall.rect.center,
                    size: wall.rect.half_size * 2.0,
                    fill_color: WALL_FILL,
                    border_color: WALL_BORDER,
                });
                push_center_marker(&mut debug_rects, wall.rect.center, 1.0);
            }
            for entity in &scene.entities {
                if let Some(collider) = &entity.collider {
                    debug_rects.push(DebugRect {
                        position: entity.position + collider.rect.center,
                        size: collider.rect.half_size * 2.0,
                        fill_color: ENTITY_FILL,
                        border_color: ENTITY_BORDER,
                    });
                    push_center_marker(
                        &mut debug_rects,
                        entity.position + collider.rect.center,
                        1.0,
                    );
                }
            }
            for trigger in &scene.triggers {
                let interactive: bool = match trigger.kind {
                    TriggerKind::Warp { .. } => false,
                    TriggerKind::Interact { .. } => true,
                };

                debug_rects.push(DebugRect {
                    position: trigger.rect.center,
                    size: trigger.rect.half_size * 2.0,
                    fill_color: if interactive {
                        INTERACT_FILL
                    } else {
                        TRIGGER_FILL
                    },
                    border_color: if interactive {
                        INTERACT_BORDER
                    } else {
                        TRIGGER_BORDER
                    },
                });
                push_center_marker(&mut debug_rects, trigger.rect.center, 1.0);
            }
        }

        // Each draw gets its own encoder and its own submit. See the
        // explanation above the code block in this message: repeatedly
        // calling write_buffer on the same buffer before a single
        // shared submit() would let every draw see only the *last*
        // written transform. Submitting per-draw guarantees each
        // write_buffer lands before its own draw executes. The first
        // draw clears the screen; every draw after it loads (paints
        // over) what's already there instead of erasing it.
        for (i, (bind_group_index, position, size)) in draws.iter().enumerate() {
            let model = model_matrix(*position, *size);
            let transform = projection * camera_view * model;
            self.queue.write_buffer(
                &self.transform_buffer,
                0,
                bytemuck::cast_slice(&transform.to_cols_array()),
            );

            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("draw encoder"),
                });

            {
                let load_op = if i == 0 {
                    wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    })
                } else {
                    wgpu::LoadOp::Load
                };

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("draw pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &frame.view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: load_op,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                    multiview_mask: None,
                });

                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, &self.bind_groups[*bind_group_index], &[]);
                render_pass.set_bind_group(1, &self.transform_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
            }

            self.queue.submit(std::iter::once(encoder.finish()));
        }

        if !debug_rects.is_empty() {
            let (vertices, indices) = build_debug_mesh(&debug_rects);
            let debug_vertex_buffer =
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("debug vertex buffer"),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
            let debug_index_buffer =
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("debug index buffer"),
                        contents: bytemuck::cast_slice(&indices),
                        usage: wgpu::BufferUsages::INDEX,
                    });

            // Positions are already world-space (baked from each rect's real
            // center/size), so only projection*camera_view is needed here —
            // no per-rect model matrix, same simplification build_text_mesh
            // already applies for screen-space text.
            let transform = projection * camera_view;
            self.queue.write_buffer(
                &self.transform_buffer,
                0,
                bytemuck::cast_slice(&transform.to_cols_array()),
            );

            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("debug rects encoder"),
                });
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("debug rects pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &frame.view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                    multiview_mask: None,
                });
                render_pass.set_pipeline(&self.debug_pipeline);
                render_pass.set_bind_group(0, &self.transform_bind_group, &[]);
                render_pass.set_vertex_buffer(0, debug_vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(debug_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
            }
            self.queue.submit(std::iter::once(encoder.finish()));
        }
    }

    pub fn render_text(&mut self, frame: &Frame, glyphs: &[crate::engine::text::PositionedGlyph]) {
        // Screen-space only — no camera_view term, so text stays fixed
        // to the window regardless of camera position or mode (HUD).
        if glyphs.is_empty() {
            return;
        }

        let (vertices, indices) = build_text_mesh(glyphs);
        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("text vertex buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("text index buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        // Screen=space only (ADR-058) - no camera_view term, so text stays fixed
        // to the window regardless of camera position or mode (HUD).
        let projection = glam::Mat4::orthographic_rh(
            0.0,
            self.config.width as f32,
            self.config.height as f32,
            0.0,
            -1.0,
            1.0,
        );
        self.queue.write_buffer(
            &self.transform_buffer,
            0,
            bytemuck::cast_slice(&projection.to_cols_array()),
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("text render encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("text draw pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // always paint over — scene already drew this frame
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.glyph_atlas_bind_group, &[]);
            render_pass.set_bind_group(1, &self.transform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn present_frame(&mut self, frame: Frame) {
        self.queue.present(frame.output);
    }

    /// Builds one bind group per texture currently in the scene's
    /// TextureStore, indexed identically to TextureId — call once,
    /// right after a scene is constructed, before the first render().
    pub fn prepare_scene(&mut self, scene: &Scene) {
        self.bind_groups.clear();
        for i in 0..scene.texture_store.len() {
            let texture = scene.texture_store.get(TextureId(i));
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("scene texture bind group"),
                layout: &self.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&texture.sampler),
                    },
                ],
            });
            self.bind_groups.push(bind_group);
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }
    }
}
