

pub struct Frame<'a> {
    // The active pass we are in. This keeps track of the step we are in.
    // - If `num_pass` is 0, then we haven't start anything yet.
    // - If `num_pass` is 1, then we have finished drawing all the objects of the scene.
    // - Otherwise the frame is finished.
    // In a more complex application you can have dozens of passes, in which case you probably
    // don't want to document them all here.
    num_pass: u8,

    // Future to wait upon before the main rendering.
    before_main_cb_future: Option<Box<dyn GpuFuture>>,
    // Framebuffer that was used when starting the render pass.
    framebuffer: Arc<dyn FramebufferAbstract + Send + Sync>,
    // The command buffer builder that will be built during the lifetime of this object.
    command_buffer: Option<AutoCommandBufferBuilder>,
    // Matrix that was passed to `frame()`.
    world_to_framebuffer
}


impl<'a> Frame<'a> {
    /// Returns an enumeration containing the next pass of the rendering.
    pub fn next_pass<'f>(&'f mut self) -> Option<Pass<'f, 'a>> {
        // This function reads `num_pass` increments its value, and returns a struct corresponding
        // to that pass that the user will be able to manipulate in order to customize the pass.
        match { let current_pass = self.num_pass; self.num_pass += 1; current_pass } {
            0 => {
                // If we are in the pass 0 then we haven't start anything yet.
                // We already called `begin_render_pass` (in the `frame()` method), and that's the
                // state we are in.
                // We return an object that will allow the user to draw objects on the scene.
                Some(Pass::Deferred(DrawPass {
                                    frame: self,
                                }))
            },

            1 => {
                // If we are in pass 2 then we have finished applying lighting.
                // We take the builder, call `end_render_pass()`, and then `build()` it to obtain
                // an actual command buffer.
                let command_buffer =
                    self.command_buffer
                        .take()
                        .unwrap()
                        .end_render_pass()
                        .unwrap()
                        .build()
                        .unwrap();

                // Extract `before_main_cb_future` and append the command buffer execution to it.
                let after_main_cb = self.before_main_cb_future.take().unwrap()
                    .then_execute(self.system.gfx_queue.clone(), command_buffer)
                    .unwrap();
                // We obtain `after_main_cb`, which we give to the user.
                Some(Pass::Finished(Box::new(after_main_cb)))
            },

            // If the pass is over 2 then the frame is in the finished state and can't do anything
            // more.
            _ => None,
        }
    }
}


/// Struct provided to the user that allows them to customize or handle the pass.
pub enum Pass<'f, 's: 'f> {
    /// We are in the pass where we draw objects on the scene. The `DrawPass` allows the user to
    /// draw the objects.
    Deferred(DrawPass<'f, 's>),

    /// The frame has been fully prepared, and here is the future that will perform the drawing
    /// on the image.
    Finished(Box<dyn GpuFuture>),
}
