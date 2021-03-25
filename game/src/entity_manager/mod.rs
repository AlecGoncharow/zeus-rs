use pantheon::context::Context;
use pantheon::input::mouse;
use pantheon::math::Dim;
use pantheon::math::Vec3;
use pantheon::math::Vec4;

use core::camera::Camera;

// use component::AsComponent;
use core::entity::component::DrawComponent;
use core::entity::component::MouseComponent;
use core::entity::{Entity, EntityKind};

#[allow(dead_code)]
pub struct EntityManager {
    pub camera: Camera,
    entities: Vec<EntityKind>,
    commands: Vec<CommandKind>,
}

// scaffolding to allow for undoable/redoable actions
//#[enum_dispatch(CommandKind)]
pub trait Command {
    fn execute(&mut self);
    fn undo(&mut self);
}

#[allow(dead_code)]
//#[enum_dispatch]
pub enum CommandKind {}

impl EntityManager {
    pub fn new(camera: Camera) -> Self {
        Self {
            camera,
            entities: vec![],
            commands: vec![],
        }
    }

    fn get_mouse_ray(ctx: &mut Context) -> Option<Vec3> {
        let mouse_pos = mouse::position(ctx);
        let ndc_x = (2.0 * mouse_pos.x) / ctx.gfx_context.window_dims.width - 1.0;
        let ndc_y = 1.0 - (2.0 * mouse_pos.y) / ctx.gfx_context.window_dims.height;

        let clip = Vec4::new(ndc_x, ndc_y, 0.0, 1.0);
        let mut eye = if let Some(inv_proj) = ctx.gfx_context.projection_transform.invert() {
            inv_proj * clip
        } else {
            return None;
        };

        eye.z = -1.0;
        eye.w = 0.0;

        let world = if let Some(inv_view) = ctx.gfx_context.view_transform.invert() {
            inv_view * eye
        } else {
            return None;
        };

        Some(world.truncate(Dim::W).make_unit_vector())
    }

    pub fn update(&mut self, ctx: &mut Context) {
        let mouse_ray = Self::get_mouse_ray(ctx);

        let camera_origin = self.camera.origin;
        let before = std::time::Instant::now();
        let mut closest: Option<(&mut dyn MouseComponent, Vec3, f32)> = None;
        self.entities.iter_mut().for_each(|entity| {
            entity.update(ctx);
            if let Some(mouse_ray) = mouse_ray {
                if let Some(hit) = entity.check_collision(ctx, camera_origin, mouse_ray) {
                    if let Some(other) = &closest {
                        if (hit.1 - camera_origin).magnitude()
                            < (other.1 - camera_origin).magnitude()
                        {
                            // hit is closer
                            closest = Some(hit);
                        }
                    } else {
                        // no other hit yet
                        closest = Some(hit);
                    }
                }
            }
        });
        let after = std::time::Instant::now();
        if ctx.timer_context.frame_count % pantheon::timer::MAX_SAMPLES == 0 {
            println!(
                "Iterate turnaround time: ns {:#?}",
                (after - before).subsec_nanos()
            );
        }

        if let Some(hit) = closest {
            //println!("hit pos : {:#?}, t: {:#?} ", hit.1, hit.2);
            hit.0.mouse_over(ctx, hit.1, &self.camera);
        }
    }

    pub fn draw(&mut self, ctx: &mut Context) {
        self.entities.iter_mut().for_each(|entity| {
            entity.draw(ctx);
        });
    }

    pub fn push_entity(&mut self, entity: EntityKind) {
        self.entities.push(entity);
    }
}
