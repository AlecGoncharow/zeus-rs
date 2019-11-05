use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, CpuBufferPool};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::DynamicState;
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::format::Format;
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass};
use vulkano::image::attachment::AttachmentImage;
use vulkano::image::SwapchainImage;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::swapchain;
use vulkano::swapchain::SwapchainAcquireFuture;
use vulkano::swapchain::{
    AcquireError, PresentMode, Surface, SurfaceTransform, Swapchain, SwapchainCreationError,
};
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture};
use vulkano_win::VkSurfaceBuild;
use winit::dpi::LogicalSize;
use winit::EventsLoop;
use winit::{Window, WindowBuilder};

use vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool;
use vulkano::format::ClearValue;

use std::sync::Arc;

use crate::math::Mat4;
use crate::math::Vec3;
use crate::math::Vec4;
use std::collections::HashMap;
use std::iter;

use vulkano::pipeline::depth_stencil::Compare;
use vulkano::pipeline::depth_stencil::DepthBounds;
use vulkano::pipeline::depth_stencil::DepthStencil;
use vulkano::pipeline::depth_stencil::Stencil;
use vulkano::pipeline::depth_stencil::StencilOp;

use crate::graphics::PolygonMode;
use crate::graphics::Topology;

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/graphics/shaders/vert.glsl"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/graphics/shaders/frag.glsl"
    }
}

pub struct GraphicsContext {
    pub recreate_swapchain: bool,
    previous_frame_end: Option<Box<dyn GpuFuture>>,

    swapchain: Arc<Swapchain<winit::Window>>,

    queue: Arc<Queue>,
    device: Arc<Device>,

    surface: Arc<Surface<Window>>,

    vertex_shader: vs::Shader,
    frag_shader: fs::Shader,

    graphics_pipelines: HashMap<Topology, Arc<dyn GraphicsPipelineAbstract + Send + Sync>>,
    graphics_pool: HashMap<
        Topology,
        FixedSizeDescriptorSetsPool<Arc<dyn GraphicsPipelineAbstract + Send + Sync>>,
    >,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,

    //depth_buffer: Arc<AttachmentImage>,
    //color_buffer: Arc<AttachmentImage>,
    uniform_buffer: Arc<CpuBufferPool<vs::ty::Data>>,
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,

    pub(crate) graphics_command_buffer: Option<AutoCommandBufferBuilder>,
    acquire_future: Option<SwapchainAcquireFuture<winit::Window>>,
    image_num: usize,

    pub window_dims: LogicalSize,
    //pub camera_projection: Arc<dyn CameraProjection>,
    pub projection_transform: Mat4,
    pub view_transform: Mat4,
    pub model_transform: Mat4,
    //pub model_projection: Arc<dyn ModelProjection>,
}

#[derive(Default, Debug, Clone)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}
vulkano::impl_vertex!(Vertex, position, color);

impl GraphicsContext {
    pub fn new_default(events_loop: &EventsLoop) -> Self {
        let instance = {
            let extensions = vulkano_win::required_extensions();

            Instance::new(None, &extensions.into(), None).unwrap()
        };

        let physical = PhysicalDevice::enumerate(&instance)
            .next()
            .expect("no device avaliable");

        for family in physical.queue_families() {
            println!(
                "Found a queue family with {:?} queue(s)",
                family.queues_count()
            );
        }
        println!(
            "Using device: {} (type: {:?})",
            physical.name(),
            physical.ty()
        );

        let dims = LogicalSize::from((1080, 1080));
        let surface = WindowBuilder::new()
            .with_dimensions(dims)
            .with_title("real game engine window")
            .with_maximized(true)
            .build_vk_surface(events_loop, instance.clone())
            .unwrap();
        let window = surface.window();

        let queue_family = physical
            .queue_families()
            .find(|&q| {
                // We take the first queue that supports drawing to our window.
                q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
            })
            .unwrap();

        // init device
        let device_ext = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };
        let (device, mut queues) = Device::new(
            physical,
            physical.supported_features(),
            &device_ext,
            [(queue_family, 0.5)].iter().cloned(),
        )
        .unwrap();

        let queue = queues.next().unwrap();

        let window_dims = window.get_inner_size().unwrap();
        let (swapchain, images) = {
            let caps = surface.capabilities(physical).unwrap();

            let usage = caps.supported_usage_flags;
            let alpha = caps.supported_composite_alpha.iter().next().unwrap();

            // Choosing the internal format that the images will have.
            let format = caps.supported_formats[0].0;
            // Because for both of these cases, the swapchain needs to be the window dimensions, we just use that.
            let initial_dimensions = if let Some(dimensions) = window.get_inner_size() {
                let dimensions: (u32, u32) =
                    dimensions.to_physical(window.get_hidpi_factor()).into();
                [dimensions.0, dimensions.1]
            } else {
                [0, 0]
            };

            println!("{:#?}", initial_dimensions);
            // Please take a look at the docs for the meaning of the parameters we didn't mention.
            Swapchain::new(
                device.clone(),
                surface.clone(),
                caps.min_image_count,
                format,
                initial_dimensions,
                1,
                usage,
                &queue,
                SurfaceTransform::Identity,
                alpha,
                PresentMode::Fifo,
                true,
                None,
            )
            .unwrap()
        };

        let vertex_shader = vs::Shader::load(device.clone()).unwrap();
        let frag_shader = fs::Shader::load(device.clone()).unwrap();

        let vertex_buffer = {
            CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), [].iter().cloned())
                .unwrap()
        };

        let render_pass = Arc::new(
            vulkano::single_pass_renderpass!(
                device.clone(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: swapchain.format(),
                        samples: 1,
                    },
                    depth: {
                        load: Clear,
                        store: Store,
                        format: Format::D16Unorm,
                        samples: 1,
                }
                },
                pass: {
                    color: [color],

                    depth_stencil: {depth}
                }
            )
            .unwrap(),
        );
        /*

        let render_pass = Arc::new(
            vulkano::ordered_passes_renderpass!(
            device.clone(),
            attachments: {
                // The image that will contain the final rendering (in this example the swapchain
                // image, but it could be another image).
                final_color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.format(),
                    samples: 1,
                },
                // Will be bound to `self.depth_buffer`.
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D16Unorm,
                    samples: 1,
                },
                initial_color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.format(),
                    samples: 1,
                }
            },
            passes: [
                // Write to the diffuse, normals and depth attachments.
            {
                color: [initial_color],
                depth_stencil: {depth},
                input: []
            },
            // Apply lighting by reading these three attachments and writing to `final_color`.
            {
                color: [final_color],
                depth_stencil: {},
                input: [depth, initial_color]
            }
            ]
                )
            .unwrap(),
        );

        */
        let (graphics_pipelines, framebuffers) = window_size_dependent_setup(
            &device,
            &vertex_shader,
            &frag_shader,
            &images,
            render_pass.clone(),
        );

        // iterate over pipelines and make descriptors
        let graphics_pool: HashMap<
            Topology,
            FixedSizeDescriptorSetsPool<Arc<dyn GraphicsPipelineAbstract + Send + Sync>>,
        > = {
            let mut set = HashMap::new();
            for top in Topology::iterator() {
                let pipeline = graphics_pipelines.get(top).unwrap();
                let descriptor = FixedSizeDescriptorSetsPool::new(pipeline.clone(), 0);
                set.insert(*top, descriptor);
            }

            set
        };

        let uniform_buffer = Arc::new(CpuBufferPool::<vs::ty::Data>::new(
            device.clone(),
            BufferUsage::all(),
        ));
        let graphics_command_buffer = Some(
            AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family())
                .unwrap(),
        );

        let recreate_swapchain = false;
        let previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<dyn GpuFuture>);

        //let camera_projection = Arc::new(crate::graphics::DefaultCamera {});
        //let model_projection = Arc::new(crate::graphics::DefaultModelProjecton {});

        let (image_num, acquire_future) =
            match swapchain::acquire_next_image(swapchain.clone(), None) {
                Ok((n, f)) => (n, Some(f)),
                Err(err) => panic!("{:?}", err),
            };

        GraphicsContext {
            recreate_swapchain,
            previous_frame_end,
            device,

            swapchain,
            queue,
            surface,

            vertex_shader,
            frag_shader,
            graphics_pipelines,
            graphics_pool,
            render_pass,
            framebuffers,

            vertex_buffer,
            uniform_buffer,
            window_dims,

            model_transform: Mat4::identity(),
            view_transform: Mat4::identity(),
            projection_transform: Mat4::identity(),

            graphics_command_buffer,
            image_num,
            acquire_future,
        }
    }

    pub fn start(&mut self, clear_color: Vec4) {
        let window = self.surface.window();
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        // Whenever the window resizes we need to recreate everything dependent on the window size.
        // In this example that includes the swapchain, the framebuffers and the dynamic state viewport.
        if self.recreate_swapchain {
            // Get the new dimensions of the window.
            let dimensions = if let Some(dimensions) = window.get_inner_size() {
                let dimensions: (u32, u32) =
                    dimensions.to_physical(window.get_hidpi_factor()).into();
                [dimensions.0, dimensions.1]
            } else {
                return;
            };

            self.window_dims = window.get_inner_size().unwrap();
            println!("{:#?}", dimensions);

            let (new_swapchain, new_images) =
                match self.swapchain.recreate_with_dimension(dimensions) {
                    Ok(r) => r,
                    // This error tends to happen when the user is manually resizing the window.
                    // Simply restarting the loop is the easiest way to fix this issue.
                    Err(SwapchainCreationError::UnsupportedDimensions) => return,
                    Err(err) => panic!("{:?}", err),
                };

            self.swapchain = new_swapchain;
            // Because framebuffers contains an Arc on the old swapchain, we need to
            // recreate framebuffers as well.
            let (new_pipelines, new_framebuffers) = window_size_dependent_setup(
                &self.device,
                &self.vertex_shader,
                &self.frag_shader,
                &new_images,
                self.render_pass.clone(),
            );

            self.graphics_pipelines = new_pipelines;
            self.framebuffers = new_framebuffers;

            self.recreate_swapchain = false;
        }

        // acquire and image from the swapchain
        let (image_num, acquire_future) =
            match swapchain::acquire_next_image(self.swapchain.clone(), None) {
                Ok((n, f)) => (n, Some(f)),
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return;
                }
                Err(err) => panic!("{:?}", err),
            };

        let clear_color: [f32; 4] = clear_color.into();
        let clear_values = vec![clear_color.into(), 1f32.into()]; //ClearValue::DepthStencil((1f32, 1u32))];
        self.graphics_command_buffer = Some(
            AutoCommandBufferBuilder::primary_one_time_submit(
                self.device.clone(),
                self.queue.family(),
            )
            .unwrap()
            .begin_render_pass(self.framebuffers[image_num].clone(), false, clear_values)
            .unwrap(),
        );

        self.acquire_future = acquire_future;
        self.image_num = image_num;
    }

    pub fn draw(&mut self, pipeline_key: &Topology) {
        let set = {
            let data = vs::ty::Data {
                model: self.model_transform.transpose().into(),
                view: self.view_transform.transpose().into(),
                projection: self.projection_transform.transpose().into(),
            };

            let sub_buffer = self.uniform_buffer.next(data).unwrap();
            self.graphics_pool
                .get_mut(pipeline_key)
                .unwrap()
                .next()
                .add_buffer(sub_buffer)
                .unwrap()
                .build()
                .unwrap()
        };

        // @TODO FIXME Index into pipelines
        let pipeline = self.graphics_pipelines.get(pipeline_key).unwrap();
        self.graphics_command_buffer = {
            Some(
                self.graphics_command_buffer
                    .take()
                    .unwrap()
                    .draw(
                        pipeline.clone(),
                        &DynamicState::none(),
                        vec![self.vertex_buffer.clone()],
                        set,
                        (),
                    )
                    .unwrap(),
            )
        };
    }

    pub fn render(&mut self) {
        let graphics_command_buffer = self
            .graphics_command_buffer
            .take()
            .unwrap()
            .end_render_pass()
            .unwrap()
            .build()
            .unwrap();

        let prev = self.previous_frame_end.take();
        let acquire_future = self.acquire_future.take().unwrap();
        let future = prev
            .unwrap()
            .join(acquire_future)
            .then_execute(self.queue.clone(), graphics_command_buffer)
            .unwrap()
            .then_swapchain_present(self.queue.clone(), self.swapchain.clone(), self.image_num)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                // This wait is required when using NVIDIA or running on macOS.
                // See https://github.com/vulkano-rs/vulkano/issues/1247
                future.wait(None).unwrap();
                self.previous_frame_end = Some(Box::new(future) as Box<_>);
            }
            Err(FlushError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_end = Some(Box::new(sync::now(self.device.clone())) as Box<_>);
            }
            Err(e) => {
                println!("{:?}", e);
                self.previous_frame_end = Some(Box::new(sync::now(self.device.clone())) as Box<_>);
            }
        }
    }

    pub fn set_verts(&mut self, verts: &Vec<(Vec3, Vec3)>) {
        self.vertex_buffer = {
            CpuAccessibleBuffer::from_iter(
                self.device.clone(),
                BufferUsage::all(),
                verts_from_vec(verts).iter().cloned(),
            )
            .unwrap()
        };
    }
}

fn verts_from_vec(verts: &Vec<(Vec3, Vec3)>) -> Vec<Vertex> {
    verts
        .into_iter()
        .map(|(point, col)| Vertex {
            position: [point.x as f32, point.y as f32, point.z as f32],
            color: [col.x as f32, col.y as f32, col.z as f32],
        })
        .collect()
}

fn window_size_dependent_setup(
    device: &Arc<Device>,
    vs: &vs::Shader,
    fs: &fs::Shader,
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
) -> (
    HashMap<Topology, Arc<dyn GraphicsPipelineAbstract + Send + Sync>>,
    Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
) {
    let dimensions = images[0].dimensions();
    let depth_buffer =
        AttachmentImage::transient(device.clone(), dimensions, Format::D16Unorm).unwrap();
    let framebuffers = images
        .iter()
        .map(|image| {
            Arc::new(
                Framebuffer::start(render_pass.clone())
                    .add(image.clone())
                    .unwrap()
                    .add(depth_buffer.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
            ) as Arc<dyn FramebufferAbstract + Send + Sync>
        })
        .collect::<Vec<_>>();

    let pipelines = {
        let mut map: HashMap<Topology, Arc<dyn GraphicsPipelineAbstract + Send + Sync>> =
            HashMap::new();

        use std::u32;
        for top in Topology::iterator() {
            println!("NEW PIPELINE: {:#?}", top);
            let pipeline_builder = GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .primitive_topology(top.into());

            let pipeline_builder = match top.inner() {
                PolygonMode::Fill => pipeline_builder.polygon_mode_fill(),
                PolygonMode::Line => pipeline_builder.polygon_mode_line(),
                PolygonMode::Point => pipeline_builder.polygon_mode_point(),
            };

            let pipeline = Arc::new(
                pipeline_builder
                    .viewports_dynamic_scissors_irrelevant(1)
                    .viewports(iter::once(Viewport {
                        origin: [0.0, 0.0],
                        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                        depth_range: 0.0..1.0,
                    }))
                    .fragment_shader(fs.main_entry_point(), ())
                    .depth_stencil(DepthStencil {
                        depth_write: true,
                        depth_compare: Compare::Less,
                        depth_bounds_test: DepthBounds::Disabled,
                        stencil_front: Stencil {
                            compare: Compare::Never,
                            pass_op: StencilOp::Keep,
                            fail_op: StencilOp::Keep,
                            depth_fail_op: StencilOp::Keep,
                            compare_mask: Some(u32::MAX),
                            write_mask: Some(u32::MAX),
                            reference: Some(u32::MAX),
                        },
                        stencil_back: Stencil {
                            compare: Compare::Never,
                            pass_op: StencilOp::Keep,
                            fail_op: StencilOp::Keep,
                            depth_fail_op: StencilOp::Keep,
                            compare_mask: Some(u32::MAX),
                            write_mask: Some(u32::MAX),
                            reference: Some(u32::MAX),
                        },
                    })
                    .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                    .build(device.clone())
                    .unwrap(),
            );

            map.insert(*top, pipeline);
        }

        map
    };

    (pipelines, framebuffers)
}
