use atlas::rendering;
use atlas::vertex::BasicTexturedVertex;
use pantheon::graphics::prelude::*;
use pantheon::prelude::*;
use pantheon::Vec2;

pub struct TexturedQuad<'a> {
    pub quad: TexturableQuad,
    pub label: &'a str,
    #[allow(dead_code)]
    texture_handle: TextureHandle<'a>,
    bind_group_handle: BindGroupHandle<'a>,
    draw_call_handle: Option<DrawCallHandle<'a>>,
}

impl<'a> TexturedQuad<'a> {
    const TOPOLOGY: Topology = Topology::TriangleList(PolygonMode::Fill);

    pub fn new(
        ctx: &mut Context<'a>,
        quad: TexturableQuad,
        texture: Texture,
        label: &'a str,
    ) -> Self {
        let (bind_group_handle, texture_handle) =
            rendering::register_texture(ctx, texture, label, "basic_textured", None);

        Self {
            quad,
            label,
            texture_handle,
            bind_group_handle,
            draw_call_handle: None,
        }
    }

    pub fn new_with_handles(
        quad: TexturableQuad,
        bind_group_handle: BindGroupHandle<'a>,
        texture_handle: TextureHandle<'a>,
        label: &'a str,
    ) -> Self {
        Self {
            quad,
            label,
            bind_group_handle,
            texture_handle,
            draw_call_handle: None,
        }
    }

    pub fn register(&mut self, ctx: &mut Context<'a>) {
        self.draw_call_handle = Some(rendering::register(
            ctx,
            &["basic_textured"],
            "basic_textured",
            Self::TOPOLOGY,
            &self.quad.verts,
            0..1,
            None,
            Some(self.bind_group_handle),
        ));
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TexturableQuad {
    pub verts: [BasicTexturedVertex; 6],
}

impl TexturableQuad {
    /// https://github.com/gfx-rs/wgpu#coordinate-systems
    pub fn new(bot_left: Vec2, top_right: Vec2) -> Self {
        let diff_y = top_right.y - bot_left.y;
        let diff_x = top_right.x - bot_left.x;

        let top_left = Vec2 {
            x: bot_left.x,
            y: bot_left.y + diff_y,
        };
        let bot_right = Vec2 {
            x: bot_left.x + diff_x,
            y: bot_left.y,
        };

        Self {
            verts: [
                BasicTexturedVertex::new(top_left, (0.0, 0.0).into()),
                BasicTexturedVertex::new(bot_left, (0.0, 1.0).into()),
                BasicTexturedVertex::new(top_right, (1.0, 0.0).into()),
                BasicTexturedVertex::new(top_right, (1.0, 0.0).into()),
                BasicTexturedVertex::new(bot_left, (0.0, 1.0).into()),
                BasicTexturedVertex::new(bot_right, (1.0, 1.0).into()),
            ],
        }
    }

    pub fn new_vk_coords(bot_left: Vec2, top_right: Vec2) -> Self {
        let add_one = Vec2::new(1, 1);
        let scale = Vec2::new(2, -2);

        Self::new(
            scale.make_comp_mul(&bot_left) + add_one,
            scale.make_comp_mul(&top_right) + add_one,
        )
    }
}
