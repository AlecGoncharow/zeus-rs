use core::entity::terrain::Terrain;
use pantheon::context::Context;
use pantheon::input::mouse;
use pantheon::math::Dim;
use pantheon::math::Vec3;
use pantheon::math::Vec4;

use pantheon::winit::window::CursorIcon;

use core::camera::Camera;

// use component::AsComponent;
use core::entity::component::DrawComponent;
use core::entity::component::MouseComponent;
use core::entity::component::MousePick;
use core::entity::cube;
use core::entity::sun::Sun;
use core::entity::{Entity, EntityKind};
use core::Color;

#[allow(dead_code)]
pub struct EntityManager {
    pub camera: Camera,
    pub sun: Sun,
    terrain: Terrain,
    new_entities: Vec<EntityKind>,
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
    pub fn new(camera: Camera, terrain: Terrain) -> Self {
        Self {
            camera,
            terrain,
            sun: Sun::new(
                (100, 0, 100).into(),
                20.,
                Color::new(255, 250, 209),
                Color::new(255, 250, 209),
            ),
            new_entities: vec![],
            entities: vec![],
            commands: vec![],
        }
    }

    fn get_mouse_ray(ctx: &mut Context) -> Option<Vec3> {
        let mouse_pos = mouse::position(ctx);
        let ndc_x = (2.0 * mouse_pos.x) / ctx.gfx_context.window_dims.width - 1.0;
        let ndc_y = 1.0 - (2.0 * mouse_pos.y) / ctx.gfx_context.window_dims.height;

        let clip = Vec4::new(ndc_x, ndc_y, 0.0, 1.0);
        let mut eye = if let Some(inv_proj) = ctx.gfx_context.uniforms.projection.invert() {
            inv_proj * clip
        } else {
            return None;
        };

        eye.z = -1.0;
        eye.w = 0.0;

        let world = if let Some(inv_view) = ctx.gfx_context.uniforms.view.invert() {
            inv_view * eye
        } else {
            return None;
        };

        Some(world.truncate(Dim::W).make_unit_vector())
    }

    pub fn update(&mut self, ctx: &mut Context) {
        for mut entity in self.new_entities.drain(..) {
            entity.init(ctx);
            self.entities.push(entity);
        }

        self.sun.update(ctx);

        let mouse_ray = Self::get_mouse_ray(ctx);

        let camera_origin = self.camera.origin;
        let before = std::time::Instant::now();
        let mut closest: Option<MousePick> = None;
        if let Some(mouse_ray) = mouse_ray {
            closest = self.sun.check_collision(ctx, camera_origin, mouse_ray);
            self.entities.iter_mut().for_each(|entity| {
                entity.update(ctx);
                {
                    if let Some(hit) = entity.check_collision(ctx, camera_origin, mouse_ray) {
                        if let Some(other) = &closest {
                            if (hit.point - camera_origin).magnitude()
                                < (other.point - camera_origin).magnitude()
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
        }
        let after = std::time::Instant::now();
        if ctx.timer_context.frame_count % (pantheon::timer::MAX_SAMPLES * 10) == 0 {
            println!(
                "Iterate turnaround time: ns {:#?}",
                (after - before).subsec_nanos()
            );
        }

        if let Some(hit) = closest {
            //println!("hit pos : {:#?}, t: {:#?} ", hit.1, hit.2);
            hit.entity.mouse_over(ctx, hit.point, &self.camera);
            ctx.set_cursor_icon(CursorIcon::Move);
        } else {
            ctx.set_cursor_icon(CursorIcon::Default);
        }
    }

    pub fn draw(&mut self, ctx: &mut Context) {
        self.terrain.draw(ctx);
        self.sun.draw(ctx);
        self.entities.iter_mut().for_each(|entity| {
            entity.draw(ctx);
        });
    }

    pub fn debug_draw(&mut self, ctx: &mut Context) {
        self.terrain.debug_draw(ctx);
        self.sun.debug_draw(ctx);
        self.entities.iter_mut().for_each(|entity| {
            entity.debug_draw(ctx);
        });
    }

    pub fn push_entity(&mut self, entity: EntityKind) {
        self.new_entities.push(entity);
    }

    pub fn get_sun_mesh(&self) -> Sun {
        self.sun
    }

    pub fn set_sun_mesh(&mut self, mesh: cube::Cuboid) {
        self.sun.cube = mesh;
    }
}
