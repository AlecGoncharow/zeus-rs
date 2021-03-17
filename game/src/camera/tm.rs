/// based on, ty thinmatrix https://github.com/TheThinMatrix/LowPolyTerrain/blob/master/src/main/Camera.java
///
/// questionable java code but I just want to get something working
use pantheon::graphics::CameraProjection;
use pantheon::math::Mat4;
use pantheon::math::Vec2;
use pantheon::math::Vec3;

use pantheon::context::Context;
use pantheon::winit::dpi::LogicalSize;

const PITCH_SENSITIVITY: f32 = 0.3;
const YAW_SENSITIVITY: f32 = 0.3;
const MAX_PITCH: f32 = 90.0;

const FOV: f32 = 70.0;
const NEAR_PLANE: f32 = 0.4;
const FAR_PLANE: f32 = 2500.0;

const Y_OFFSET: f32 = 5.0;

const TERRAIN_SIZE: f32 = 0.5;

#[derive(Debug)]
pub struct Camera {
    pub position: Vec3,
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,

    pub yaw: f32,

    pub pitch_float: SmoothFloat,
    pub angle_float: SmoothFloat,
    pub distance_float: SmoothFloat,

    pub dims: LogicalSize<f32>,
}

impl CameraProjection for &Camera {
    fn projection_view_matrix(&self) -> Mat4 {
        let clip: Mat4 = (
            (1.0, 0.0, 0.0, 0.0),
            (0.0, -1.0, 0.0, 0.0),
            (0.0, 0.0, 0.5, 0.0),
            (0.0, 0.0, 0.5, 1.0),
        )
            .into();

        clip * self.projection_matrix * self.view_matrix
    }

    fn projection_matrix(&self) -> Mat4 {
        self.projection_matrix
    }
    fn view_matrix(&self) -> Mat4 {
        self.view_matrix
    }
}

impl Camera {
    pub fn new(ctx: &Context) -> Self {
        Self {
            position: Vec3::new_from_one(-1),
            view_matrix: Mat4::identity(),
            projection_matrix: Self::create_projection_matrix(ctx),

            yaw: 0.0,

            pitch_float: SmoothFloat::new(10.0, 10.0),
            angle_float: SmoothFloat::new(0.0, 10.0),
            distance_float: SmoothFloat::new(10.0, 5.0),

            dims: ctx
                .gfx_context
                .window_dims
                .to_logical(ctx.window.scale_factor()),
        }
    }

    pub fn update(&mut self) {
        // xd 60 fps :) :) :) @TODO FIX
        self.angle_float.update(1.0 / 60.0);
        self.distance_float.update(0.01);
        self.pitch_float.update(1.0 / 60.0);

        self.update_camera_position();
        self.yaw = 360.0 - self.angle_float.actual;
        self.yaw %= 360.0;
        self.update_view_matrix();
    }

    fn update_view_matrix(&mut self) {
        let mut view_matrix = Mat4::identity();
        //println!("{:#?}", view_matrix);

        let pitch_rotate = Mat4::rotation_from_degrees(self.pitch_float.actual, (1, 0, 0).into());
        view_matrix = pitch_rotate * view_matrix;
        //println!("{:#?}", view_matrix);

        let yaw_rotate = Mat4::rotation_from_degrees(self.yaw, (0, 1, 0).into());
        view_matrix = yaw_rotate * view_matrix;
        //println!("{:#?}", view_matrix);

        let negative_camera = -1.0 * self.position;
        let camera_trans =
            Mat4::translation((negative_camera.x, negative_camera.y, negative_camera.z));
        view_matrix = camera_trans * view_matrix;
        //println!("{:#?}", view_matrix);

        self.view_matrix = view_matrix;
    }

    // https://www.khronos.org/registry/OpenGL-Refpages/gl2.1/xhtml/gluPerspective.xml
    pub fn create_projection_matrix(ctx: &Context) -> Mat4 {
        let mut projection_matrix = Mat4::identity();

        let aspect_ratio = ctx.gfx_context.window_dims.width / ctx.gfx_context.window_dims.height;
        let y_scale = 1.0 / (FOV / 2.0).to_radians().tan();
        let x_scale = y_scale / aspect_ratio;
        let fustrum_length = FAR_PLANE - NEAR_PLANE;

        println!("scale {}, {}", x_scale, y_scale);

        projection_matrix.x.x = x_scale;
        projection_matrix.y.y = y_scale;
        projection_matrix.z.z = -(FAR_PLANE + NEAR_PLANE) / fustrum_length;
        projection_matrix.z.w = -1.0;
        projection_matrix.w.z = -(2.0 * NEAR_PLANE * FAR_PLANE) / fustrum_length;
        projection_matrix.w.w = 0.0;

        projection_matrix
    }

    pub fn update_camera_position(&mut self) {
        let theta = self.angle_float.actual;
        let hori_distance = self.distance_float.actual * self.pitch_float.actual.to_radians().cos();
        let vert_distance = self.distance_float.actual * self.pitch_float.actual.to_radians().sin();

        self.position.x = TERRAIN_SIZE / 2.0 + (hori_distance * theta.to_radians().sin());
        self.position.y = vert_distance + Y_OFFSET;
        self.position.z = TERRAIN_SIZE / 2.0 + (hori_distance * theta.to_radians().cos());
    }

    pub fn update_pitch_and_angle(&mut self, ctx: &Context) {
        let pitch = ctx.mouse_context.last_delta.y as f32 * PITCH_SENSITIVITY;
        self.pitch_float.target -= pitch;
        self.pitch_float.clamp_target(0.0, MAX_PITCH);

        let angle = ctx.mouse_context.last_delta.x as f32 * YAW_SENSITIVITY;
        self.angle_float.target -= angle;
    }

    pub fn update_zoom(&mut self, delta: Vec2) {
        let mut target = self.distance_float.target;
        let zoom = delta.y * 0.0008 * target;
        target -= zoom;
        target = if target < 1.0 { 1.0 } else { target };

        self.distance_float.target = target;
    }
}

#[derive(Debug)]
pub struct SmoothFloat {
    pub agility: f32,
    pub actual: f32,
    pub target: f32,
}

impl SmoothFloat {
    pub fn new(init: f32, agility: f32) -> Self {
        Self {
            agility,
            actual: init,
            target: init,
        }
    }

    pub fn update(&mut self, delta: f32) {
        let offset = self.target - self.actual;
        let change = offset * delta * self.agility;
        self.actual += change;
    }

    pub fn clamp_target(&mut self, min: f32, max: f32) {
        self.target = if self.target < min {
            min
        } else if self.target > max {
            max
        } else {
            self.target
        };
    }
}
