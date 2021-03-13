use engine::math::Mat4;
use engine::math::Vec2;
use engine::math::Vec3;
use engine::math::Vec4;
use engine::winit::event::VirtualKeyCode;

//const YAW_DEFAULT: f32 = -90.0;
//const PITCH_DEFAULT: f32 = 0.0;
const MOVE_DEFAULT: f32 = 0.05;
const MOUSE_DEFAULT: f32 = 0.5;

#[derive(Debug)]
pub struct Camera {
    pub origin: Vec3,
    pub u: Vec3,
    pub v: Vec3,
    pub w: Vec3,
    pub world_up: Vec3,

    // rotation things
    pub yaw: f32,
    pub pitch: f32,

    // control things
    pub move_speed: f32,
    pub fast_move: bool,
    pub mouse_speed: f32,

    // projection matric things, not sure if it should be here
    pub aspect: f32,
    pub vfov: f32,
    pub near_plane: f32,
    pub far_plane: f32,
}

impl Camera {
    pub fn new(
        look_from: Vec3,
        look_at: Vec3,
        world_up: Vec3,
        vfov: f32,
        (width, height): (f32, f32),
        (near_plane, far_plane): (f32, f32),
    ) -> Self {
        let aspect = width / height;

        let origin = look_from;
        // todo thinkabout camera change on resize
        let w = (look_from - look_at).make_unit_vector();
        let u = (w.cross(&world_up)).make_unit_vector();
        let v = u.cross(&w).make_unit_vector();

        println!("u: {:#?}, v: {:#?}, w: {:#?}", u, v, w);
        let pitch = w.y.asin();
        let yaw = {
            if pitch.cos() != 0.0 {
                let cos_yaw = w.x / pitch.cos();
                cos_yaw.acos().to_degrees()
            } else {
                // degenerate case idk what to do with this
                1.0f32.to_degrees()
            }
        };
        let pitch = pitch.to_degrees();
        println!("pitch: {:#?}, yaw: {:#?}", pitch, yaw);

        Self {
            origin,
            u,
            v,
            w,
            world_up,

            yaw,
            pitch,

            move_speed: MOVE_DEFAULT,
            fast_move: false,
            mouse_speed: MOUSE_DEFAULT,

            aspect,
            vfov,
            near_plane,
            far_plane,
        }
    }

    pub fn set_aspect(&mut self, (width, height): (f32, f32)) {
        self.aspect = width / height;
    }

    pub fn view_matrix(&self) -> Mat4 {
        let rotation = Mat4::new(
            Vec4::new(self.u.x, self.v.x, self.w.x, 0.0),
            Vec4::new(self.u.y, self.v.y, self.w.y, 0.0),
            Vec4::new(self.u.z, self.v.z, self.w.z, 0.0),
            (0, 0, 0, 1).into(),
        );

        let negative_from = -1.0 * self.origin;
        let translation = Mat4::translation::<f32>(negative_from.into());

        rotation * translation
    }

    pub fn projection_matrix(&self) -> Mat4 {
        let mut projection_matrix = Mat4::identity();
        let near_plane = self.near_plane;
        let far_plane = self.far_plane;
        let fov: f32 = self.vfov;
        let aspect_ratio = self.aspect;
        //let right = self.lower_left_corner.x + self.horizontal.magnitude();
        //let left = self.lower_left_corner.x;
        //let top = self.lower_left_corner.y + self.vertical.magnitude();
        //let bottom = self.lower_left_corner.y;

        let y_scale = 1.0 / (fov / 2.0).to_radians().tan();
        let x_scale = y_scale / aspect_ratio;
        let frustrum_length = far_plane - near_plane;

        projection_matrix.x.x = x_scale;

        projection_matrix.y.y = -y_scale;

        projection_matrix.z.z = -(near_plane + far_plane) / frustrum_length;
        projection_matrix.z.w = -(2.0 * near_plane * far_plane) / frustrum_length;

        projection_matrix.w.z = -1.0;
        projection_matrix.w.w = 0.0;

        println!("new projection: {:#?}", projection_matrix);

        /*
        let to_vk_ndc: Mat4 = (
            (1.0, 0.0, 0.0, 0.0),
            (0.0, -1.0, 0.0, 0.0),
            (0.0, 0.0, 0.5, 0.5),
            (0.0, 0.0, 0.0, 1.0),
        )
            .into();

        let gl = to_vk_ndc * projection_matrix;
        */
        projection_matrix
    }

    // reference https://learnopengl.com/code_viewer_gh.php?code=includes/learnopengl/camera.h
    pub fn update_orientation(&mut self) {
        // front vector
        self.w.x = self.yaw.to_radians().cos() * self.pitch.to_radians().cos();
        self.w.y = self.pitch.to_radians().sin();
        self.w.z = self.yaw.to_radians().sin() * self.pitch.to_radians().cos();
        self.w = self.w.make_unit_vector();

        self.u = self.w.cross(&self.world_up).make_unit_vector();
        self.v = self.u.cross(&self.w).make_unit_vector();
    }

    pub fn process_keypress(&mut self, key: VirtualKeyCode) {
        let speed = if self.fast_move {
            self.move_speed * 5.0
        } else {
            self.move_speed
        };

        match key {
            VirtualKeyCode::W => {
                self.origin -= speed * self.w;
            }
            VirtualKeyCode::A => {
                self.origin -= speed * self.u;
            }
            VirtualKeyCode::S => {
                self.origin += speed * self.w;
            }
            VirtualKeyCode::D => {
                self.origin += speed * self.u;
            }
            VirtualKeyCode::LShift => {
                self.fast_move = true;
            }
            _ => (),
        }
    }

    pub fn process_keyrelease(&mut self, key: VirtualKeyCode) {
        if let VirtualKeyCode::LShift = key {
            self.fast_move = false;
        };
    }

    pub fn process_mouse_move(&mut self, delta: Vec2) {
        let x_offset = delta.x * self.mouse_speed;
        let y_offset = delta.y * self.mouse_speed;

        self.yaw += x_offset;
        self.pitch += y_offset;

        self.pitch = if self.pitch > 89.0 {
            89.0
        } else if self.pitch < -89.0 {
            -89.0
        } else {
            self.pitch
        };

        self.update_orientation();
    }
}
