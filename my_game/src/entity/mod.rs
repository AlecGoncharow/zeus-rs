use my_engine::context::Context;
use my_engine::input::mouse;
use my_engine::math::Dim;
use my_engine::math::Vec3;
use my_engine::math::Vec4;

use crate::camera::my_camera::Camera;

pub mod component;
use component::AsComponent;
use component::MouseComponent;

pub mod cube;
pub mod plane;
pub mod triangle;

#[allow(dead_code)]
enum Message {
    Foo,
    Bar,
}

pub trait Entity: AsComponent {
    // TODO add callback message function
    fn update(&mut self, ctx: &mut Context);
}

#[allow(dead_code)]
pub struct EntityManager {
    pub camera: Camera,
    entities: Vec<Box<dyn Entity>>,
    commands: Vec<Box<dyn Command>>,
}

// scaffolding to allow for undoable/redoable actions
pub trait Command {
    fn execute(&mut self);
    fn undo(&mut self);
}

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
        let ndc_y = (2.0 * mouse_pos.y) / ctx.gfx_context.window_dims.height - 1.0;

        println!("NDC COORD {:?}, {:?}", ndc_x, ndc_y);

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
        println!(
            "mouse ray {:#?}, camera_pos: {:#?}",
            mouse_ray, self.camera.origin
        );

        let camera_origin = self.camera.origin;
        let mut closest: Option<(&mut dyn MouseComponent, Vec3, f64)> = None;
        self.entities.iter_mut().for_each(|entity| {
            entity.update(ctx);
            if let Some(mouse_ray) = mouse_ray {
                if let Some(mousable) = entity.as_mouseable() {
                    if let Some(hit) = mousable.check_collision(ctx, camera_origin, mouse_ray) {
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
            }
        });

        if let Some(hit) = closest {
            println!("hit pos : {:#?}, t: {:#?} ", hit.1, hit.2);
            hit.0.mouse_over(ctx, hit.1, &self.camera);
        }
    }

    pub fn draw(&mut self, ctx: &mut Context) {
        self.entities.iter_mut().for_each(|entity| {
            if let Some(drawable) = entity.as_drawable() {
                drawable.draw(ctx);
            }
        });
    }

    pub fn push_entity(&mut self, entity: impl Entity + 'static) {
        self.entities.push(Box::new(entity));
    }
}
