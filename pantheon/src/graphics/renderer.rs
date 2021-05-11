use std::usize;

use winit::window::Window;

use wgpu::util::DeviceExt;

use crate::graphics::mode::DrawMode;
use crate::graphics::mode::PolygonMode;
use crate::graphics::mode::MODE_COUNT;
use crate::graphics::texture::Texture;
use crate::graphics::texture::TextureKind;
use crate::graphics::Color;
use crate::graphics::Topology;
use crate::math::{Mat4, Vec3};

use super::mesh::Mesh;
use super::vertex::ShadedVertex;
use super::vertex::TexturedVertex;
use super::vertex::Vertex;

type Shaders = (
    wgpu::ShaderModule,
    wgpu::ShaderModule,
    wgpu::ShaderModule,
    wgpu::ShaderModule,
    wgpu::ShaderModule,
    wgpu::ShaderModule,
    wgpu::ShaderModule,
);

#[repr(C)]
pub struct EntityUniforms {
    pub model: Mat4,
    pub view: Mat4,
    pub projection: Mat4,
}

impl EntityUniforms {
    pub fn new(model: Mat4, view: Mat4, projection: Mat4) -> Self {
        Self {
            model,
            view,
            projection,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let data_ptr: *const Self = self;
            let byte_ptr: *const u8 = data_ptr as *const _;
            std::slice::from_raw_parts(byte_ptr, std::mem::size_of::<Self>())
        }
    }
}

#[repr(C)]
pub struct LightUniforms {
    pub light_view_project: Mat4,
    pub light_position: Vec3,
    _padding: u32,
    pub light_color: Color,
}

impl LightUniforms {
    pub fn new(light_view_project: Mat4, light_position: Vec3, light_color: Color) -> Self {
        Self {
            light_view_project,

            light_position,
            _padding: 0,
            light_color,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let data_ptr: *const Self = self;
            let byte_ptr: *const u8 = data_ptr as *const _;
            std::slice::from_raw_parts(byte_ptr, std::mem::size_of::<Self>())
        }
    }
}

pub struct GraphicsContext {
    pub size: winit::dpi::PhysicalSize<u32>,
    pub clear_color: wgpu::Color,
    render_pipelines: Vec<wgpu::RenderPipeline>,

    entities: Vec<Mesh>,

    entity_bind_group_layout: wgpu::BindGroupLayout,
    shadow_bind_group_layout: wgpu::BindGroupLayout,
    forward_bind_group_layout: wgpu::BindGroupLayout,
    textured_quad_bind_group_layout: wgpu::BindGroupLayout,

    depth_texture: Texture,
    pub shadow_texture: Texture,

    pub window_dims: winit::dpi::PhysicalSize<f32>,

    pub entity_uniforms: EntityUniforms,
    pub light_uniforms: LightUniforms,
}

impl GraphicsContext {
    const MAX_LIGHTS: usize = 1;
    const SHADOW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    const SHADOW_SIZE: wgpu::Extent3d = wgpu::Extent3d {
        width: 4096,
        height: 4096,
        depth_or_array_layers: Self::MAX_LIGHTS as u32,
    };

    pub async fn new(
        window: &Window,
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        clear_color: crate::math::Vec4,
    ) -> Self {
        let size = window.inner_size();

        let clear_color = wgpu::Color {
            r: clear_color.x.into(),
            g: clear_color.y.into(),
            b: clear_color.z.into(),
            a: clear_color.w.into(),
        };

        let (
            vs_module,
            fs_module,
            shaded_vs_module,
            shaded_fs_module,
            shadow_bake,
            textured_vs_module,
            textured_fs_module,
        ) = Self::get_shader_modules(device);

        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        let shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: Self::SHADOW_SIZE,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::SHADOW_FORMAT,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
            label: None,
        });
        let shadow_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let shadow_texture = Texture {
            texture: shadow_texture,
            view: shadow_view,
            sampler: shadow_sampler,
        };

        let shadow_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT | wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },

                    count: None,
                }],
                label: Some("uniform_bind_group_layout"),
            });

        let entity_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },

                    count: None,
                }],
                label: Some("uniform_bind_group_layout"),
            });

        let textured_quad_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                            filtering: true,
                        },
                        count: None,
                    },
                ],
                label: Some("uniform_bind_group_layout"),
            });

        let forward_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: true,
                            filtering: true,
                        },
                        count: None,
                    },
                ],
                label: Some("uniform_bind_group_layout"),
            });

        let depth_texture = crate::graphics::texture::Texture::create_depth_texture(
            &device,
            &sc_desc,
            "depth_texture",
        );

        let mut render_pipelines: Vec<wgpu::RenderPipeline> = Vec::with_capacity(MODE_COUNT);
        Self::populate_pipelines(
            &mut render_pipelines,
            device,
            &[&entity_bind_group_layout],
            &vs_module,
            &fs_module,
            sc_desc,
            Vertex::desc,
            DrawMode::normal_modes(),
            Some("normal forward pipelines"),
        );

        Self::populate_pipelines(
            &mut render_pipelines,
            device,
            &[
                &entity_bind_group_layout,
                &forward_bind_group_layout,
                &shadow_bind_group_layout,
            ],
            &shaded_vs_module,
            &shaded_fs_module,
            sc_desc,
            ShadedVertex::desc,
            DrawMode::shaded_modes(),
            Some("shaded forward pipelines"),
        );

        Self::populate_shadow_pipelines(
            &mut render_pipelines,
            device,
            &[&entity_bind_group_layout, &shadow_bind_group_layout],
            &shadow_bake,
            ShadedVertex::desc,
            DrawMode::shadow_modes(),
        );

        Self::populate_textured_pipelines(
            &mut render_pipelines,
            device,
            &[&textured_quad_bind_group_layout],
            &textured_vs_module,
            &textured_fs_module,
            sc_desc,
            TexturedVertex::desc,
            DrawMode::textured_modes(),
        );

        let window_dims = window.inner_size().cast::<f32>();

        Self {
            size,
            clear_color,
            render_pipelines,

            entities: Vec::with_capacity(1024),

            entity_bind_group_layout,
            forward_bind_group_layout,
            shadow_bind_group_layout,
            textured_quad_bind_group_layout,

            depth_texture,

            shadow_texture,

            window_dims,

            entity_uniforms: EntityUniforms {
                model: Mat4::identity(),
                view: Mat4::identity(),
                projection: Mat4::identity(),
            },

            light_uniforms: LightUniforms {
                light_view_project: Mat4::identity(),
                light_position: Vec3::new(20, -20, 0),
                _padding: 0,
                light_color: Color::new(255, 250, 209),
            },
        }
    }

    pub fn reload_shaders(&mut self, device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) {
        let (
            vs_module,
            fs_module,
            shaded_vs_module,
            shaded_fs_module,
            shadow_bake,
            textured_vs_module,
            textured_fs_module,
        ) = Self::get_shader_modules(device);

        self.render_pipelines.clear();

        Self::populate_pipelines(
            &mut self.render_pipelines,
            device,
            &[
                &self.entity_bind_group_layout,
                &self.forward_bind_group_layout,
                &self.shadow_bind_group_layout,
            ],
            &vs_module,
            &fs_module,
            sc_desc,
            Vertex::desc,
            DrawMode::normal_modes(),
            Some("normal forward pipelines"),
        );

        Self::populate_pipelines(
            &mut self.render_pipelines,
            device,
            &[
                &self.entity_bind_group_layout,
                &self.forward_bind_group_layout,
                &self.shadow_bind_group_layout,
            ],
            &shaded_vs_module,
            &shaded_fs_module,
            sc_desc,
            ShadedVertex::desc,
            DrawMode::shaded_modes(),
            Some("shaded forward pipelines"),
        );

        Self::populate_shadow_pipelines(
            &mut self.render_pipelines,
            device,
            &[
                &self.entity_bind_group_layout,
                &self.shadow_bind_group_layout,
            ],
            &shadow_bake,
            ShadedVertex::desc,
            DrawMode::shadow_modes(),
        );

        Self::populate_textured_pipelines(
            &mut self.render_pipelines,
            device,
            &[&self.textured_quad_bind_group_layout],
            &textured_vs_module,
            &textured_fs_module,
            sc_desc,
            TexturedVertex::desc,
            DrawMode::textured_modes(),
        );
    }

    pub fn resize(
        &mut self,
        new_size: winit::dpi::PhysicalSize<u32>,
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        window: &winit::window::Window,
    ) {
        self.size = new_size;
        self.depth_texture = crate::graphics::texture::Texture::create_depth_texture(
            device,
            sc_desc,
            "depth_texture",
        );
        self.window_dims = window.inner_size().cast::<f32>();
    }

    pub fn start(&mut self) {
        self.entities.clear();
    }

    pub fn draw<T>(&mut self, device: &wgpu::Device, mode: DrawMode, verts: &[T])
    where
        T: bytemuck::Pod,
    {
        self.entities.push(Mesh {
            mode,

            vertex: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&verts),
                usage: wgpu::BufferUsage::VERTEX,
            }),

            bind_group: device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.entity_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Uniform Buffer"),
                            contents: self.entity_uniforms.as_bytes(),
                            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
                        })
                        .as_entire_binding(),
                }],
                label: None,
            }),
            index: None,
            count: verts.len() as u32,
            texture: None,
        })
    }

    pub fn draw_indexed<T>(
        &mut self,
        device: &wgpu::Device,
        mode: DrawMode,
        verts: &[T],
        indices: &[u32],
    ) where
        T: bytemuck::Pod,
    {
        self.entities.push(Mesh {
            mode,

            vertex: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&verts),
                usage: wgpu::BufferUsage::VERTEX,
            }),

            bind_group: device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.entity_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Uniform Buffer"),
                            contents: self.entity_uniforms.as_bytes(),
                            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
                        })
                        .as_entire_binding(),
                }],
                label: None,
            }),

            index: Some(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(indices),
                    usage: wgpu::BufferUsage::INDEX,
                }),
            ),
            count: indices.len() as u32,
            texture: None,
        });
    }

    pub fn draw_textured<'a, T>(
        &mut self,
        device: &wgpu::Device,
        mode: DrawMode,
        verts: &[T],
        texture_kind: TextureKind<'a>,
    ) where
        T: bytemuck::Pod,
    {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture = match texture_kind {
            TextureKind::Depth => &self.depth_texture,
            TextureKind::Shadow => &self.shadow_texture,
            TextureKind::Custom(texture) => &texture,
        };
        self.entities.push(Mesh {
            mode,

            vertex: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&verts),
                usage: wgpu::BufferUsage::VERTEX,
            }),

            bind_group: device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.textured_quad_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
                label: None,
            }),
            index: None,
            count: verts.len() as u32,
            texture: None,
        })
    }

    pub fn render(&mut self, device: &wgpu::Device, view: &wgpu::TextureView, queue: &wgpu::Queue) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let shadow_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.shadow_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Uniform Buffer"),
                        contents: self.light_uniforms.as_bytes(),
                        usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
                    })
                    .as_entire_binding(),
            }],
            label: None,
        });

        let forward_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.forward_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.shadow_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.shadow_texture.sampler),
                },
            ],
            label: None,
        });

        encoder.push_debug_group("shadow pass");
        // shadow pass
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Start Shadow Render Pass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.shadow_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_pipeline(
            &self
                .render_pipelines
                .get(usize::from(DrawMode::_ShadowPass(Topology::TriangleList(
                    PolygonMode::Fill,
                ))))
                .unwrap(),
        );

        render_pass.set_bind_group(1, &shadow_bind_group, &[]);

        for entity in &self.entities {
            match entity.mode {
                DrawMode::Shaded(_) => (),
                _ => continue,
            }

            render_pass.set_bind_group(0, &entity.bind_group, &[]);
            render_pass.set_pipeline(
                &self.render_pipelines[usize::from(DrawMode::_ShadowPass(entity.mode.inner()))],
            );
            if let Some(index) = entity.index.as_ref() {
                render_pass.set_vertex_buffer(0, entity.vertex.slice(..));
                render_pass.set_index_buffer(index.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..entity.count, 0, 0..1);
            } else {
                render_pass.set_vertex_buffer(0, entity.vertex.slice(..));
                render_pass.draw(0..entity.count as u32, 0..1);
            }
        }
        drop(render_pass);
        encoder.pop_debug_group();

        encoder.push_debug_group("forward pass");
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Start Render Pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_bind_group(1, &forward_bind_group, &[]);
        render_pass.set_bind_group(2, &shadow_bind_group, &[]);
        for entity in &self.entities {
            //@TODO FIXME need optimization pass, should utilize offsets into singular buffers
            //somehow
            match entity.mode {
                DrawMode::Textured(_) => continue,
                _ => (),
            }

            render_pass.set_bind_group(0, &entity.bind_group, &[]);
            render_pass.set_pipeline(&self.render_pipelines[usize::from(entity.mode)]);
            if let Some(index) = entity.index.as_ref() {
                render_pass.set_vertex_buffer(0, entity.vertex.slice(..));
                render_pass.set_index_buffer(index.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..entity.count, 0, 0..1);
            } else {
                render_pass.set_vertex_buffer(0, entity.vertex.slice(..));
                render_pass.draw(0..entity.count as u32, 0..1);
            }
        }
        drop(render_pass);
        encoder.pop_debug_group();

        encoder.push_debug_group("texture pass");
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Start Texture Render Pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        for entity in &self.entities {
            match entity.mode {
                DrawMode::Textured(_) => (),
                _ => continue,
            }

            //println!("");

            render_pass.set_pipeline(&self.render_pipelines[usize::from(entity.mode)]);
            render_pass.set_bind_group(0, &entity.bind_group, &[]);
            if let Some(index) = entity.index.as_ref() {
                render_pass.set_vertex_buffer(0, entity.vertex.slice(..));
                render_pass.set_index_buffer(index.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..entity.count, 0, 0..1);
            } else {
                render_pass.set_vertex_buffer(0, entity.vertex.slice(..));
                render_pass.draw(0..entity.count as u32, 0..1);
            }
        }
        drop(render_pass);
        encoder.pop_debug_group();

        queue.submit(std::iter::once(encoder.finish()));
    }

    fn populate_pipelines<'a>(
        pipelines: &mut Vec<wgpu::RenderPipeline>,
        device: &wgpu::Device,
        uniform_bind_group_layouts: &[&wgpu::BindGroupLayout],
        vs_module: &wgpu::ShaderModule,
        fs_module: &wgpu::ShaderModule,
        sc_desc: &wgpu::SwapChainDescriptor,
        vert_desc: fn() -> wgpu::VertexBufferLayout<'a>,
        modes: Vec<DrawMode>,
        label: Option<&str>
    ) {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline Layout Descriptor"),
                bind_group_layouts: uniform_bind_group_layouts,
                push_constant_ranges: &[],
            });
        for mode in modes {
            let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label,
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &vs_module,
                    entry_point: "main",
                    buffers: &[vert_desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    // 2.
                    module: &fs_module,
                    entry_point: "main",
                    targets: &[wgpu::ColorTargetState {
                        format: sc_desc.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrite::ALL,
                    }],
                }),

                primitive: wgpu::PrimitiveState {
                    topology: mode.inner().into(),
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: mode.inner().inner().into(),
                    clamp_depth: false,
                    conservative: false,
                },

                depth_stencil: Some(wgpu::DepthStencilState {
                    format: crate::graphics::texture::Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),

                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
            });

            if usize::from(mode) != pipelines.len() {
                panic!("Render pipeline construction broke");
            }

            pipelines.push(render_pipeline);
        }
    }

    fn populate_shadow_pipelines<'a>(
        pipelines: &mut Vec<wgpu::RenderPipeline>,
        device: &wgpu::Device,
        uniform_bind_group_layouts: &[&wgpu::BindGroupLayout],
        vs_module: &wgpu::ShaderModule,
        vert_desc: fn() -> wgpu::VertexBufferLayout<'a>,
        modes: Vec<DrawMode>,
    ) {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline Layout Descriptor"),
                bind_group_layouts: uniform_bind_group_layouts,
                push_constant_ranges: &[],
            });
        for mode in modes {
            let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Shadow Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &vs_module,
                    entry_point: "main",
                    buffers: &[vert_desc()],
                },
                fragment: None,
                primitive: wgpu::PrimitiveState {
                    topology: mode.inner().into(),
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: mode.inner().inner().into(),
                    clamp_depth: false,
                    conservative: false,
                },

                depth_stencil: Some(wgpu::DepthStencilState {
                    format: crate::graphics::texture::Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),

                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
            });

            if usize::from(mode) != pipelines.len() {
                panic!("Render pipeline construction broke");
            }

            pipelines.push(render_pipeline);
        }
    }

    fn populate_textured_pipelines<'a>(
        pipelines: &mut Vec<wgpu::RenderPipeline>,
        device: &wgpu::Device,
        uniform_bind_group_layouts: &[&wgpu::BindGroupLayout],
        vs_module: &wgpu::ShaderModule,
        fs_module: &wgpu::ShaderModule,
        sc_desc: &wgpu::SwapChainDescriptor,
        vert_desc: fn() -> wgpu::VertexBufferLayout<'a>,
        modes: Vec<DrawMode>,
    ) {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline Layout Descriptor"),
                bind_group_layouts: uniform_bind_group_layouts,
                push_constant_ranges: &[],
            });
        for mode in modes {
            let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Texture Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &vs_module,
                    entry_point: "main",
                    buffers: &[vert_desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    // 2.
                    module: &fs_module,
                    entry_point: "main",
                    targets: &[wgpu::ColorTargetState {
                        format: sc_desc.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrite::ALL,
                    }],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: mode.inner().into(),
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: mode.inner().inner().into(),
                    clamp_depth: false,
                    conservative: false,
                },

                depth_stencil: None,

                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
            });

            if usize::from(mode) != pipelines.len() {
                panic!("Render pipeline construction broke");
            }

            pipelines.push(render_pipeline);
        }
    }

    fn get_shader_modules(device: &wgpu::Device) -> Shaders {
        let make_module = |path: &str| {
            let spirv_source = std::fs::read(path).unwrap();

            device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: Some(path),
                source: wgpu::util::make_spirv(&spirv_source),
                flags: wgpu::ShaderFlags::VALIDATION,
            })
        };

        let vs_module = make_module("pantheon/src/graphics/shaders/build/shader.vert.spv");
        let fs_module = make_module("pantheon/src/graphics/shaders/build/shader.frag.spv");

        let shaded_vs_module = make_module("pantheon/src/graphics/shaders/build/shaded.vert.spv");
        let shaded_fs_module = make_module("pantheon/src/graphics/shaders/build/shaded.frag.spv");

        let shadow_bake = make_module("pantheon/src/graphics/shaders/build/bake_shadow.vert.spv");

        let textured_vs_module =
            make_module("pantheon/src/graphics/shaders/build/textured.vert.spv");
        let textured_fs_module =
            make_module("pantheon/src/graphics/shaders/build/textured.frag.spv");

        (
            vs_module,
            fs_module,
            shaded_vs_module,
            shaded_fs_module,
            shadow_bake,
            textured_vs_module,
            textured_fs_module,
        )
    }
}
