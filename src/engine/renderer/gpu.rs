use super::mesh::{INDICES, SolidVertex, VERTICES, Vertex};
use super::texture::{Texture, TextureId, TextureStore};
use crate::engine::renderer;
use crate::engine::scene::Scene;
use anyhow::Result;
use glam::{Mat4, Vec2, Vec4};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;

/// A GPU-acquired frame buffer, ready to be drawn into one or more
/// times before being shown on screen. Bundles the swapchain texture
/// with its view, so render_scene/render_text can share one frame
/// instead of each acquiring (and presenting) their own — which would
/// cause flickering, since the swapchain is double/triple-buffered
/// and two separate acquisitions would land on two different buffers.
pub struct Frame {
    output: wgpu::SurfaceTexture,
    pub(super) view: wgpu::TextureView,
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    pub(super) device: wgpu::Device,
    pub(super) queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    pub(super) render_pipeline: wgpu::RenderPipeline,
    pub(super) vertex_buffer: wgpu::Buffer,
    pub(super) index_buffer: wgpu::Buffer,
    pub(super) transform_buffer: wgpu::Buffer,
    pub(super) transform_bind_group: wgpu::BindGroup,
    pub(super) num_indices: u32,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) bind_groups: Vec<wgpu::BindGroup>,
    pub(super) debug_pipeline: wgpu::RenderPipeline,
    // Held only to keep its GPU resources alive for as long as
    // glyph_atlas_bind_group borrows from them — never read again after
    // construction, since the bind group is what render_text actually uses.
    #[allow(dead_code)]
    font_atlas: Texture,
    pub(super) glyph_atlas_bind_group: wgpu::BindGroup,
    #[allow(dead_code)]
    tile_atlas: Texture,
    pub(super) tile_atlas_bind_group: wgpu::BindGroup,
    #[allow(dead_code)]
    ttf_atlas: Texture,
    pub(super) ttf_atlas_bind_group: wgpu::BindGroup,
    pub ttf_glyphs: std::collections::HashMap<char, crate::engine::renderer::text::TTFGlyph>,
    pub camera_position: Vec2,
    pub smoothed_camera: Vec2,
    pub zoom: f32,
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

        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/shader.wgsl"));

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
            contents: bytemuck::cast_slice(&Mat4::IDENTITY.to_cols_array()),
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

        let mut font_store = TextureStore::new();
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

        let (ttf_atlas, ttf_glyphs) = crate::engine::renderer::text::build_ttf_atlas(
            &device,
            &queue,
            "assets/font/cairopixel.ttf",
            32.0,
        )?;

        let ttf_atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ttf atlas bind group"),
            layout: &texture_bind_group_layout, // same layout, same shape as glyph_atlas_bind_group
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&ttf_atlas.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&ttf_atlas.sampler),
                },
            ],
        });

        let mut tile_store = TextureStore::new();
        let tile_atlas_id = tile_store.load(&device, &queue, "assets/isometric_tiles.aseprite")?;
        let tile_atlas = tile_store.take(tile_atlas_id);

        let tile_atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tile atlas bind group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&tile_atlas.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&tile_atlas.sampler),
                },
            ],
        });

        let debug_shader =
            device.create_shader_module(wgpu::include_wgsl!("shaders/debug_shader.wgsl"));

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
                buffers: &[Some(SolidVertex::desc())],
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
            tile_atlas,
            tile_atlas_bind_group,
            ttf_atlas,
            ttf_atlas_bind_group,
            ttf_glyphs,
            bind_groups: Vec::new(),
            camera_position: Vec2::ZERO,
            smoothed_camera: Vec2::ZERO,
            zoom: 1.0,
        })
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn screen_size(&self) -> Vec2 {
        Vec2::new(self.config.width as f32, self.config.height as f32)
    }

    /// World-space projection - scaled by zoom, always centered on
    /// player_position regardless of camera_position (which only
    /// controls panning/framing, per CameraMode). Keeps zoom always
    /// tracking the player even in Static-camera scenes, where the
    /// camera itself deliberately doesn't follow them.
    pub fn world_projection(&self) -> Mat4 {
        let zoomed_half_width = (self.config.width as f32 / self.zoom) * 0.5;
        let zoomed_half_height = (self.config.height as f32 / self.zoom) * 0.5;
        Mat4::orthographic_rh(
            -zoomed_half_width,
            zoomed_half_width,
            zoomed_half_height,
            -zoomed_half_height,
            -1.0,
            1.0,
        )
    }

    /// Screen-space projection - always 1:1 with real pixels, never
    /// affected by zoom. Used by permanent UI: dialogue, inspector, HUD.
    pub fn screen_projection(&self) -> Mat4 {
        Mat4::orthographic_rh(
            0.0,
            self.config.width as f32,
            self.config.height as f32,
            0.0,
            -1.0,
            1.0,
        )
    }

    pub fn isometric_projection(&self) -> Mat4 {
        const K: f32 = 1.0; // starting guess, tune by feel once visible

        // Column-major: transforms
        // (x, y) -> ((x - y) * k, (x + y) * k * 0.5)
        Mat4::from_cols(
            Vec4::new(K, K * 0.5, 0.0, 0.0),
            Vec4::new(-K, K * 0.5, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    /// Shears a single world-space point into iso space - used for
    /// camera offset and, per-entity, for anchor position. Never
    /// applied to a whole shape here.
    pub fn shear(&self, p: Vec2) -> Vec2 {
        self.isometric_projection()
            .transform_point3(p.extend(0.0))
            .truncate()
    }

    /// Inverse of shear(): maps a sheared (isometric screen-space-ish) offset
    /// back to plain world space. See ADR-087 — isometric_projection() has no
    /// translation, so this is safe to use on offsets as well as points.
    pub fn inverse_shear(&self, p: Vec2) -> Vec2 {
        self.isometric_projection()
            .inverse()
            .transform_point3(p.extend(0.0))
            .truncate()
    }

    /// Camera_position is set per-frame by the caller based on scene.camera_mode (ADR-041).
    /// For isometric mode, camera_position is sheared first so it lands in the same space
    /// as each entity's sheared anchor - shape is never touched by this matrix.
    pub fn sprite_camera_view(&self, is_isometric: bool) -> Mat4 {
        // Pan is driven by camera_position (Static/Follow, per scene).
        // Zoom's expansion center is always the player - scaling the
        // difference between camera_position and player_position by
        // (1 - 1/zoom) keeps that difference visually fixed at zoom=1
        // and shrinks it as zoom increases, so higher zoom pulls the
        // frame toward the player without the camera itself panning.
        if is_isometric {
            Mat4::from_translation((-self.shear(self.smoothed_camera)).extend(0.0))
        } else {
            Mat4::from_translation((-self.smoothed_camera).extend(0.0))
        }
    }

    fn effective_camera_target(&self, player_position: Vec2, is_isometric: bool) -> Vec2 {
        if self.zoom >= renderer::FOLLOW_ZOOM_THRESHOLD || is_isometric {
            player_position
        } else {
            self.camera_position
        }
    }

    /// Call once per frame, before rendering. Eases smoothed_camera
    /// toward wherever it should ideally be (the room's static anchor,
    /// or the player, depending on zoom) - same exponential-smoothing
    /// approach already used for smoothed_fps, applied here to avoid a
    /// hard snap whenever the "should I follow the player" answer flips.
    pub fn update_smoothed_camera(&mut self, player_position: Vec2, is_isometric: bool) {
        let target = self.effective_camera_target(player_position, is_isometric);
        self.smoothed_camera = self.smoothed_camera.lerp(target, 0.1);
    }

    /// Snaps smoothed_camera directly to its current target, bypassing
    /// the lerp - used whenever a scene is (re)constructed, so the
    /// camera doesn't visibly animate in from wherever it was left after
    /// the previous scene, which reads as disorienting (especially at
    /// higher zoom levels, where the animated distance is more visible
    /// on screen).
    pub fn snap_camera(&mut self, player_position: Vec2, is_isometric: bool) {
        self.smoothed_camera = self.effective_camera_target(player_position, is_isometric);
    }

    pub fn dialogue_text_position(&self) -> Vec2 {
        const MARGIN: f32 = 20.0;
        const TEXT_PADDING: f32 = 32.0; // inset from the panel's own edges
        let screen = self.screen_size();
        let panel_height = screen.y / 3.0 - MARGIN;
        let panel_top = screen.y - panel_height - MARGIN;
        Vec2::new(MARGIN + TEXT_PADDING, panel_top + TEXT_PADDING)
    }

    pub fn dialogue_caret_position(&self) -> Vec2 {
        const MARGIN: f32 = 40.0;
        const CARET_INSET: (f32, f32) = (40.0, 60.0); // from the panel's bottom-right corner
        let screen = self.screen_size();
        Vec2::new(
            screen.x - MARGIN - CARET_INSET.0,
            screen.y - MARGIN - CARET_INSET.1,
        )
    }

    pub fn dialogue_text_max_width(&self) -> f32 {
        const MARGIN: f32 = 20.0;
        const TEXT_PADDING: f32 = 32.0;
        let screen = self.screen_size();
        let panel_width = screen.x - MARGIN * 2.0;
        panel_width - TEXT_PADDING * 2.0
    }

    /// Converts a screen-space pixel position (e.g. CursorMoved's
    /// position — top-left origin, y-down, same axis convention as world
    /// space per ADR-031) into world-space coordinates, by inverting the
    /// same translation render_scene's camera_view applies going the
    /// other direction: world = screen - screen_center + camera_position.
    /// Always reads self.camera_position fresh, so results stay correct
    /// across CameraMode::Follow's per-frame camera movement.
    pub fn screen_to_world(&self, screen_pos: Vec2, is_isometric: bool) -> Vec2 {
        let screen_center = Vec2::new(
            self.config.width as f32 / 2.0,
            self.config.height as f32 / 2.0,
        );
        let zoomed_offset = (screen_pos - screen_center) / self.zoom;

        let world_offset = if is_isometric {
            self.inverse_shear(zoomed_offset)
        } else {
            zoomed_offset
        };

        world_offset + self.smoothed_camera
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
