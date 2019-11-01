use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass};
use vulkano::image::SwapchainImage;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::swapchain;
use vulkano::swapchain::{
    AcquireError, PresentMode, Surface, SurfaceTransform, Swapchain, SwapchainCreationError,
};
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture};

use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano_win::VkSurfaceBuild;

use winit::dpi::LogicalSize;
use winit::EventsLoop;
use winit::{Window, WindowBuilder};

use std::sync::Arc;

use crate::math::Vec3;

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

    graphics_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    dynamic_state: DynamicState,
    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,

    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    window_dims: LogicalSize,
}

#[derive(Default, Debug, Clone)]
struct Vertex {
    position: [f32; 3],
}
vulkano::impl_vertex!(Vertex, position);

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

        let surface = WindowBuilder::new()
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

        let vs = vs::Shader::load(device.clone()).unwrap();
        let fs = fs::Shader::load(device.clone()).unwrap();

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
                    }

                },
                pass: {
                    color: [color],

                    depth_stencil: {}
                }
            )
            .unwrap(),
        );

        let graphics_pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                // vert shader
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                // frag shader
                .fragment_shader(fs.main_entry_point(), ())
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(device.clone())
                .unwrap(),
        );

        // Dynamic viewports allow us to recreate just the viewport when the window is resized
        // Otherwise we would have to recreate the whole pipeline.
        let mut dynamic_state = DynamicState {
            line_width: None,
            viewports: None,
            scissors: None,
            compare_mask: None,
            write_mask: None,
            reference: None,
        };

        let framebuffers =
            window_size_dependent_setup(&images, render_pass.clone(), &mut dynamic_state);

        let recreate_swapchain = false;
        let previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<dyn GpuFuture>);
        GraphicsContext {
            recreate_swapchain,
            previous_frame_end,

            swapchain,
            queue,
            device,
            surface,

            graphics_pipeline,
            render_pass,
            dynamic_state,
            framebuffers,
            vertex_buffer,
            window_dims,
        }
    }

    pub fn render(&mut self) {
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
            self.framebuffers = window_size_dependent_setup(
                &new_images,
                self.render_pass.clone(),
                &mut self.dynamic_state,
            );

            self.recreate_swapchain = false;
        }

        // acquire and image from the swapchain
        let (image_num, acquire_future) =
            match swapchain::acquire_next_image(self.swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return;
                }
                Err(err) => panic!("{:?}", err),
            };

        let clear_values = vec![[0.0, 0.0, 1.0, 1.0].into()];
        let graphics_command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(
            self.device.clone(),
            self.queue.family(),
        )
        .unwrap()
        .begin_render_pass(self.framebuffers[image_num].clone(), false, clear_values)
        .unwrap()
        .draw(
            self.graphics_pipeline.clone(),
            &self.dynamic_state,
            //@TODO fix
            vec![self.vertex_buffer.clone()],
            (),
            (),
        )
        .unwrap()
        .end_render_pass()
        .unwrap()
        .build()
        .unwrap();

        let prev = self.previous_frame_end.take();

        let future = prev
            .unwrap()
            .join(acquire_future)
            .then_execute(self.queue.clone(), graphics_command_buffer)
            .unwrap()
            .then_swapchain_present(self.queue.clone(), self.swapchain.clone(), image_num)
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

    pub fn set_verts(&mut self, verts: &Vec<Vec3>) {
        self.vertex_buffer = {
            CpuAccessibleBuffer::from_iter(
                self.device.clone(),
                BufferUsage::all(),
                verts_from_vec(normalize_vec_to_ndc(verts, self.window_dims))
                    .iter()
                    .cloned(),
            )
            .unwrap()
        };
    }
}

fn normalize_vec_to_ndc(verts: &Vec<Vec3>, dims: LogicalSize) -> Vec<Vec3> {
    let max_x = dims.width as f32;
    let max_y = dims.height as f32;

    let mut ret = vec![];

    for vec in verts {
        println!("{:#?}", vec);
        let mut x = vec.x / max_x;
        let mut y = vec.y / max_y;
        let z = vec.z;

        x = 2.0 * x - 1.0;
        y = 2.0 * y - 1.0;

        let point = Vec3 { x, y, z };
        println!("{:#?}", point);
        ret.push(point);
    }

    ret
}

fn verts_from_vec(verts: Vec<Vec3>) -> Vec<Vertex> {
    verts
        .into_iter()
        .map(|point| Vertex {
            position: [point.x, point.y, point.z],
        })
        .collect()
}

fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    dynamic_state: &mut DynamicState,
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
    let dimensions = images[0].dimensions();

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0..1.0,
    };
    dynamic_state.viewports = Some(vec![viewport]);

    images
        .iter()
        .map(|image| {
            Arc::new(
                Framebuffer::start(render_pass.clone())
                    .add(image.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
            ) as Arc<dyn FramebufferAbstract + Send + Sync>
        })
        .collect::<Vec<_>>()
}
