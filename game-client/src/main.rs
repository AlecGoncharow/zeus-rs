#![feature(never_type)]
use atlas::entity::light::infinite::Infinite;
use atlas::entity::light::infinite::CASCADE_COUNT;
use pantheon::context::Context;
use pantheon::event::EventHandler;

use pantheon::graphics::PolygonMode;

use pantheon::input::keyboard;
use pantheon::input::mouse;

use pantheon::anyhow::*;
use pantheon::winit::event::ModifiersState;
use pantheon::winit::event::MouseButton;
use pantheon::winit::event::VirtualKeyCode;

use pantheon::graphics::prelude::*;
use pantheon::math::prelude::*;

use atlas::camera::Camera;

use atlas::rendering::init::*;
use atlas::rendering::prelude::*;
use entity_manager::EntityManager;

use ui::*;

use atlas::entity::water::*;
use atlas::entity::Entity;
use atlas::entity::EntityKind;
use atlas::message::GameMessage;
use atlas::prelude::*;
use atlas::proc_gen;
use atlas::proc_gen::terrain::TerrainGenerator;
use atlas::vertex::*;
use proc_gen::color::ColorGenerator;
use proc_gen::noise::Perlin;

use hermes::client::ClientInterface;
use hermes::message::Message;

use hermes::tokio;

use pantheon::wgpu;

const PRESENT_MODE: wgpu::PresentMode = wgpu::PresentMode::AutoNoVsync;

// tracing
use std::fs::File;
use std::io::BufWriter;
use std::mem::MaybeUninit;
use std::path::Path;
use tracing::{span, Level};
use tracing_flame::FlameLayer;
use tracing_flame::FlushGuard;
use tracing_subscriber::{prelude::*, registry::Registry};

static PATH: &str = "flame.folded";

fn setup_global_collector(dir: &Path) -> FlushGuard<BufWriter<File>> {
    let (flame_layer, guard) = FlameLayer::with_file(dir.join(PATH)).unwrap();

    let collector = Registry::default().with(flame_layer);

    tracing::subscriber::set_global_default(collector).unwrap();

    guard
}

pub mod entity_manager;
pub mod ui;

struct State<'a> {
    frame: u32,
    entity_manager: EntityManager<'a>,
    mouse_down: bool,
    network_client: ClientInterface<GameMessage>,
    network_queue: Vec<(std::net::SocketAddr, Message<GameMessage>)>,
    fps: f32,
    debug: bool,
    //texture: Texture,
    camera_uniforms: CameraUniforms,
    reflected_camera_uniforms: CameraUniforms,
    handles: Handles<'a>,
    infinite_light: Infinite,
    #[allow(dead_code)]
    global_light_uniforms: GlobalLightUniforms,
    global_shadow_uniforms: GlobalShadowUniforms,
    global_shadow_bake_uniforms: [ShadowBakeUniforms; CASCADE_COUNT],

    //passes: [Pass<'a>; NUM_PASSES],
    #[allow(dead_code)]
    tracing_guard: FlushGuard<BufWriter<File>>,
}

impl<'a> EventHandler<'a> for State<'a> {
    fn draw(&mut self, ctx: &mut Context<'a>) -> Result<()> {
        ctx.start_drawing();
        self.frame += 1;

        let span = span!(Level::DEBUG, "entity_manager draw").entered();
        self.entity_manager.draw(ctx);

        if self.debug {
            self.entity_manager.debug_draw(ctx);
        }
        drop(span);

        //println!("{:#?}", shadow_quad);

        /* Shadow texture debug
        ctx.draw_textured(
            DrawMode::Textured(Topology::TriangleList(PolygonMode::Fill)),
            &self.top_right_ui.verts,
            //TextureKind::Custom(&self.texture),
            TextureKind::Shadow,
            //TextureKind::Depth,
        );
        */
        let _span = span!(Level::DEBUG, "render").entered();
        ctx.render();
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context<'a>) -> Result<()> {
        //self.camera.update();
        //if self.mouse_down {
        //    self.camera.update_pitch_and_angle(ctx);
        //}
        self.network_client
            .drain_message_queue(&mut self.network_queue);

        for (_source, mut message) in self.network_queue.drain(..) {
            println!("[Networking] Got msg {}", message);
            match message.header.id {
                GameMessage::GetId => {
                    let id: usize = message.pull().unwrap();
                    println!("[Networking] Got id {}", id);
                }
                GameMessage::SyncWorld => {
                    while !message.body.is_empty() {
                        let entity: EntityKind = message.pull().unwrap();
                        //println!("[Networking] Got entity: {:#?}", entity);
                        match entity {
                            EntityKind::Sun(s) => {
                                self.entity_manager.sun = s;
                                self.entity_manager.sun.init(ctx);
                            }
                            _ => self.entity_manager.push_entity(ctx, entity),
                        }
                    }
                }
                _ => {}
            }
        }

        let delta_time = ctx.timer_context.delta_time();
        for key in keyboard::pressed_keys(ctx).iter() {
            self.entity_manager
                .camera
                .process_keypress(*key, delta_time);
        }

        if self.mouse_down {
            let delta = mouse::delta(ctx);
            self.entity_manager
                .camera
                .process_mouse_move((delta.x, delta.y).into(), delta_time);
        }

        if self.entity_manager.camera.dirty {
            let span = span!(Level::DEBUG, "dirty camera").entered();
            self.camera_uniforms.view = self.entity_manager.camera.update_view_matrix();
            self.camera_uniforms.position = self.entity_manager.camera.origin;
            self.camera_uniforms.w = self.entity_manager.camera.w;
            self.camera_uniforms
                .push(ctx, &self.handles.camera_uniforms);

            //println!("[CAMERA UNIFORMS] {:#?}", self.camera_uniforms);

            self.infinite_light.update(&self.entity_manager.camera);
            for i in 0..CASCADE_COUNT {
                self.global_shadow_bake_uniforms[i].view =
                    self.infinite_light.cascades[i].cascade_space_transform;
                self.global_shadow_bake_uniforms[i].projection =
                    self.infinite_light.cascades[i].projection;

                self.global_shadow_bake_uniforms[i]
                    .push(ctx, &self.handles.global_shadow_bake_uniforms[i]);
            }

            self.global_shadow_uniforms.shadow0 = self.infinite_light.shadow0;
            self.global_shadow_uniforms.cascade_offsets = self.infinite_light.cascade_offsets;
            self.global_shadow_uniforms.cascade_scales = self.infinite_light.cascade_scales;
            self.global_shadow_uniforms.cascade_planes = self.infinite_light.cascade_planes;
            self.global_shadow_uniforms
                .push(ctx, &self.handles.global_shadow_uniforms);

            // @NOTE reflected camera uniform's position is not updated ever because
            // it is only used for the reflection pass, which doesn't care about the
            // position. Only the water passes care about the camera's position due
            // to the Fresnel Effect
            // See: https://en.wikipedia.org/wiki/Fresnel_equations
            self.reflected_camera_uniforms.view =
                self.entity_manager.camera.update_reflected_view_matrix();
            self.reflected_camera_uniforms.projection = self.entity_manager.camera.projection;
            self.reflected_camera_uniforms.w = self.entity_manager.camera.w_r;
            self.reflected_camera_uniforms
                .push(ctx, &self.handles.reflected_camera_uniforms);

            self.entity_manager.camera.dirty = false;
            drop(span);
        }

        let span = span!(Level::DEBUG, "entity_manager update").entered();
        self.entity_manager.update(ctx);
        drop(span);

        self.fps = 1.0 / ctx.timer_context.average_tick;
        if ctx.timer_context.frame_count % (pantheon::timer::MAX_SAMPLES) == 0 {
            println!(
                "FPS: {:#?}, sample_sum {}, MAX_SAMPLES {}, average_tick {}",
                self.fps,
                ctx.timer_context.sample_sum,
                pantheon::timer::MAX_SAMPLES,
                ctx.timer_context.average_tick
            );
        }

        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context<'a>, keycode: VirtualKeyCode, _repeat: bool) {
        if keycode == VirtualKeyCode::Escape {
            pantheon::event::quit(ctx);
        }

        if keycode == VirtualKeyCode::L {
            if let Some(_) = ctx.forced_draw_mode {
                ctx.forced_draw_mode = None;
            } else {
                ctx.forced_draw_mode = Some(PolygonMode::Line);
            }
        }

        if keycode == VirtualKeyCode::O {
            self.entity_manager.water.toggle_topology(ctx);
        }

        if keycode == VirtualKeyCode::P {
            self.debug = !self.debug;
        }

        if keycode == VirtualKeyCode::C {
            self.entity_manager.camera.infinite_perspective =
                !self.entity_manager.camera.infinite_perspective;
            self.entity_manager.camera.update_projection_matrix();
        }

        if keycode == VirtualKeyCode::R {
            ctx.reload_shaders();
        }

        if keycode == VirtualKeyCode::T {
            let terrain_size = if cfg!(debug_assertions) { 10 } else { 250 };
            self.entity_manager.terrain = generate_terrain(terrain_size, true, None);
            self.entity_manager.terrain.init(ctx);
        }

        if keycode == VirtualKeyCode::Y {
            let terrain_size = if cfg!(debug_assertions) { 10 } else { 250 };
            self.entity_manager.terrain = generate_terrain(terrain_size, false, None);
            self.entity_manager.terrain.init(ctx);
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: VirtualKeyCode) {
        self.entity_manager.camera.process_keyrelease(keycode);
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        println!("Mouse button pressed: {:?}, x: {}, y: {}", button, x, y);
        //self.points.push(Vec3::new(x, y, 0.0));

        if let MouseButton::Right = button {
            self.mouse_down = true;
        }
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        println!("Mouse button pressed: {:?}, x: {}, y: {}", button, x, y);
        //self.points.push(Vec3::new(x, y, 0.0));
        //self.camera.update_pitch_and_angle(ctx);

        if let MouseButton::Right = button {
            self.mouse_down = false;
        }
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, x: f32, y: f32) {
        println!("Mouse wheel scrolled: x: {}, y: {}", x, y);
        //self.camera.update_zoom(Vec2::new(x, y));
    }

    /*
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: VirtualKeyCode,
        _keymods: ModifiersState,
        _repeat: bool,
    ) {
        self.camera.process_keypress(keycode);
        ctx.gfx_context.uniforms.view = self.camera.view_matrix();
    }
    */

    fn resize_event(&mut self, ctx: &mut Context<'a>, width: f32, height: f32) {
        // prevent degenerate case where things go wrong if resize while moving camera orientation
        self.mouse_down = false;

        println!("resize_event: width: {}, height: {}", width, height);
        self.entity_manager.camera.set_aspect((width, height));

        self.camera_uniforms.projection = self.entity_manager.camera.update_projection_matrix();

        self.infinite_light.resize(&self.entity_manager.camera);
        for i in 0..CASCADE_COUNT {
            self.global_shadow_bake_uniforms[i].view =
                self.infinite_light.cascades[i].cascade_space_transform;
            self.global_shadow_bake_uniforms[i].projection =
                self.infinite_light.cascades[i].projection;

            self.global_shadow_bake_uniforms[i]
                .push(ctx, &self.handles.global_shadow_bake_uniforms[i]);
        }

        self.reflected_camera_uniforms.projection = self.entity_manager.camera.projection;
        self.reflected_camera_uniforms
            .push(ctx, &self.handles.reflected_camera_uniforms);

        let depth_texture =
            Texture::create_depth_texture(&ctx.device, &ctx.surface_config, "depth");
        let refraction_depth_texture =
            Texture::create_depth_texture(&ctx.device, &ctx.surface_config, "refraction_depth");
        let reflection_texture = Texture::create_surface_texture(
            &ctx.device,
            &ctx.surface_config,
            self.handles.reflection_texture.label,
        );
        let refraction_texture = Texture::create_surface_texture(
            &ctx.device,
            &ctx.surface_config,
            self.handles.refraction_texture.label,
        );

        /*
        ctx.wrangler
            .swap_texture(&self.handles.depth_texture, depth_texture);
        */

        ctx.wrangler.start_resize();

        rendering::register_surface_bound_texture(ctx, depth_texture, "depth", "depth");

        rendering::register_surface_bound_texture(
            ctx,
            refraction_depth_texture,
            "refraction_depth",
            "depth",
        );

        rendering::register_surface_bound_texture(
            ctx,
            reflection_texture,
            self.handles.reflection_texture.label,
            "basic_textured",
        );
        rendering::register_surface_bound_texture(
            ctx,
            refraction_texture,
            self.handles.refraction_texture.label,
            "basic_textured",
        );

        rendering::recreate_water_sampler_bind_group(ctx);

        ctx.wrangler.validate_resize();
    }

    fn key_mods_changed(&mut self, _ctx: &mut Context, _modifiers_state: ModifiersState) {}
}

#[allow(dead_code)]
fn get_corner_positions(row: f32, col: f32, y: f32) -> [(Vec3, Color, Vec3); 4] {
    [
        (
            (col, y, row).into(),
            Color::floats(0.3, 0.3, 0.3),
            (0, 1, 0).into(),
        ),
        (
            (col, y, row + 1.0).into(),
            Color::floats(0.4, 0.4, 0.4),
            (0, 1, 0).into(),
        ),
        (
            (col + 1.0, y, row).into(),
            Color::floats(0.5, 0.5, 0.5),
            (0, 1, 0).into(),
        ),
        (
            (col + 1.0, y, row + 1.0).into(),
            Color::floats(1., 1., 1.),
            (0, 1, 0).into(),
        ),
    ]
}

#[allow(dead_code)]
fn populate_grid(grid: &mut Vec<(Vec3, Color, Vec3)>, size: i32, y: f32) {
    for row in -size..size {
        for col in -size..size {
            let pos = get_corner_positions(row as f32, col as f32, y);
            grid.push(pos[0]);
            grid.push(pos[1]);
            grid.push(pos[3]);
            grid.push(pos[0]);
            grid.push(pos[2]);
            grid.push(pos[3]);
        }
    }
}

fn generate_terrain(
    terrain_size: usize,
    clamped: bool,
    seed: Option<isize>,
) -> atlas::entity::terrain::Terrain<'static> {
    let mut perlin = Perlin::default();
    if let Some(seed) = seed {
        perlin.seed = seed;
    }

    let color_gen = ColorGenerator::new(
        vec![
            (201, 178, 99).into(),
            (164, 155, 98).into(),
            (164, 155, 98).into(),
            (229, 219, 164).into(),
            (135, 184, 82).into(),
            //(80, 171, 93).into(),
            (120, 120, 120).into(),
            (200, 200, 210).into(),
        ],
        0.45,
    );

    let terrain_gen = TerrainGenerator::new(perlin, color_gen);
    let mut terrain = terrain_gen.generate(terrain_size, clamped);
    terrain.center = (terrain_size as f32 / 2., 0., terrain_size as f32 / 2.).into();

    terrain
}

#[tokio::main]
async fn main() {
    let tracing_guard = setup_global_collector(&Path::new("./flamegraph"));
    let _root_span = span!(Level::INFO, "main").entered();

    let netup = span!(Level::INFO, "network setup").entered();
    let mut network_client: ClientInterface<GameMessage> = ClientInterface::new();
    println!(
        "Connection status: {:?}",
        network_client.connect("127.0.0.1", 8080).await
    );
    let message: Message<GameMessage> = Message::new(GameMessage::GetId);
    network_client.send(message).await.unwrap();
    let message: Message<GameMessage> = Message::new(GameMessage::SyncWorld);
    network_client.send(message).await.unwrap();
    netup.exit();

    let ctx_span = span!(Level::INFO, "Context setup").entered();
    let shader_path = std::path::PathBuf::from("game-client/assets/shaders/wgsl/");
    let (mut ctx, event_loop) =
        Context::new(PRESENT_MODE, Color::new(135, 206, 235).into(), shader_path);
    ctx_span.exit();

    let mut camera = Camera::new(
        (20, 15, 0).into(),
        Vec3::new_from_one(0),
        (0, 1, 0).into(),
        90.0,
        (
            ctx.gfx_context.window_dims.width,
            ctx.gfx_context.window_dims.height,
        ),
        (0.8, 100.0),
    );
    let direction = Vec3::new(3, -1, 0).unit_vector();
    let color = Vec3::new(1, 0.95, 0.95);
    camera.update_orientation();

    let infinite = Infinite::new(&camera, direction, color.into());

    let global_shadow_uniforms = GlobalShadowUniforms::new(
        infinite.shadow0,
        infinite.cascade_offsets,
        infinite.cascade_scales,
        infinite.cascade_planes,
    );

    // @NOTE this has to be 0 unless we want out camera to be paramertized against the water's
    // height which I think is a bit much, probably easier to just approach life as water == 0
    // height
    let water_height = 0.;
    let terrain_height = 3.;
    let world_scale = 2.0;

    let bias = Vec2::new(0.3, 0.8);
    let global_light_uniforms = GlobalLightUniforms::new(direction, color, bias);

    let init_span = span!(Level::INFO, "renderer init").entered();

    let init_params = InitParams {
        global_light_uniforms,
        water_height,
        refraction_offset: 1.0,
    };
    init::init(&mut ctx, init_params);

    init_span.exit();

    let depth_texture_handle = ctx.wrangler.handle_to_texture("depth").expect(":)");

    println!(
        "{:#?}",
        (
            ctx.gfx_context.window_dims.width,
            ctx.gfx_context.window_dims.height,
        )
    );

    let terrain_size = if cfg!(debug_assertions) { 50 } else { 250 };
    let mut terrain = generate_terrain(terrain_size, false, Some(0));
    terrain.center = (
        -(terrain_size as f32) / 2.,
        terrain_height,
        -(terrain_size as f32) / 2.,
    )
        .into();
    terrain.scale = world_scale;
    terrain.init(&mut ctx);
    terrain.register(&mut ctx);

    let mut water = generate_water(terrain_size);
    water.center = terrain.center - (0., terrain_height, 0.).into();
    water.scale = world_scale;
    water.register(&mut ctx);

    let entity_manager = EntityManager::new(camera, terrain, water);

    /*
    let test_bytes = include_bytes!("../../dog.png");
    let texture = pantheon::graphics::texture::Texture::from_bytes(
        &ctx.device,
        &ctx.queue,
        test_bytes,
        "dog",
    )
    .unwrap();
    */

    let image_bytes = include_bytes!("../assets/images/wab.png");
    let _texture = pantheon::graphics::texture::Texture::from_bytes(
        &ctx.device,
        &ctx.queue,
        image_bytes,
        "wab",
    )
    .unwrap();
    let _left_left_ui = TexturableQuad::new((-1., 0.5).into(), (-0.5, 1.0).into());
    let _left_ui = TexturableQuad::new((-0.5, 0.5).into(), (0.0, 1.0).into());
    let _right_ui = TexturableQuad::new((0.0, 0.5).into(), (0.5, 1.0).into());
    let _right_right_ui = TexturableQuad::new((0.5, 0.5).into(), (1., 1.0).into());

    /*
    let refraction = ctx.wrangler.handle_to_texture("refraction").unwrap();
    let refraction_bind = ctx.wrangler.handle_to_bind_group("refraction").unwrap();
    let reflection = ctx.wrangler.handle_to_texture("reflection").unwrap();
    let reflection_bind = ctx.wrangler.handle_to_bind_group("reflection").unwrap();
    let _refraction_depth = ctx.wrangler.handle_to_texture("refraction_depth").unwrap();
    let _refraction_depth_bind = ctx
        .wrangler
        .handle_to_bind_group("refraction_depth")
        .unwrap();
    */

    /*
    let mut textured_quad =
        TexturedQuad::new_with_handles(right_right_ui, refraction_bind, refraction, "refraction");
    textured_quad.register(&mut ctx);
    let mut textured_quad =
        TexturedQuad::new_with_handles(left_left_ui, reflection_bind, reflection, "reflection");
    textured_quad.register(&mut ctx);
    /*
    let mut textured_quad = TexturedQuad::new_with_handles(
        right_ui,
        refraction_depth_bind,
        refraction_depth,
        "refraction_depth",
    );
    textured_quad.register(&mut ctx);
    */
    */

    //@TODO FIXME currently the initial mirrored view matrix is wrong, don't care because it gets
    //"fixed" when orientation is updated

    let global_shadow_bake_uniforms: [ShadowBakeUniforms; CASCADE_COUNT] = {
        let mut arr: [MaybeUninit<ShadowBakeUniforms>; CASCADE_COUNT] =
            unsafe { MaybeUninit::uninit().assume_init() };

        for (i, cascade) in infinite.cascades.iter().enumerate() {
            arr[i].write(ShadowBakeUniforms::new(
                cascade.projection,
                cascade.cascade_space_transform,
            ));
        }

        unsafe { std::mem::transmute(arr) }
    };

    let global_shadow_bake_uniform_handles = {
        let mut arr: [MaybeUninit<BufferHandle>; CASCADE_COUNT] =
            unsafe { MaybeUninit::uninit().assume_init() };

        for (i, uniform) in global_shadow_bake_uniforms.iter().enumerate() {
            let label = int_to_cascade(i);
            let handle = ctx.wrangler.handle_to_uniform_buffer(label).unwrap();
            uniform.push(&mut ctx, &handle);
            arr[i].write(handle);
        }

        unsafe { std::mem::transmute(arr) }
    };

    let camera_uniforms = CameraUniforms::new(
        entity_manager.camera.view,
        entity_manager.camera.projection,
        entity_manager.camera.origin,
        entity_manager.camera.w,
        Vec2::new(
            entity_manager.camera.near_plane,
            entity_manager.camera.far_plane,
        ),
    );
    let reflected_camera_uniforms = CameraUniforms::new(
        entity_manager.camera.reflected_view,
        entity_manager.camera.projection,
        entity_manager.camera.origin,
        entity_manager.camera.w_r,
        Vec2::new(
            entity_manager.camera.near_plane,
            entity_manager.camera.far_plane,
        ),
    );

    let handles = Handles {
        camera_uniforms: ctx.wrangler.handle_to_uniform_buffer(CAMERA).expect(":)"),
        reflected_camera_uniforms: ctx
            .wrangler
            .handle_to_uniform_buffer(CAMERA_REFLECT)
            .expect(":)"),
        depth_texture: depth_texture_handle,
        reflection_texture: ctx
            .wrangler
            .handle_to_texture(REFLECTION_TEXTURE)
            .expect(":)"),
        refraction_texture: ctx
            .wrangler
            .handle_to_texture(REFRACTION_TEXTURE)
            .expect(":)"),
        shaded_pass: ctx.wrangler.handle_to_pass("shaded").expect(":)"),
        global_light_uniform: ctx
            .wrangler
            .handle_to_uniform_buffer(GLOBAL_LIGHT)
            .expect(":)"),
        global_shadow_uniforms: ctx
            .wrangler
            .handle_to_uniform_buffer(GLOBAL_LIGHT_SHADOW)
            .expect(":)"),
        global_shadow_bake_uniforms: global_shadow_bake_uniform_handles,
    };
    camera_uniforms.push(&mut ctx, &handles.camera_uniforms);
    reflected_camera_uniforms.push(&mut ctx, &handles.reflected_camera_uniforms);
    global_shadow_uniforms.push(&mut ctx, &handles.global_shadow_uniforms);

    let mut my_game = State {
        frame: 0,
        entity_manager,
        mouse_down: false,
        network_client,
        network_queue: vec![],
        fps: 0.,
        debug: false,
        camera_uniforms,
        reflected_camera_uniforms,
        handles,

        infinite_light: infinite,
        global_light_uniforms,
        global_shadow_uniforms,
        global_shadow_bake_uniforms,

        tracing_guard,
    };

    /* @TODO
    ctx.set_view(my_game.entity_manager.camera.view);
    ctx.set_projection(my_game.entity_manager.camera.projection);
    */
    let cube =
        atlas::entity::cube::Cuboid::cube(5.0, (0, 15, 0).into(), None, VertexKind::Shaded, None);
    my_game
        .entity_manager
        .push_entity(&mut ctx, EntityKind::from(cube));

    pantheon::event::run(event_loop, ctx, my_game);
}
