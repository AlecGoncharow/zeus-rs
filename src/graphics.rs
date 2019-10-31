use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, CpuBufferPool};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::{Device, DeviceExtensions};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass};
use vulkano::image::SwapchainImage;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::swapchain;
use vulkano::swapchain::{
    AcquireError, PresentMode, SurfaceTransform, Swapchain, SwapchainCreationError,
};
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture};

use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::format::Format;
use vulkano::image::Dimensions;
use vulkano::image::StorageImage;
use vulkano::pipeline::ComputePipeline;
use vulkano::sampler::Sampler;
use vulkano_win::VkSurfaceBuild;

use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use std::sync::Arc;

mod vs {
    vulkano_shaders::shader! {
            ty: "vertex",
            path: "src/vert.glsl"
    }
}

mod fs {
    vulkano_shaders::shader! {
            ty: "fragment",
            path: "src/frag.glsl"
    }
}

pub fn run_mandelbrot() {
    mod cs {
        vulkano_shaders::shader! {
            ty: "compute",
            path: "src/comp.glsl"
        }
    }

    let instance = {
        let extensions = vulkano_win::required_extensions();

        Instance::new(None, &extensions, None).unwrap()
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

    // windowing stuff
    let events_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&events_loop, instance.clone())
        .unwrap();
    let window = surface.window();

    // choose queue
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

    // take first queue out of queues, don't need the rest for now
    let queue = queues.next().unwrap();

    let (mut swapchain, images) = {
        let caps = surface.capabilities(physical).unwrap();

        let usage = caps.supported_usage_flags;
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();

        // Choosing the internal format that the images will have.
        let format = caps.supported_formats[0].0;
        // Because for both of these cases, the swapchain needs to be the window dimensions, we just use that.
        let initial_dimensions = {
            // convert to physical pixels
            let dimensions: (u32, u32) = window
                .inner_size()
                .to_physical(window.hidpi_factor())
                .into();
            [dimensions.0, dimensions.1]
        };

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

    #[derive(Default, Debug, Clone)]
    struct Vertex {
        position: [f32; 2],
    }
    vulkano::impl_vertex!(Vertex, position);
    let vertex_buffer = {
        CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            [
                Vertex {
                    position: [-1.0, -1.0],
                },
                Vertex {
                    position: [-1.0, 1.0],
                },
                Vertex {
                    position: [1.0, -1.0],
                },
                Vertex {
                    position: [1.0, 1.0],
                },
            ]
            .iter()
            .cloned(),
        )
        .unwrap()
    };

    let image = StorageImage::new(
        device.clone(),
        Dimensions::Dim2d {
            width: 1024,
            height: 1024,
        },
        Format::R8G8B8A8Unorm,
        Some(queue.family()),
    )
    .unwrap();

    let out_image = StorageImage::new(
        device.clone(),
        Dimensions::Dim2d {
            width: 1024,
            height: 1024,
        },
        Format::R8G8B8A8Unorm,
        Some(queue.family()),
    )
    .unwrap();

    let image_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        (0..1024 * 1024 * 4).map(|_| 0u8),
    )
    .expect("failed to create buffer");

    let sampler = Sampler::simple_repeat_linear_no_mipmap(device.clone());

    let vs = vs::Shader::load(device.clone()).unwrap();
    let fs = fs::Shader::load(device.clone()).unwrap();
    let cs = cs::Shader::load(device.clone()).unwrap();

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

    let compute_pipeline = Arc::new(
        ComputePipeline::new(device.clone(), &cs.main_entry_point(), &())
            .expect("failed to create compute pipeline"),
    );

    /*
    let set = Arc::new(
        PersistentDescriptorSet::start(compute_pipeline.clone(), 0)
            .add_image(image.clone())
            .unwrap()
            .build()
            .unwrap(),
    );
    */

    // build pipeline
    let graphics_pipeline = Arc::new(
        GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex>()
            // vert shader
            .vertex_shader(vs.main_entry_point(), ())
            .triangle_strip()
            .viewports_dynamic_scissors_irrelevant(1)
            // frag shader
            .fragment_shader(fs.main_entry_point(), ())
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(device.clone())
            .unwrap(),
    );

    let frag_set = Arc::new(
        PersistentDescriptorSet::start(graphics_pipeline.clone(), 0)
            .add_sampled_image(out_image.clone(), sampler.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    // Dynamic viewports allow us to recreate just the viewport when the window is resized
    // Otherwise we would have to recreate the whole pipeline.
    let mut dynamic_state = DynamicState {
        line_width: None,
        viewports: None,
        scissors: None,
    };

    let mut framebuffers =
        window_size_dependent_setup(&images, render_pass.clone(), &mut dynamic_state);

    // Initialization is finally finished!

    let mut recreate_swapchain = false;
    let mut previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<dyn GpuFuture>);

    let mut gran = 0.5;
    let data_buffer = CpuBufferPool::<cs::ty::Data>::new(device.clone(), BufferUsage::all());

    events_loop.run(move |ev, _, cf| {
        *cf = ControlFlow::Poll;
        let window = surface.window();

        previous_frame_end.as_mut().unwrap().cleanup_finished();

        // Whenever the window resizes we need to recreate everything dependent on the window size.
        // In this example that includes the swapchain, the framebuffers and the dynamic state viewport.
        if recreate_swapchain {
            // Get the new dimensions of the window.
            let dimensions = {
                let dimensions: (u32, u32) = window
                    .inner_size()
                    .to_physical(window.hidpi_factor())
                    .into();
                [dimensions.0, dimensions.1]
            };

            let (new_swapchain, new_images) = match swapchain.recreate_with_dimension(dimensions) {
                Ok(r) => r,
                // This error tends to happen when the user is manually resizing the window.
                // Simply restarting the loop is the easiest way to fix this issue.
                Err(SwapchainCreationError::UnsupportedDimensions) => return,
                Err(err) => panic!("{:?}", err),
            };

            swapchain = new_swapchain;
            // Because framebuffers contains an Arc on the old swapchain, we need to
            // recreate framebuffers as well.
            framebuffers =
                window_size_dependent_setup(&new_images, render_pass.clone(), &mut dynamic_state);

            recreate_swapchain = false;
        }

        // acquire and image from the swapchain
        let (image_num, acquire_future) =
            match swapchain::acquire_next_image(swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    recreate_swapchain = true;
                    return;
                }
                Err(err) => panic!("{:?}", err),
            };

        gran /= 1.01;
        gran = if gran <= 0.00005 { 0.5 } else { gran };
        println!("{:?}", gran);
        let comp_data = cs::ty::Data { granularity: gran };
        let sub_buffer = data_buffer.next(comp_data).unwrap();
        let set = Arc::new(
            PersistentDescriptorSet::start(compute_pipeline.clone(), 0)
                .add_image(image.clone())
                .unwrap()
                .add_buffer(sub_buffer)
                .unwrap()
                .build()
                .unwrap(),
        );

        let compute_command_buffer =
            AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family())
                .unwrap()
                .dispatch(
                    [1024 / 8, 1024 / 8, 1],
                    compute_pipeline.clone(),
                    set.clone(),
                    (),
                )
                .unwrap()
                .copy_image_to_buffer(image.clone(), image_buffer.clone())
                .unwrap()
                .copy_buffer_to_image(image_buffer.clone(), out_image.clone())
                .unwrap()
                .build()
                .unwrap();

        let clear_values = vec![[0.0, 0.0, 1.0, 1.0].into()];
        let graphics_command_buffer =
            AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family())
                .unwrap()
                .begin_render_pass(framebuffers[image_num].clone(), false, clear_values)
                .unwrap()
                .draw(
                    graphics_pipeline.clone(),
                    &dynamic_state,
                    vertex_buffer.clone(),
                    frag_set.clone(),
                    (),
                )
                .unwrap()
                .end_render_pass()
                .unwrap()
                .build()
                .unwrap();

        // wait for previous fram to end and then pass in new commands
        let prev = previous_frame_end.take();

        let future = prev
            .unwrap()
            .join(acquire_future)
            .then_execute(queue.clone(), compute_command_buffer)
            .unwrap()
            .then_execute(queue.clone(), graphics_command_buffer)
            .unwrap()
            .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                // This wait is required when using NVIDIA or running on macOS.
                // See https://github.com/vulkano-rs/vulkano/issues/1247
                future.wait(None).unwrap();
                previous_frame_end = Some(Box::new(future) as Box<_>);
            }
            Err(FlushError::OutOfDate) => {
                recreate_swapchain = true;
                previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<_>);
            }
            Err(e) => {
                println!("{:?}", e);
                previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<_>);
            }
        }

        match ev {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *cf = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => recreate_swapchain = true,
            _ => {}
        }
    });
}

pub fn run_ray() {
    mod cs {
        vulkano_shaders::shader! {
            ty: "compute",
            path: "src/ray_comp.glsl"
        }
    }

    let height = 400;
    let width = 800;

    let instance = {
        let extensions = vulkano_win::required_extensions();

        Instance::new(None, &extensions, None).unwrap()
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

    // windowing stuff
    let events_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&events_loop, instance.clone())
        .unwrap();
    let window = surface.window();

    // choose queue
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

    // take first queue out of queues, don't need the rest for now
    let queue = queues.next().unwrap();

    let (mut swapchain, images) = {
        let caps = surface.capabilities(physical).unwrap();

        let usage = caps.supported_usage_flags;
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();

        // Choosing the internal format that the images will have.
        let format = caps.supported_formats[0].0;
        // Because for both of these cases, the swapchain needs to be the window dimensions, we just use that.
        let initial_dimensions = {
            // convert to physical pixels
            let dimensions: (u32, u32) = window
                .inner_size()
                .to_physical(window.hidpi_factor())
                .into();
            [dimensions.0, dimensions.1]
        };

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

    #[derive(Default, Debug, Clone)]
    struct Vertex {
        position: [f32; 2],
    }
    vulkano::impl_vertex!(Vertex, position);
    let vertex_buffer = {
        CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            [
                Vertex {
                    position: [-1.0, -1.0],
                },
                Vertex {
                    position: [-1.0, 1.0],
                },
                Vertex {
                    position: [1.0, -1.0],
                },
                Vertex {
                    position: [1.0, 1.0],
                },
            ]
            .iter()
            .cloned(),
        )
        .unwrap()
    };

    let image = StorageImage::new(
        device.clone(),
        Dimensions::Dim2d {
            width: width,
            height: height,
        },
        Format::R8G8B8A8Unorm,
        Some(queue.family()),
    )
    .unwrap();

    let out_image = StorageImage::new(
        device.clone(),
        Dimensions::Dim2d {
            width: width,
            height: height,
        },
        Format::R8G8B8A8Unorm,
        Some(queue.family()),
    )
    .unwrap();

    let image_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        (0..height * width * 4).map(|_| 0u8),
    )
    .expect("failed to create buffer");

    let sampler = Sampler::simple_repeat_linear_no_mipmap(device.clone());

    let vs = vs::Shader::load(device.clone()).unwrap();
    let fs = fs::Shader::load(device.clone()).unwrap();
    let cs = cs::Shader::load(device.clone()).unwrap();

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

    let compute_pipeline = Arc::new(
        ComputePipeline::new(device.clone(), &cs.main_entry_point(), &())
            .expect("failed to create compute pipeline"),
    );

    use cs::ty::Sphere;

    let sphere_buffer = {
        CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            [
                Sphere {
                    center: [0.0, 0.0, -1.0],
                    radius: 0.5,
                },
                Sphere {
                    center: [0.0, -100.5, -1.0],
                    radius: 100.0,
                },
            ]
            .iter()
            .cloned(),
        )
        .unwrap()
    };

    let set = Arc::new(
        PersistentDescriptorSet::start(compute_pipeline.clone(), 0)
            .add_image(image.clone())
            .unwrap()
            .add_buffer(sphere_buffer)
            .unwrap()
            .build()
            .unwrap(),
    );

    // build pipeline
    let graphics_pipeline = Arc::new(
        GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex>()
            // vert shader
            .vertex_shader(vs.main_entry_point(), ())
            .triangle_strip()
            .viewports_dynamic_scissors_irrelevant(1)
            // frag shader
            .fragment_shader(fs.main_entry_point(), ())
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(device.clone())
            .unwrap(),
    );

    let frag_set = Arc::new(
        PersistentDescriptorSet::start(graphics_pipeline.clone(), 0)
            .add_sampled_image(out_image.clone(), sampler.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    // Dynamic viewports allow us to recreate just the viewport when the window is resized
    // Otherwise we would have to recreate the whole pipeline.
    let mut dynamic_state = DynamicState {
        line_width: None,
        viewports: None,
        scissors: None,
    };

    let mut framebuffers =
        window_size_dependent_setup(&images, render_pass.clone(), &mut dynamic_state);

    // Initialization is finally finished!

    let mut recreate_swapchain = false;
    let mut previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<dyn GpuFuture>);

    events_loop.run(move |ev, _, cf| {
        *cf = ControlFlow::Poll;
        let window = surface.window();

        previous_frame_end.as_mut().unwrap().cleanup_finished();

        // Whenever the window resizes we need to recreate everything dependent on the window size.
        // In this example that includes the swapchain, the framebuffers and the dynamic state viewport.
        if recreate_swapchain {
            // Get the new dimensions of the window.
            let dimensions = {
                let dimensions: (u32, u32) = window
                    .inner_size()
                    .to_physical(window.hidpi_factor())
                    .into();
                [dimensions.0, dimensions.1]
            };

            let (new_swapchain, new_images) = match swapchain.recreate_with_dimension(dimensions) {
                Ok(r) => r,
                // This error tends to happen when the user is manually resizing the window.
                // Simply restarting the loop is the easiest way to fix this issue.
                Err(SwapchainCreationError::UnsupportedDimensions) => return,
                Err(err) => panic!("{:?}", err),
            };

            swapchain = new_swapchain;
            // Because framebuffers contains an Arc on the old swapchain, we need to
            // recreate framebuffers as well.
            framebuffers =
                window_size_dependent_setup(&new_images, render_pass.clone(), &mut dynamic_state);

            recreate_swapchain = false;
        }

        // acquire and image from the swapchain
        let (image_num, acquire_future) =
            match swapchain::acquire_next_image(swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    recreate_swapchain = true;
                    return;
                }
                Err(err) => panic!("{:?}", err),
            };

        let compute_command_buffer =
            AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family())
                .unwrap()
                .dispatch(
                    [width / 8, height / 8, 1],
                    compute_pipeline.clone(),
                    set.clone(),
                    (),
                )
                .unwrap()
                .copy_image_to_buffer(image.clone(), image_buffer.clone())
                .unwrap()
                .copy_buffer_to_image(image_buffer.clone(), out_image.clone())
                .unwrap()
                .build()
                .unwrap();

        let clear_values = vec![[0.0, 0.0, 1.0, 1.0].into()];
        let graphics_command_buffer =
            AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family())
                .unwrap()
                .begin_render_pass(framebuffers[image_num].clone(), false, clear_values)
                .unwrap()
                .draw(
                    graphics_pipeline.clone(),
                    &dynamic_state,
                    vertex_buffer.clone(),
                    frag_set.clone(),
                    (),
                )
                .unwrap()
                .end_render_pass()
                .unwrap()
                .build()
                .unwrap();

        // wait for previous fram to end and then pass in new commands
        let prev = previous_frame_end.take();

        let future = prev
            .unwrap()
            .join(acquire_future)
            .then_execute(queue.clone(), compute_command_buffer)
            .unwrap()
            .then_execute(queue.clone(), graphics_command_buffer)
            .unwrap()
            .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                // This wait is required when using NVIDIA or running on macOS.
                // See https://github.com/vulkano-rs/vulkano/issues/1247
                future.wait(None).unwrap();
                previous_frame_end = Some(Box::new(future) as Box<_>);
            }
            Err(FlushError::OutOfDate) => {
                recreate_swapchain = true;
                previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<_>);
            }
            Err(e) => {
                println!("{:?}", e);
                previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<_>);
            }
        }

        match ev {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *cf = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => recreate_swapchain = true,
            _ => {}
        }
    });
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
