use std::usize;

use winit::window::Window;

use crate::graphics::prelude::*;

pub struct GraphicsContext {
    pub size: winit::dpi::PhysicalSize<u32>,
    pub clear_color: wgpu::Color,
    pub window_dims: winit::dpi::PhysicalSize<f32>,
}

impl GraphicsContext {
    pub fn new(window: &Window, clear_color: crate::math::Vec4) -> Self {
        let size = window.inner_size();

        let clear_color = wgpu::Color {
            r: clear_color.x.into(),
            g: clear_color.y.into(),
            b: clear_color.z.into(),
            a: clear_color.w.into(),
        };
        let window_dims = window.inner_size().cast::<f32>();

        Self {
            size,
            clear_color,
            window_dims,
        }
    }

    pub fn resize(
        &mut self,
        new_size: winit::dpi::PhysicalSize<u32>,
        window: &winit::window::Window,
    ) {
        self.size = new_size;
        self.window_dims = window.inner_size().cast::<f32>();
    }

    pub fn render<'a>(
        &mut self,
        wrangler: &RenderWrangler<'a>,
        //passes: &[Pass<'a>],
        device: &wgpu::Device,
        output: wgpu::SurfaceTexture,
        queue: &wgpu::Queue,
    ) {
        let view = &output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let mut pass_mask = 1;
        for pass in &wrangler.passes {
            encoder.push_debug_group(pass.label);

            let attachment;
            let color_attachments: &[wgpu::RenderPassColorAttachment] =
                if let Some(ops) = pass.color_attachment_ops {
                    let attach_view = if let Some(handle) = &pass.color_attachment_view_handle {
                        &wrangler.get_texture(handle).view
                    } else {
                        view
                    };
                    attachment = [wgpu::RenderPassColorAttachment {
                        view: attach_view,
                        resolve_target: None,
                        ops,
                    }];
                    &attachment
                } else {
                    &[]
                };

            let depth_stencil_attachment = if let Some(handle) = &pass.depth_stencil_view_handle {
                let view = &wrangler.get_texture(handle).view;
                Some(wgpu::RenderPassDepthStencilAttachment {
                    view,
                    depth_ops: pass.depth_ops,
                    stencil_ops: pass.stencil_ops,
                })
            } else {
                None
            };

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(pass.label),
                color_attachments,
                depth_stencil_attachment,
            });

            render_pass.set_bind_group(
                0,
                wrangler.get_bind_group(&wrangler.frame_bind_group_handle),
                &[],
            );
            if let Some(pass_bind_group_handle) = &pass.pass_bind_group_handle {
                render_pass.set_bind_group(1, wrangler.get_bind_group(pass_bind_group_handle), &[]);
            }

            let index_buffer = wrangler.get_index_buffer(&pass.index_buffer_handle);

            let vertex_buffer = wrangler.get_vertex_buffer(&pass.vertex_buffer_handle);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));

            for LabeledEntry { entry: call, .. } in wrangler
                .draw_calls
                .iter()
                .filter(|entry| (entry.entry.pass_flags & pass_mask) != 0)
            {
                if let Some(handle) = &call.bind_group_handle {
                    render_pass.set_bind_group(2, wrangler.get_bind_group(handle), &[]);
                }
                render_pass.set_pipeline(&pass.pipelines[usize::from(call.topology)]);
                if let Some(push_constant) = &call.push_constant {
                    render_pass.set_push_constants(
                        push_constant.stages,
                        push_constant.offset,
                        &push_constant.data,
                    );
                }

                match call.kind {
                    DrawCallKind::Vertex => {
                        // @TODO @SPEED revisit this, Range does not impl Copy, is it better to keep
                        // params of range in data and to just create on fly vs clone?
                        // See: https://github.com/rust-lang/rust/pull/27186
                        render_pass.draw(call.index_range.clone(), call.instances.clone());
                    }
                    DrawCallKind::Indexed(base_vertex) => {
                        render_pass
                            .set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                        render_pass.draw_indexed(
                            call.index_range.clone(),
                            base_vertex,
                            call.instances.clone(),
                        );
                    }
                }
            }
            drop(render_pass);
            encoder.pop_debug_group();

            pass_mask <<= 1;
        }

        queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
