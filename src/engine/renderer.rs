use crate::engine::ids::TextureId;
use crate::engine::scene::Scene;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

struct DebugRect {
    position: glam::Vec2,
    size: glam::Vec2,
    fill_color: [f32; 4],
    border_color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct DebugRectUniform {
    fill_color: [f32; 4],
    border_color: [f32; 4],
    border_thickness: [f32; 4], // x, y used; z, w are padding
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
    debug_bind_group_layout: wgpu::BindGroupLayout,
    debug_pipeline: wgpu::RenderPipeline,
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

        let debug_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("debug rect bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let debug_shader = device.create_shader_module(wgpu::include_wgsl!("debug_shader.wgsl"));

        let debug_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("debug pipeline layout"),
                bind_group_layouts: &[
                    Some(&debug_bind_group_layout),
                    Some(&transform_bind_group_layout),
                ],
                immediate_size: 0,
            });

        let debug_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("debug rect pipeline"),
            layout: Some(&debug_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &debug_shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(Vertex::desc())],
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
            debug_bind_group_layout,
            debug_pipeline,
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

    pub fn render(&mut self, scene: &Scene, show_colliders: bool) -> anyhow::Result<()> {
        if !self.is_surface_configured {
            return Ok(());
        }

        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t)
            | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => return Ok(()),
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => anyhow::bail!("surface lost"),
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

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

        // This makes it static
        let camera_view =
            glam::Mat4::from_translation((screen_center - self.camera_position).extend(0.0));

        // This moves camera with the player
        // let camera_view =
        //     glam::Mat4::from_translation((screen_center - scene.player().position).extend(0.0));

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
        draws.push((
            scene.background.0,
            scene.background_position,
            scene.background_size,
        ));
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

            for wall in &scene.walls {
                debug_rects.push(DebugRect {
                    position: wall.rect.center,
                    size: wall.rect.half_size * 2.0,
                    fill_color: WALL_FILL,
                    border_color: WALL_BORDER,
                });
            }
            for entity in &scene.entities {
                if let Some(collider) = &entity.collider {
                    debug_rects.push(DebugRect {
                        position: entity.position + collider.rect.center,
                        size: collider.rect.half_size * 2.0,
                        fill_color: ENTITY_FILL,
                        border_color: ENTITY_BORDER,
                    });
                }
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
                        view: &view,
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

        const BORDER_PX: f32 = 3.0;

        for rect in &debug_rects {
            let uniform = DebugRectUniform {
                fill_color: rect.fill_color,
                border_color: rect.border_color,
                border_thickness: [BORDER_PX / rect.size.x, BORDER_PX / rect.size.y, 0.0, 0.0],
            };

            let debug_uniform_buffer =
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("debug rect uniform"),
                        contents: bytemuck::cast_slice(&[uniform]),
                        usage: wgpu::BufferUsages::UNIFORM,
                    });
            let debug_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("debug rect bind group"),
                layout: &self.debug_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: debug_uniform_buffer.as_entire_binding(),
                }],
            });

            let model = model_matrix(rect.position, rect.size);
            let transform = projection * camera_view * model;
            self.queue.write_buffer(
                &self.transform_buffer,
                0,
                bytemuck::cast_slice(&transform.to_cols_array()),
            );

            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("debug rect encoder"),
                });
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("debug rect pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
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
                render_pass.set_bind_group(0, &debug_bind_group, &[]);
                render_pass.set_bind_group(1, &self.transform_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
            }
            self.queue.submit(std::iter::once(encoder.finish()));
        }

        self.queue.present(output);

        Ok(())
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
