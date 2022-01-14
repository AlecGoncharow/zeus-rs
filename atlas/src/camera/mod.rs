use pantheon::math::Mat3;
use pantheon::math::Mat4;
use pantheon::math::Vec2;
use pantheon::math::Vec3;
use pantheon::winit::event::VirtualKeyCode;

//const YAW_DEFAULT: f32 = -90.0;
//const PITCH_DEFAULT: f32 = 0.0;
const MOVE_DEFAULT: f32 = 5.;
const MOUSE_DEFAULT: f32 = 100.;
const EPSILON: f32 = 9.53674316 * 0.00000001;
const INFINITE_PERSPECTIVE: bool = false;

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

    // projection matrix things, not sure if it should be here
    pub aspect: f32,
    pub vfov: f32,
    pub near_plane: f32,
    pub far_plane: f32,

    pub projection: Mat4,
    pub view: Mat4,
    pub translation: Mat4,
    pub translation_inv: Mat4,

    pub reflected_view: Mat4,
    pub u_r: Vec3,
    pub v_r: Vec3,
    pub w_r: Vec3,
    pub camera_flip: Vec3,

    pub dirty: bool,

    pub infinite_perspective: bool,
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
        let w = (look_from - look_at).unit_vector();
        let u = (w.cross(&world_up)).unit_vector();
        let v = u.cross(&w).unit_vector();

        let reflected_look_from = look_from.make_comp_mul(&(0, -1, 0).into());
        let w_r = (reflected_look_from - look_at).unit_vector();
        let u_r = (w.cross(&world_up)).unit_vector();
        let v_r = u.cross(&w).unit_vector();

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
        let projection = if INFINITE_PERSPECTIVE {
            Mat4::infinite_perspective(vfov, aspect, near_plane, EPSILON)
        } else {
            Mat4::reverse_infinite_perspective(vfov, aspect, near_plane, EPSILON)
        };

        let uvw = Mat3::new(u, v, w);
        let rotation = uvw.transpose().mat4();

        let negative_from = -1.0 * origin;
        let translation = Mat4::translation::<f32>(negative_from.into());
        let view = rotation * translation;
        // @TODO @MATH idk if this is correct
        /*
        let mut transform = uvw.mat4();
        transform.w = origin.vec4_with(1.0);
        */
        //let transform = view.invert().unwrap();

        let uvw_r = Mat3::new(u_r, v_r, w_r);
        let rotation = uvw_r.transpose().mat4();

        let negative_from = -1.0 * reflected_look_from;
        let translation = Mat4::translation::<f32>(origin.into());
        let translation_inv = Mat4::translation::<f32>(negative_from.into());
        let reflected_view = rotation * translation;

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

            projection,
            view,
            translation,
            translation_inv,

            reflected_view,
            u_r,
            v_r,
            w_r,
            camera_flip: Vec3::new(1, -1, 1),
            dirty: false,

            infinite_perspective: INFINITE_PERSPECTIVE,
        }
    }

    pub fn set_aspect(&mut self, (width, height): (f32, f32)) {
        self.aspect = width / height;
    }

    pub fn update_view_matrix(&mut self) -> Mat4 {
        let uvw = Mat3::new(self.u, self.v, self.w);
        let rotation = uvw.transpose().mat4();

        /*
        self.transform = uvw.mat4();
        self.transform.w = self.origin.vec4_with(1.0);
        */

        let negative_from = -1.0 * self.origin;
        self.translation_inv = Mat4::translation::<f32>(negative_from.into());
        self.translation = Mat4::translation::<f32>(self.origin.into());

        self.view = rotation * self.translation_inv;

        //self.transform = self.view.invert().unwrap();

        self.view
    }

    pub fn update_reflected_view_matrix(&mut self) -> Mat4 {
        let rotation = Mat3::new(self.u_r, self.v_r, self.w_r).transpose().mat4();

        //let negative_from: Vec3 = -1.0 * Vec3::new(self.origin.x, -5., self.origin.z);
        let negative_from: Vec3 = -1.0 * self.origin.make_comp_mul(&self.camera_flip);
        let translation = Mat4::translation::<f32>(negative_from.into());

        self.reflected_view = rotation * translation;

        self.reflected_view
    }

    pub fn update_projection_matrix(&mut self) -> Mat4 {
        self.dirty = true;
        let projection_matrix = if self.infinite_perspective {
            Mat4::infinite_perspective(self.vfov, self.aspect, self.near_plane, EPSILON)
        } else {
            Mat4::reverse_infinite_perspective(self.vfov, self.aspect, self.near_plane, EPSILON)
        };

        /*
        let to_vk_ndc: Mat4 =
            (1.0, 0.0, 0.0, 0.0),
            (0.0, -1.0, 0.0, 0.0),
            (0.0, 0.0, 0.5, 0.5),
            (0.0, 0.0, 0.0, 1.0),
        )
            .into();

        let gl = to_vk_ndc * projection_matrix;
        */
        self.projection = projection_matrix;
        projection_matrix
    }

    // reference https://learnopengl.com/code_viewer_gh.php?code=includes/learnopengl/camera.h
    pub fn update_orientation(&mut self) {
        self.dirty = true;
        // front vector
        self.w.x = self.yaw.to_radians().cos() * self.pitch.to_radians().cos();
        self.w.y = self.pitch.to_radians().sin();
        self.w.z = self.yaw.to_radians().sin() * self.pitch.to_radians().cos();
        self.w = self.w.unit_vector();

        self.u = self.w.cross(&self.world_up).unit_vector();
        self.v = self.u.cross(&self.w).unit_vector();

        let pitch_flip = -self.pitch;
        self.w_r.x = self.yaw.to_radians().cos() * pitch_flip.to_radians().cos();
        self.w_r.y = pitch_flip.to_radians().sin();
        self.w_r.z = self.yaw.to_radians().sin() * pitch_flip.to_radians().cos();
        self.w_r = self.w_r.unit_vector();

        self.u_r = self.w_r.cross(&self.world_up).unit_vector();
        self.v_r = self.u_r.cross(&self.w_r).unit_vector();
    }

    pub fn process_keypress(&mut self, key: VirtualKeyCode, delta_time: f32) {
        let speed = if self.fast_move {
            self.move_speed * 5.0 * delta_time
        } else {
            self.move_speed * delta_time
        };

        match key {
            VirtualKeyCode::W => {
                self.dirty = true;
                self.origin -= speed * self.w;
            }
            VirtualKeyCode::A => {
                self.dirty = true;
                self.origin -= speed * self.u;
            }
            VirtualKeyCode::S => {
                self.dirty = true;
                self.origin += speed * self.w;
            }
            VirtualKeyCode::D => {
                self.dirty = true;
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

    pub fn process_mouse_move(&mut self, delta: Vec2, delta_time: f32) {
        let x_offset = -delta.x * self.mouse_speed * delta_time;
        let y_offset = delta.y * self.mouse_speed * delta_time;

        self.yaw += x_offset;
        self.pitch += y_offset;

        self.pitch = if self.pitch > 89.0 {
            println!("Clamping Camera pitch to 89 deg");
            89.0
        } else if self.pitch < -89.0 {
            println!("Clamping Camera pitch to -89 deg");
            -89.0
        } else {
            self.pitch
        };

        self.update_orientation();
    }
}
