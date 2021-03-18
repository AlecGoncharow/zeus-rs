use crate::context::EngineEvent;

const OUTPUT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

/// This is the repaint signal type that egui needs for requesting a repaint from another thread.
/// It sends the custom RequestRedraw event to the winit event loop.
pub struct RepaintSignal(std::sync::Mutex<winit::event_loop::EventLoopProxy<EngineEvent>>);

impl epi::RepaintSignal for RepaintSignal {
    fn request_repaint(&self) {
        self.0
            .lock()
            .unwrap()
            .send_event(EngineEvent::RequestRedraw)
            .ok();
    }
}

pub struct UiContext {
    pub render_pass: egui_wgpu_backend::RenderPass,
    pub platform: egui_winit_platform::Platform,
    previous_frame_time: Option<f32>,
    pub repaint_signal: std::sync::Arc<RepaintSignal>,
}

impl UiContext {
    pub fn new(
        device: &wgpu::Device,
        window: &winit::window::Window,
        event_loop_proxy: std::sync::Mutex<winit::event_loop::EventLoopProxy<EngineEvent>>,
    ) -> Self {
        // We use the egui_winit_platform crate as the platform.
        let size = window.inner_size();
        let platform =
            egui_winit_platform::Platform::new(egui_winit_platform::PlatformDescriptor {
                physical_width: size.width as u32,
                physical_height: size.height as u32,
                scale_factor: window.scale_factor(),
                font_definitions: egui::FontDefinitions::default(),
                style: Default::default(),
            });

        // We use the egui_wgpu_backend crate as the render backend.
        let render_pass = egui_wgpu_backend::RenderPass::new(&device, OUTPUT_FORMAT);
        let repaint_signal = std::sync::Arc::new(RepaintSignal(event_loop_proxy));
        Self {
            platform,
            render_pass,
            previous_frame_time: None,
            repaint_signal,
        }
    }

    pub fn draw(
        &mut self,
        mut command_encoder: wgpu::CommandEncoder,
        window: &winit::window::Window,
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        sc_frame: &wgpu::SwapChainFrame,
        queue: &mut wgpu::Queue,
    ) -> wgpu::CommandEncoder {
        let egui_start = std::time::Instant::now();
        self.platform.begin_frame();
        let mut app_output = epi::backend::AppOutput::default();

        let _frame = epi::backend::FrameBuilder {
            info: epi::IntegrationInfo {
                web_info: None,
                cpu_usage: self.previous_frame_time,
                seconds_since_midnight: None,
                native_pixels_per_point: Some(window.scale_factor() as _),
            },
            tex_allocator: &mut self.render_pass,
            output: &mut app_output,
            repaint_signal: self.repaint_signal.clone(),
        }
        .build();
        // Draw the demo application.
        //self.demo_app.update(&self.platform.context(), &mut frame);

        // End the UI frame. We could now handle the output and draw the UI with the backend.
        let (_output, paint_commands) = self.platform.end_frame();
        let paint_jobs = self.platform.context().tessellate(paint_commands);

        let frame_time = (std::time::Instant::now() - egui_start).as_secs_f64() as f32;
        self.previous_frame_time = Some(frame_time);

        // Upload all resources for the GPU.
        let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
            physical_width: sc_desc.width,
            physical_height: sc_desc.height,
            scale_factor: window.scale_factor() as f32,
        };
        self.render_pass
            .update_texture(device, queue, &self.platform.context().texture());
        self.render_pass.update_user_textures(&device, &queue);
        self.render_pass
            .update_buffers(device, queue, &paint_jobs, &screen_descriptor);

        // Record all render passes.
        self.render_pass.execute(
            &mut command_encoder,
            &sc_frame.output.view,
            &paint_jobs,
            &screen_descriptor,
            None,
        );
        command_encoder
    }
}
