use winit::window::Window;

use wgpu::util::DeviceExt;

use crate::graphics::mode::DrawMode;
use crate::graphics::mode::PolygonMode;
use crate::graphics::mode::MODE_COUNT;
use crate::graphics::Color;
use crate::graphics::Topology;
use crate::math::{Mat4, Vec3};

use super::vertex::ShadedVertex;
use super::vertex::Vertex;

const VERTICES: &[ShadedVertex] = &[];

const INDICES: &[u16] = &[];

const UNIFORM: &[f32] = &[
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
];
pub struct GraphicsContext {
    pub size: winit::dpi::PhysicalSize<u32>,
    pub clear_color: wgpu::Color,
    render_pipelines: Vec<wgpu::RenderPipeline>,
    pub(crate) command_encoder: Option<wgpu::CommandEncoder>,
    pub(crate) command_buffers: Vec<wgpu::CommandBuffer>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    uniform_bind_group: wgpu::BindGroup,
    depth_texture: crate::graphics::texture::Texture,

    pub window_dims: winit::dpi::PhysicalSize<f32>,

    pub projection_transform: Mat4,
    pub view_transform: Mat4,
    pub model_transform: Mat4,

    pub light_position: Vec3,
    pub light_color: Color,
}

#[repr(C)]
pub struct UniformItems {
    pub model: Mat4,
    pub view: Mat4,
    pub projection: Mat4,
    pub light_position: Vec3,
    _padding: u32,
    pub light_color: Color,
}

impl UniformItems {
    pub fn new(
        model: Mat4,
        view: Mat4,
        projection: Mat4,
        light_position: Vec3,
        light_color: Color,
    ) -> Self {
        Self {
            model,
            view,
            projection,
            light_position,
            _padding: 0,
            light_color,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let data_ptr: *const Self = self;
            let byte_ptr: *const u8 = data_ptr as *const _;
            let byte_slice: &[u8] =
                std::slice::from_raw_parts(byte_ptr, std::mem::size_of::<Self>());

            byte_slice
        }
    }
}

impl GraphicsContext {
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

        let vs_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/shader.vert.spv"));
        let fs_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/shader.frag.spv"));
        let shaded_vs_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/shaded.vert.spv"));
        let shaded_fs_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/shaded.frag.spv"));

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(UNIFORM),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let uniform_bind_group_layout =
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

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
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
            &uniform_bind_group_layout,
            &vs_module,
            &fs_module,
            sc_desc,
            Vertex::desc,
            DrawMode::normal_modes(),
        );

        Self::populate_pipelines(
            &mut render_pipelines,
            device,
            &uniform_bind_group_layout,
            &shaded_vs_module,
            &shaded_fs_module,
            sc_desc,
            ShadedVertex::desc,
            DrawMode::shaded_modes(),
        );

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsage::INDEX,
        });

        let window_dims = window.inner_size().cast::<f32>();

        Self {
            size,
            clear_color,
            render_pipelines,

            command_encoder: None,
            command_buffers: vec![],
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            uniform_bind_group_layout,
            uniform_bind_group,
            depth_texture,

            window_dims,

            model_transform: Mat4::identity(),
            view_transform: Mat4::identity(),
            projection_transform: Mat4::identity(),

            light_position: Vec3::new(20, -20, 0),
            light_color: Color::new(255, 250, 209),
        }
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

    pub fn start(&mut self, device: &wgpu::Device, view: &wgpu::TextureView, queue: &wgpu::Queue) {
        // clear frame
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Start Render Pass"),
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.depth_texture.view,
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
                .get(usize::from(Topology::TriangleList(PolygonMode::Fill)))
                .unwrap(),
        );
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..0, 0..0);
        drop(render_pass);

        queue.submit(std::iter::once(encoder.finish()));

        self.command_encoder =
            Some(device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None }));
    }

    pub fn draw<T>(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        mode: DrawMode,
        verts: &[T],
    ) where
        T: bytemuck::Pod,
    {
        let mut encoder = self.command_encoder.take().unwrap();

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Draw Render Pass"),
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                }),
                stencil_ops: None,
            }),
        });
        render_pass.set_pipeline(&self.render_pipelines[usize::from(mode)]);

        self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let uniform_items = UniformItems::new(
            self.projection_transform,
            self.view_transform,
            self.model_transform,
            self.light_position,
            self.light_color,
        );

        self.uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: uniform_items.as_bytes(),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        self.uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..verts.len() as u32, 0..1);
        drop(render_pass);

        self.command_encoder = Some(encoder);
    }

    pub fn draw_indexed<T>(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        mode: DrawMode,
        verts: &[T],
        indices: &[u32],
    ) where
        T: bytemuck::Pod,
    {
        let mut encoder = self.command_encoder.take().unwrap();

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Draw indexed Render Pass"),
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                }),
                stencil_ops: None,
            }),
        });
        render_pass.set_pipeline(&self.render_pipelines[usize::from(mode)]);

        //let vertices: &[T] = unsafe { &*(verts as *const [F] as *const [T]) };
        self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsage::VERTEX,
        });

        self.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsage::INDEX,
        });

        /*
        let buffer_data: [[f32; 4]; 4] =
            (self.projection_transform * self.view_transform * self.model_transform).into();
        */

        // let mvp = self.projection_transform * self.view_transform * self.model_transform;
        let uniform_items = UniformItems::new(
            self.projection_transform,
            self.view_transform,
            self.model_transform,
            self.light_position,
            self.light_color,
        );
        /*
        let buffer_data: [[[f32; 4]; 4]; 3] = [
            self.projection_transform.into(),
            self.view_transform.into(),
            self.model_transform.into(),
        ];
        */

        self.uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: uniform_items.as_bytes(),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        self.uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
        drop(render_pass);

        self.command_encoder = Some(encoder);
    }

    pub fn push_encoder(&mut self, encoder: wgpu::CommandEncoder) {
        self.command_buffers.push(encoder.finish());
    }

    pub fn render(&mut self, queue: &wgpu::Queue) {
        //@TODO this sucks
        self.command_buffers
            .insert(0, self.command_encoder.take().unwrap().finish());
        queue.submit(self.command_buffers.drain(..));
    }

    fn populate_pipelines<'a>(
        pipelines: &mut Vec<wgpu::RenderPipeline>,
        device: &wgpu::Device,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
        vs_module: &wgpu::ShaderModule,
        fs_module: &wgpu::ShaderModule,
        sc_desc: &wgpu::SwapChainDescriptor,
        vert_desc: fn() -> wgpu::VertexBufferLayout<'a>,
        modes: Vec<DrawMode>,
    ) {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline Layout Descriptor"),
                bind_group_layouts: &[uniform_bind_group_layout],
                push_constant_ranges: &[],
            });
        for mode in modes {
            let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
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
                        alpha_blend: wgpu::BlendState::REPLACE,
                        color_blend: wgpu::BlendState::REPLACE,
                        write_mask: wgpu::ColorWrite::ALL,
                    }],
                }),

                primitive: wgpu::PrimitiveState {
                    topology: mode.inner().into(), // 1.
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw, // 2.
                    cull_mode: wgpu::CullMode::None,
                    polygon_mode: mode.inner().inner().into(),
                },

                depth_stencil: Some(wgpu::DepthStencilState {
                    format: crate::graphics::texture::Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less, // 1.
                    stencil: wgpu::StencilState::default(),     // 2.
                    bias: wgpu::DepthBiasState::default(),
                    // Setting this to true requires Features::DEPTH_CLAMPING
                    clamp_depth: false,
                }),

                multisample: wgpu::MultisampleState {
                    count: 1,                         // 2.
                    mask: !0,                         // 3.
                    alpha_to_coverage_enabled: false, // 4.
                },
            });

            //pipelines[usize::from(mode)] = render_pipeline;
            pipelines.push(render_pipeline);
        }
    }
}
