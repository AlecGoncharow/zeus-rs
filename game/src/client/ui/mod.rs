use pantheon::graphics::vertex::TexturedVertex;
use pantheon::{Color, Vec2};

#[derive(Debug)]
pub struct TexturableQuad {
    pub verts: [TexturedVertex; 6],
}

impl TexturableQuad {
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
                TexturedVertex::new(top_left.vec3(), Color::new(1, 1, 1), (0.0, 0.0).into()),
                TexturedVertex::new(bot_left.vec3(), Color::new(1, 1, 1), (0.0, 1.0).into()),
                TexturedVertex::new(top_right.vec3(), Color::new(1, 1, 1), (1.0, 0.0).into()),
                TexturedVertex::new(top_right.vec3(), Color::new(1, 1, 1), (1.0, 0.0).into()),
                TexturedVertex::new(bot_left.vec3(), Color::new(1, 1, 1), (0.0, 1.0).into()),
                TexturedVertex::new(bot_right.vec3(), Color::new(1, 1, 1), (1.0, 1.0).into()),
            ],
        }
    }
}
