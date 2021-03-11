use winit::window::Window;

use wgpu::util::DeviceExt;

use crate::graphics::Topology;
use crate::math::Mat4;
use crate::math::Vec3;

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f64; 3],
    pub color: [f64; 3],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Double3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f64; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Double3,
                },
            ],
        }
    }
}

impl From<(Vec3, Vec3)> for Vertex {
    fn from(vecs: (Vec3, Vec3)) -> Self {
        Self {
            position: [vecs.0.x, vecs.0.y, vecs.0.z],
            color: [vecs.1.x, vecs.1.y, vecs.1.z],
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 0.0, 1.0],
    },
];

const INDICES: &[u16] = &[0, 1, 2];

pub struct GraphicsContext {
    pub size: winit::dpi::PhysicalSize<u32>,
    pub clear_color: wgpu::Color,
    render_pipeline: wgpu::RenderPipeline,

    pub(crate) command_encoder: Option<wgpu::CommandEncoder>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    pub window_dims: winit::dpi::LogicalSize<f64>,

    // @TODO pass uniform buffers into shaders
    pub projection_transform: Mat4,
    pub view_transform: Mat4,
    pub model_transform: Mat4,
}

impl GraphicsContext {
    // Creating some of the wgpu types requires async code
    pub async fn new(
        window: &Window,
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        clear_color: crate::math::Vec4,
    ) -> Self {
        let size = window.inner_size();

        let clear_color = wgpu::Color {
            r: clear_color.x,
            g: clear_color.y,
            b: clear_color.z,
            a: clear_color.w,
        };

        let vs_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/shader.vert.spv"));
        let fs_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/shader.frag.spv"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline Layout Descriptor"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[],
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
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: wgpu::CullMode::Back,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
            },

            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
        });

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

        let window_dims = window.inner_size().to_logical::<f64>(window.scale_factor());

        Self {
            size,
            clear_color,
            render_pipeline,
            command_encoder: None,
            vertex_buffer,
            index_buffer,

            window_dims,

            model_transform: Mat4::identity(),
            view_transform: Mat4::identity(),
            projection_transform: Mat4::identity(),
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
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
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..0, 0..0);
        drop(render_pass);

        queue.submit(std::iter::once(encoder.finish()));

        self.command_encoder =
            Some(device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None }));
    }

    pub fn draw(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        _mode: Topology,
        verts: &[(Vec3, Vec3)],
    ) {
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
            depth_stencil_attachment: None,
        });
        render_pass.set_pipeline(&self.render_pipeline);

        let vertices: &[Vertex] = unsafe { std::mem::transmute(verts) };
        self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsage::VERTEX,
        });

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..verts.len() as u32, 0..1);
        drop(render_pass);

        self.command_encoder = Some(encoder);
    }

    pub fn draw_indexed(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        _mode: Topology,
        verts: &[(Vec3, Vec3)],
        indices: &[u16],
    ) {
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
            depth_stencil_attachment: None,
        });
        render_pass.set_pipeline(&self.render_pipeline);

        let vertices: &[Vertex] = unsafe { std::mem::transmute(verts) };
        self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsage::VERTEX,
        });

        self.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsage::INDEX,
        });

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
        drop(render_pass);

        self.command_encoder = Some(encoder);
    }

    pub fn render(&mut self, queue: &wgpu::Queue) {
        // submit will accept anything that implements IntoIter
        queue.submit(std::iter::once(
            self.command_encoder.take().unwrap().finish(),
        ));
        self.command_encoder = None;
    }
}
