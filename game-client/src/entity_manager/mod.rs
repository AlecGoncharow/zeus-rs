use atlas::entity::terrain::Terrain;
use pantheon::context::Context;
use pantheon::input::mouse;
use pantheon::math::Dim;
use pantheon::math::Vec3;
use pantheon::math::Vec4;

use pantheon::winit::window::CursorIcon;

use atlas::camera::Camera;

// use component::AsComponent;
use atlas::entity::component::DrawComponent;
use atlas::entity::component::MouseComponent;
use atlas::entity::component::MousePick;
use atlas::entity::cube;
use atlas::entity::sun::Sun;
use atlas::entity::water::Water;
use atlas::entity::{Entity, EntityKind};
use atlas::Color;

// @NOTE this probably should move over to `atlas` but I will hold off on doing that until
// the server code gets more complex

#[allow(dead_code)]
pub struct EntityManager<'a> {
    pub camera: Camera,
    pub sun: Sun<'a>,
    pub terrain: Terrain<'a>,
    pub water: Water<'a>,
    new_entities: Vec<EntityKind<'a>>,
    entities: Vec<EntityKind<'a>>,
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

impl<'a> EntityManager<'a> {
    pub fn new(camera: Camera, terrain: Terrain<'a>, water: Water<'a>) -> Self {
        Self {
            camera,
            terrain,
            sun: Sun::new(
                (200, 100, 0).into(),
                20.,
                Color::new(255, 250, 209),
                Color::new(255, 250, 209),
            ),
            new_entities: vec![],
            entities: vec![],
            commands: vec![],
            water,
        }
    }

    fn get_mouse_ray(&self, ctx: &mut Context) -> Option<Vec3> {
        let mouse_pos = mouse::position(ctx);
        let ndc_x = (2.0 * mouse_pos.x) / ctx.gfx_context.window_dims.width - 1.0;
        let ndc_y = 1.0 - (2.0 * mouse_pos.y) / ctx.gfx_context.window_dims.height;

        let device = Vec4::new(ndc_x, ndc_y, 0.0, 1.0);
        let mut camera = if let Some(inv_proj) = self.camera.projection.invert() {
            inv_proj * device
        } else {
            return None;
        };

        camera.z = -1.0;
        camera.w = 0.0;

        let world = self.camera.transform * camera;

        Some(world.truncate(Dim::W).make_unit_vector())
    }

    pub fn update(&mut self, ctx: &mut Context<'a>) {
        for mut entity in self.new_entities.drain(..) {
            entity.init(ctx);
            self.entities.push(entity);
        }

        self.sun.update(ctx);
        self.water.update(ctx);

        let mouse_ray = self.get_mouse_ray(ctx);

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
        if ctx.timer_context.frame_count % (pantheon::timer::MAX_SAMPLES) == 0 {
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

    pub fn draw(&mut self, ctx: &mut Context<'a>) {
        self.terrain.draw(ctx);
        self.sun.draw(ctx);
        self.entities.iter_mut().for_each(|entity| {
            entity.draw(ctx);
        });
    }

    pub fn debug_draw(&mut self, ctx: &mut Context<'a>) {
        self.terrain.debug_draw(ctx);
        self.sun.debug_draw(ctx);
        self.entities.iter_mut().for_each(|entity| {
            entity.debug_draw(ctx);
        });
    }

    pub fn push_entity(&mut self, ctx: &mut Context<'a>, mut entity: EntityKind<'a>) {
        entity.register(ctx);
        self.new_entities.push(entity);
    }

    pub fn get_sun_mesh(&self) -> Sun {
        self.sun
    }

    pub fn set_sun_mesh(&mut self, mesh: cube::Cuboid<'a>) {
        self.sun.cube = mesh;
    }
}
