use crate::rendering;
use crate::rendering::init::UNIFORM_BUFFER_VERTEX;
use crate::rendering::prelude::*;
use crate::vertex::*;
use pantheon::graphics::prelude::*;
use pantheon::graphics::Drawable;
use pantheon::math::prelude::*;
use pantheon::prelude::*;

unsafe impl bytemuck::Pod for WaterPushConstants {}
unsafe impl bytemuck::Zeroable for WaterPushConstants {}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct WaterPushConstants {
    pub wave_time: f32,
}

#[derive(Debug, Clone)]
pub struct Water<'a> {
    pub verts: Vec<WaterVertex>,
    pub center: Vec3,
    draw_call_handle: Option<DrawCallHandle<'a>>,
    topology: Topology,
    wave_time: f32,
}

impl<'a> Water<'a> {
    pub fn from_data(verts: Vec<WaterVertex>) -> Self {
        Self {
            verts,
            center: Vec3::new_from_one(0),
            draw_call_handle: None,
            topology: Topology::TriangleList(PolygonMode::Fill),
            wave_time: 0.0,
        }
    }

    pub fn register(&mut self, ctx: &mut Context<'a>) {
        let push_constant = Some(PushConstant::vertex_data(
            0,
            &[WaterPushConstants { wave_time: 0.0 }],
        ));

        let uniforms = StaticEntityUniforms {
            model_matrix: self.model_matrix(),
        };
        let (bind_group_handle, _buffer_handle) =
            uniforms.register(ctx, UNIFORM_BUFFER_VERTEX, "water1");

        self.draw_call_handle = Some(rendering::register(
            ctx,
            &["water"],
            "water",
            self.topology,
            &self.verts,
            0..1,
            push_constant,
            &[bind_group_handle],
        ));
    }

    pub fn update(&mut self, ctx: &mut Context<'a>) {
        if let Some(draw_call_handle) = self.draw_call_handle {
            self.wave_time += ctx.timer_context.delta_time();
            draw_call_handle.set_push_constant_data(
                ctx,
                &[WaterPushConstants {
                    wave_time: self.wave_time,
                }],
            );
        }
    }

    pub fn toggle_topology(&mut self, ctx: &mut Context<'a>) {
        if let Some(draw_call_handle) = self.draw_call_handle {
            let mut draw_call = ctx.wrangler.get_draw_call_mut(&draw_call_handle);

            if let DrawCall::Vertex {
                ref mut topology, ..
            } = &mut draw_call
            {
                match topology.inner() {
                    PolygonMode::Fill => topology.set_inner(PolygonMode::Line),
                    PolygonMode::Line => topology.set_inner(PolygonMode::Point),
                    PolygonMode::Point => topology.set_inner(PolygonMode::Fill),
                }
            }
        }
    }
}

const VERTS_PER_SQUARE: usize = 3 * 2;

/// inspired by https://github.com/TheThinMatrix/WaterStep1/blob/master/water/water/WaterGenerator.java
pub fn generate_water<'a>(size: usize) -> Water<'a> {
    let vert_count = size * size * VERTS_PER_SQUARE;
    let mut verts: Vec<WaterVertex> = Vec::with_capacity(vert_count);

    fn get_corner_pos(col: u32, row: u32) -> [Vec2; 4] {
        [
            Vec2::new(col, row),
            Vec2::new(col, row + 1),
            Vec2::new(col + 1, row),
            Vec2::new(col + 1, row + 1),
        ]
    }

    fn get_indicators(corner_pos: [Vec2; 4], current: usize, v1: usize, v2: usize) -> [i8; 4] {
        let current = corner_pos[current];
        let v1_pos = corner_pos[v1];
        let v2_pos = corner_pos[v2];
        let offset1 = v1_pos - current;
        let offset2 = v2_pos - current;

        [
            offset1.x as i8,
            offset1.y as i8,
            offset2.x as i8,
            offset2.y as i8,
        ]
    }

    fn push_triangle(verts: &mut Vec<WaterVertex>, corner_pos: [Vec2; 4], left: bool) {
        let i0 = if left { 0 } else { 2 };
        let i1 = 1;
        let i2 = if left { 2 } else { 3 };

        verts.push(WaterVertex::from((
            corner_pos[i0],
            get_indicators(corner_pos, i0, i1, i2),
        )));
        verts.push(WaterVertex::from((
            corner_pos[i1],
            get_indicators(corner_pos, i1, i2, i0),
        )));
        verts.push(WaterVertex::from((
            corner_pos[i2],
            get_indicators(corner_pos, i2, i0, i1),
        )));
    }

    for row in 0..size {
        for col in 0..size {
            let corner_pos = get_corner_pos(col as u32, row as u32);
            push_triangle(&mut verts, corner_pos, true);
            push_triangle(&mut verts, corner_pos, false);
        }
    }

    Water::from_data(verts)
}

impl<'a> Drawable for Water<'a> {
    fn model_matrix(&self) -> Mat4 {
        Mat4::translation::<f32>((-1. * self.center).into())
    }

    fn rotate(&mut self, _theta: f32, _axis: Vec3) {}

    fn translate(&mut self, _tuple: (f32, f32, f32)) {}
}
