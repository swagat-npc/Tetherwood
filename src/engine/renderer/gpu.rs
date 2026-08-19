use super::mesh::{INDICES, SolidVertex, VERTICES, Vertex};
use super::texture::{Texture, TextureId, TextureStore};
use crate::engine::scene::Scene;
use anyhow::Result;
use glam::Vec2;
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
    pub camera_position: Vec2,
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
            bind_groups: Vec::new(),
            camera_position: Vec2::ZERO,
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

    pub fn screen_projection(&self) -> glam::Mat4 {
        glam::Mat4::orthographic_rh(
            0.0,
            self.config.width as f32,
            self.config.height as f32,
            0.0,
            -1.0,
            1.0,
        )
    }

    pub fn isometric_projection(&self) -> glam::Mat4 {
        const K: f32 = 1.0; // starting guess, tune by feel once visible

        // Column-major: transforms
        // (x, y) -> ((x - y) * k, (x + y) * k * 0.5)
        glam::Mat4::from_cols(
            glam::Vec4::new(K, K * 0.5, 0.0, 0.0),
            glam::Vec4::new(-K, K * 0.5, 0.0, 0.0),
            glam::Vec4::new(0.0, 0.0, 1.0, 0.0),
            glam::Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
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
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        let screen_center = Vec2::new(
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
