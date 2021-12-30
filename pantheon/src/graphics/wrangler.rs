use std::marker::PhantomData;

use super::prelude::*;
use crate::graphics::pass::*;
use crate::shader::ShaderContext;

pub struct RenderWrangler<'a> {
    pub passes: Vec<Pass<'a>>,

    //@TODO swap LabeledEntry for PhantomData enforced types
    pub bind_group_layouts: Vec<LabeledEntry<'a, wgpu::BindGroupLayout>>,
    pub bind_groups: Vec<LabeledEntry<'a, wgpu::BindGroup>>,
    pub vertex_buffers: Vec<LabeledEntry<'a, wgpu::Buffer>>,
    pub vertex_buffer_cursors: Vec<LabeledEntry<'a, wgpu::BufferAddress>>,
    pub index_buffers: Vec<LabeledEntry<'a, wgpu::Buffer>>,
    pub index_buffer_cursors: Vec<LabeledEntry<'a, wgpu::BufferAddress>>,
    pub uniform_buffers: Vec<LabeledEntry<'a, wgpu::Buffer>>,
    pub textures: Vec<LabeledEntry<'a, Texture>>,
    pub draw_calls: Vec<LabeledEntry<'a, DrawCall<'a>>>,
}
impl<'a> RenderWrangler<'a> {
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
            bind_group_layouts: Vec::new(),
            bind_groups: Vec::new(),
            vertex_buffers: Vec::new(),
            vertex_buffer_cursors: Vec::new(),
            index_buffers: Vec::new(),
            index_buffer_cursors: Vec::new(),
            uniform_buffers: Vec::new(),
            textures: Vec::new(),
            draw_calls: Vec::new(),
        }
    }

    /**
     * Bind Group Layouts
     */

    /// unchecked get in release mode, label equality asserted on in debug
    pub fn get_bind_group_layout(&self, handle: &BindGroupLayoutHandle) -> &wgpu::BindGroupLayout {
        let bind_group_layout = &self.bind_group_layouts[handle.idx];
        #[cfg(debug_assertions)]
        assert_eq!(handle.label, bind_group_layout.label);
        &bind_group_layout.entry
    }

    /// checked add, if another entry exists with same label, a handle to that is returned
    /// instead, this might be a footgun
    pub fn add_bind_group_layout(
        &mut self,
        bind_group_layout: wgpu::BindGroupLayout,
        label: &'a str,
    ) -> BindGroupLayoutHandle<'a> {
        if let Some(handle) = self.handle_to_bind_group_layout(label) {
            handle
        } else {
            let idx = self.bind_group_layouts.len();
            self.bind_group_layouts.push(LabeledEntry {
                label,
                entry: bind_group_layout,
            });
            BindGroupLayoutHandle {
                label,
                idx,
                marker: PhantomData,
            }
        }
    }

    pub fn handle_to_bind_group_layout(&self, label: &'a str) -> Option<BindGroupLayoutHandle<'a>> {
        let idx = self
            .bind_group_layouts
            .iter()
            .position(|entry| entry.label == label)?;

        Some(BindGroupLayoutHandle {
            idx,
            label,
            marker: PhantomData,
        })
    }

    /**
     *  Bind Groups
     */

    pub fn get_bind_group(&self, handle: &BindGroupHandle) -> &wgpu::BindGroup {
        let bind_group = &self.bind_groups[handle.idx];
        #[cfg(debug_assertions)]
        assert_eq!(handle.label, bind_group.label);
        &bind_group.entry
    }

    pub fn add_bind_group(
        &mut self,
        bind_group: wgpu::BindGroup,
        label: &'a str,
    ) -> BindGroupHandle<'a> {
        if let Some(handle) = self.handle_to_bind_group(label) {
            handle
        } else {
            let idx = self.bind_groups.len();
            self.bind_groups.push(LabeledEntry {
                label,
                entry: bind_group,
            });
            BindGroupHandle {
                label,
                idx,
                marker: PhantomData,
            }
        }
    }

    pub fn handle_to_bind_group(&self, label: &'a str) -> Option<BindGroupHandle<'a>> {
        let idx = self
            .bind_groups
            .iter()
            .position(|entry| entry.label == label)?;

        Some(BindGroupHandle {
            idx,
            label,
            marker: PhantomData,
        })
    }

    /**
     * Vertex buffers
     */

    pub fn get_vertex_buffer(&self, handle: &BufferHandle) -> &wgpu::Buffer {
        let buffer = &self.vertex_buffers[handle.idx];
        #[cfg(debug_assertions)]
        assert_eq!(handle.label, buffer.label);
        &buffer.entry
    }

    pub fn add_vertex_buffer(
        &mut self,
        vertex_buffer: wgpu::Buffer,
        label: &'a str,
    ) -> BufferHandle<'a> {
        if let Some(handle) = self.handle_to_vertex_buffer(label) {
            handle
        } else {
            let idx = self.vertex_buffers.len();
            self.vertex_buffers.push(LabeledEntry {
                label,
                entry: vertex_buffer,
            });
            self.vertex_buffer_cursors
                .push(LabeledEntry { label, entry: 0 });
            BufferHandle {
                label,
                idx,
                marker: PhantomData,
            }
        }
    }

    pub fn handle_to_vertex_buffer(&self, label: &'a str) -> Option<BufferHandle<'a>> {
        let idx = self
            .vertex_buffers
            .iter()
            .position(|entry| entry.label == label)?;

        Some(BufferHandle {
            idx,
            label,
            marker: PhantomData,
        })
    }

    pub fn get_vertex_buffer_cursor(&self, handle: &BufferHandle) -> &wgpu::BufferAddress {
        let cursor = &self.vertex_buffer_cursors[handle.idx];
        #[cfg(debug_assertions)]
        assert_eq!(handle.label, cursor.label);
        &cursor.entry
    }

    pub fn handle_to_vertex_buffer_cursor(&self, label: &'a str) -> Option<BufferHandle<'a>> {
        let idx = self
            .vertex_buffer_cursors
            .iter()
            .position(|entry| entry.label == label)?;

        Some(BufferHandle {
            idx,
            label,
            marker: PhantomData,
        })
    }

    pub fn swap_vertex_buffer_cursor(&mut self, handle: BufferHandle, cursor: wgpu::BufferAddress) {
        let c = &mut self.vertex_buffer_cursors[handle.idx];
        #[cfg(debug_assertions)]
        assert_eq!(handle.label, c.label);
        c.entry = cursor;
    }

    /**
     * Index buffers
     */

    pub fn get_index_buffer(&self, handle: &BufferHandle) -> &wgpu::Buffer {
        let buffer = &self.index_buffers[handle.idx];
        #[cfg(debug_assertions)]
        assert_eq!(handle.label, buffer.label);
        &buffer.entry
    }

    pub fn add_index_buffer(
        &mut self,
        index_buffer: wgpu::Buffer,
        label: &'a str,
    ) -> BufferHandle<'a> {
        if let Some(handle) = self.handle_to_index_buffer(label) {
            handle
        } else {
            let idx = self.index_buffers.len();
            self.index_buffers.push(LabeledEntry {
                label,
                entry: index_buffer,
            });
            self.index_buffer_cursors
                .push(LabeledEntry { label, entry: 0 });
            BufferHandle {
                label,
                idx,
                marker: PhantomData,
            }
        }
    }

    pub fn handle_to_index_buffer(&self, label: &'a str) -> Option<BufferHandle<'a>> {
        let idx = self
            .index_buffers
            .iter()
            .position(|entry| entry.label == label)?;

        Some(BufferHandle {
            idx,
            label,
            marker: PhantomData,
        })
    }

    pub fn get_index_buffer_cursor(&self, handle: &BufferHandle<'a>) -> &wgpu::BufferAddress {
        let cursor = &self.index_buffer_cursors[handle.idx];
        #[cfg(debug_assertions)]
        assert_eq!(handle.label, cursor.label);
        &cursor.entry
    }

    pub fn handle_to_index_buffer_cursor(&self, label: &'a str) -> Option<BufferHandle<'a>> {
        let idx = self
            .index_buffer_cursors
            .iter()
            .position(|entry| entry.label == label)?;

        Some(BufferHandle {
            idx,
            label,
            marker: PhantomData,
        })
    }

    pub fn swap_index_buffer_cursor(&mut self, handle: BufferHandle, cursor: wgpu::BufferAddress) {
        let c = &mut self.index_buffer_cursors[handle.idx];
        #[cfg(debug_assertions)]
        assert_eq!(handle.label, c.label);
        c.entry = cursor;
    }

    /**
     * Uniform buffers
     */

    pub fn get_uniform_buffer(&self, handle: &BufferHandle) -> &wgpu::Buffer {
        let uniform_buffer = &self.uniform_buffers[handle.idx];
        #[cfg(debug_assertions)]
        assert_eq!(handle.label, uniform_buffer.label);
        &uniform_buffer.entry
    }
    pub fn add_uniform_buffer(
        &mut self,
        uniform_buffer: wgpu::Buffer,
        label: &'a str,
    ) -> BufferHandle<'a> {
        if let Some(handle) = self.handle_to_uniform_buffer(label) {
            handle
        } else {
            let idx = self.uniform_buffers.len();
            self.uniform_buffers.push(LabeledEntry {
                label,
                entry: uniform_buffer,
            });
            BufferHandle {
                label,
                idx,
                marker: PhantomData,
            }
        }
    }

    pub fn handle_to_uniform_buffer(&self, label: &'a str) -> Option<BufferHandle<'a>> {
        let idx = self
            .uniform_buffers
            .iter()
            .position(|entry| entry.label == label)?;

        Some(BufferHandle {
            idx,
            label,
            marker: PhantomData,
        })
    }

    /**
     * Draw Texture
     */

    pub fn get_texture(&self, handle: &TextureHandle<'a>) -> &super::texture::Texture {
        let texture = &self.textures[handle.idx];
        #[cfg(debug_assertions)]
        assert_eq!(handle.label, texture.label);
        &texture.entry
    }

    pub fn add_texture(&mut self, texture: Texture, label: &'a str) -> TextureHandle<'a> {
        if let Some(handle) = self.handle_to_texture(label) {
            handle
        } else {
            let idx = self.textures.len();
            self.textures.push(LabeledEntry {
                label,
                entry: texture,
            });
            TextureHandle {
                label,
                idx,
                marker: PhantomData,
            }
        }
    }

    pub fn swap_texture(&mut self, handle: &TextureHandle, texture: Texture) {
        let entry = &mut self.textures[handle.idx];
        #[cfg(debug_assertions)]
        assert_eq!(handle.label, entry.label);
        entry.entry = texture;
    }

    pub fn handle_to_texture(&self, label: &'a str) -> Option<TextureHandle<'a>> {
        let idx = self
            .textures
            .iter()
            .position(|entry| entry.label == label)?;

        Some(TextureHandle {
            idx,
            label,
            marker: PhantomData,
        })
    }

    /**
     * Draw Call
     */

    pub fn get_draw_call(&self, handle: &DrawCallHandle<'a>) -> &DrawCall {
        let draw_call = &self.draw_calls[handle.idx];
        #[cfg(debug_assertions)]
        assert_eq!(handle.label, draw_call.label);
        &draw_call.entry
    }

    pub fn get_draw_call_mut(&mut self, handle: &DrawCallHandle<'a>) -> &mut DrawCall<'a> {
        let draw_call = &mut self.draw_calls[handle.idx];
        #[cfg(debug_assertions)]
        assert_eq!(handle.label, draw_call.label);
        &mut draw_call.entry
    }

    /// This is an unchecked add, you should keep this handle as there is no guarentee the label
    /// is unique
    pub fn add_draw_call(&mut self, draw_call: DrawCall<'a>, label: &'a str) -> DrawCallHandle<'a> {
        //@TODO think about if this ought to be unique
        let idx = self.draw_calls.len();
        self.draw_calls.push(LabeledEntry {
            label,
            entry: draw_call,
        });
        DrawCallHandle {
            label,
            idx,
            marker: PhantomData,
        }
    }

    pub fn swap_draw_call(&mut self, handle: &DrawCallHandle, draw_call: DrawCall<'a>) {
        let entry = &mut self.draw_calls[handle.idx];
        #[cfg(debug_assertions)]
        assert_eq!(handle.label, entry.label);
        entry.entry = draw_call;
    }

    pub fn handle_to_draw_call(&self, label: &'a str) -> Option<DrawCallHandle<'a>> {
        let idx = self
            .draw_calls
            .iter()
            .position(|entry| entry.label == label)?;

        Some(DrawCallHandle {
            idx,
            label,
            marker: PhantomData,
        })
    }

    /**
     * Pass
     */

    pub fn get_pass(&self, handle: &PassHandle) -> &Pass<'a> {
        let pass = &self.passes[handle.idx];
        #[cfg(debug_assertions)]
        assert_eq!(handle.label, pass.label);
        pass
    }

    pub fn get_pass_mut(&mut self, handle: &PassHandle) -> &mut Pass<'a> {
        let pass = &mut self.passes[handle.idx];
        #[cfg(debug_assertions)]
        assert_eq!(handle.label, pass.label);
        pass
    }

    pub fn add_pass(&mut self, pass: Pass<'a>, label: &'a str) -> PassHandle<'a> {
        if let Some(handle) = self.handle_to_pass(label) {
            handle
        } else {
            let idx = self.passes.len();
            self.passes.push(pass);
            PassHandle {
                label,
                idx,
                marker: PhantomData,
            }
        }
    }

    pub fn handle_to_pass(&self, label: &'a str) -> Option<PassHandle<'a>> {
        let idx = self.passes.iter().position(|entry| entry.label == label)?;

        Some(PassHandle {
            idx,
            label,
            marker: PhantomData,
        })
    }

    /**
     *Push Constants

    pub fn get_push_constant(&self, handle: &PushConstantHandle<'a>) -> &PushConstant {
        let push_constant = &self.push_constants[handle.idx];
        #[cfg(debug_assertions)]
        assert_eq!(handle.label, push_constant.label);
        &push_constant.entry
    }

    pub fn get_push_constant_mut(&mut self, handle: &PushConstantHandle<'a>) -> &mut PushConstant {
        let push_constant = &mut self.push_constants[handle.idx];
        #[cfg(debug_assertions)]
        assert_eq!(handle.label, push_constant.label);
        &mut push_constant.entry
    }

    /// This is an unchecked add, you should keep this handle as there is no guarentee the label
    /// is unique
    pub fn add_push_constant(
        &mut self,
        push_constant: PushConstant,
        label: &'a str,
    ) -> PushConstantHandle<'a> {
        //@TODO think about if this ought to be unique
        let idx = self.push_constants.len();
        self.push_constants.push(LabeledEntry {
            label,
            entry: push_constant,
        });
        println!("[add_push_constant] idx: {}, label {}", idx, label);
        PushConstantHandle {
            label,
            idx,
            marker: PhantomData,
        }
    }

    pub fn swap_push_constant(&mut self, handle: &PushConstantHandle, push_constant: PushConstant) {
        let entry = &mut self.push_constants[handle.idx];
        #[cfg(debug_assertions)]
        assert_eq!(handle.label, entry.label);
        entry.entry = push_constant;
    }

    pub fn handle_to_push_constant(&self, label: &'a str) -> Option<PushConstantHandle<'a>> {
        let idx = self
            .push_constants
            .iter()
            .position(|entry| entry.label == label)?;

        Some(PushConstantHandle {
            idx,
            label,
            marker: PhantomData,
        })
    }
     */

    /// sugar functions, inefficient and should only be used if i know what im doing :)

    pub fn find_vertex_buffer(&self, label: &'a str) -> &wgpu::Buffer {
        &self
            .vertex_buffers
            .iter()
            .find(|entry| entry.label == label)
            .expect("resource does not exist")
            .entry
    }

    pub fn find_index_buffer(&self, label: &'a str) -> &wgpu::Buffer {
        &self
            .index_buffers
            .iter()
            .find(|entry| entry.label == label)
            .expect("resource does not exist")
            .entry
    }

    pub fn find_vertex_buffer_cursor(&self, label: &'a str) -> &wgpu::BufferAddress {
        &self
            .vertex_buffer_cursors
            .iter()
            .find(|entry| entry.label == label)
            .expect("resource does not exist")
            .entry
    }

    pub fn find_index_buffer_cursor(&self, label: &'a str) -> &wgpu::BufferAddress {
        &self
            .index_buffer_cursors
            .iter()
            .find(|entry| entry.label == label)
            .expect("resource does not exist")
            .entry
    }

    /// Reconfigure stuff to support resizing and hotloading resources

    pub fn reload_shaders(
        &mut self,
        device: &wgpu::Device,
        shader_context: &ShaderContext,
        surface_config: &wgpu::SurfaceConfiguration,
    ) {
        // @TODO FIXME? :)
        let bind_group_layouts = &self.bind_group_layouts;
        let textures = &self.textures;
        let mut targets = None;
        self.passes.iter_mut().for_each(|pass| {
            let layouts = pass
                .pipeline_ctx
                .uniform_bind_group_layout_handles
                .iter()
                .map(|handle| {
                    let layout = &bind_group_layouts[handle.idx];
                    #[cfg(debug_assertions)]
                    assert_eq!(handle.label, layout.label);
                    &layout.entry
                })
                .collect::<Vec<&wgpu::BindGroupLayout>>();

            if let Some(fragment_targets) = &pass.pipeline_ctx.fragment_targets {
                targets = Some(
                    fragment_targets
                        .iter()
                        .map(|target| wgpu::ColorTargetState {
                            format: if let Some(handle) = target.format_handle {
                                let texture = &textures[handle.idx];
                                #[cfg(debug_assertions)]
                                assert_eq!(handle.label, texture.label);
                                texture.entry.format.clone()
                            } else {
                                surface_config.format
                            },
                            blend: target.blend,
                            write_mask: target.write_mask,
                        })
                        .collect(),
                );
            }

            pass.pipeline_ctx.recreate_pipelines(
                &mut pass.pipelines,
                shader_context,
                &layouts,
                device,
                targets.as_ref(),
            );
        });
    }
}
