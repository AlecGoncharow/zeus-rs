use std::mem::MaybeUninit;

use crate::camera::Camera;
use pantheon::math::prelude::*;
use pantheon::Color;

pub const CASCADE_COUNT: usize = 4;
pub const A_DISTS: [f32; CASCADE_COUNT] = [0., 10., 50., 200.];
pub const B_DISTS: [f32; CASCADE_COUNT] = [25., 100., 300., 500.];
pub const MAP_SIZE: f32 = 1024.;

/*
* let d = (verts[0] - verts[6])
           .magnitude()
           .max((verts[4] - verts[6]).magnitude())
           .ceil();
   /// 8.73
   pub fn pos(&self, t: f32) -> Vec3 {
       Vec3::new(
           ((self.x_max + self.x_min) / (2.0 * t)).floor() * t,
           ((self.y_max + self.y_min) / (2.0 * t)).floor() * t,
           self.z_min,
       )
   }
 */

#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub near: f32,
    pub far: f32,
    pub verts: [Vec3; 8],
}

impl BoundingBox {
    pub fn new(near: f32, far: f32, aspect_ratio: f32, fovy: f32) -> Self {
        let a = near;
        let b = far;
        let s = aspect_ratio;
        let g = (fovy * 0.5).atan();

        BoundingBox {
            near,
            far,
            verts: [
                Vec3::new(a * s / g, a / g, a),
                Vec3::new(a * s / g, -a / g, a),
                Vec3::new(-a * s / g, a / g, a),
                Vec3::new(-a * s / g, -a / g, a),
                //
                Vec3::new(b * s / g, b / g, b),
                Vec3::new(b * s / g, -b / g, b),
                Vec3::new(-b * s / g, b / g, b),
                Vec3::new(-b * s / g, -b / g, b),
            ],
        }
    }

    #[inline]
    pub fn update(&mut self, aspect_ratio: f32, fovy: f32) {
        let a = self.near;
        let b = self.far;
        let s = aspect_ratio;
        let g = (fovy * 0.5).atan();

        self.verts = [
            Vec3::new(a * s / g, a / g, a),
            Vec3::new(a * s / g, -a / g, a),
            Vec3::new(-a * s / g, a / g, a),
            Vec3::new(-a * s / g, -a / g, a),
            //
            Vec3::new(b * s / g, b / g, b),
            Vec3::new(b * s / g, -b / g, b),
            Vec3::new(-b * s / g, b / g, b),
            Vec3::new(-b * s / g, -b / g, b),
        ];
    }

    #[inline]
    pub fn map_size(&self) -> f32 {
        (self.verts[0] - self.verts[6])
            .magnitude()
            .max((self.verts[4] - self.verts[6]).magnitude())
            .ceil()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MinMax {
    pub x_min: f32,
    pub y_min: f32,
    pub z_min: f32,

    pub x_max: f32,
    pub y_max: f32,
    pub z_max: f32,
}

impl MinMax {
    pub fn new() -> Self {
        Self {
            x_min: std::f32::MAX,
            y_min: std::f32::MAX,
            z_min: std::f32::MAX,

            x_max: std::f32::MIN,
            y_max: std::f32::MIN,
            z_max: std::f32::MIN,
        }
    }

    pub fn init(bounding_box: &BoundingBox, light_space_transform: &Mat4) -> Self {
        let mut min_max = MinMax::new();

        min_max.update(bounding_box, light_space_transform);

        min_max
    }

    #[inline]
    pub fn reset(&mut self) {
        *self = Self {
            x_min: std::f32::MAX,
            y_min: std::f32::MAX,
            z_min: std::f32::MAX,

            x_max: std::f32::MIN,
            y_max: std::f32::MIN,
            z_max: std::f32::MIN,
        };
    }

    #[inline]
    pub fn update(&mut self, bounding_box: &BoundingBox, light_space_transform: &Mat4) {
        self.reset();

        for vert in bounding_box.verts {
            let vert = light_space_transform * vert.vec4();

            if vert.x > self.x_max {
                self.x_max = vert.x;
            }
            if vert.x < self.x_min {
                self.x_min = vert.x;
            }

            if vert.y > self.y_max {
                self.y_max = vert.y;
            }
            if vert.y < self.y_min {
                self.y_min = vert.y;
            }

            if vert.z > self.z_max {
                self.z_max = vert.z;
            }
            if vert.z < self.z_min {
                self.z_min = vert.z;
            }
        }
    }

    #[inline]
    pub fn pos(&self, t: f32) -> Vec3 {
        Vec3::new(
            ((self.x_max + self.x_min) / (2.0 * t)).floor() * t,
            ((self.y_max + self.y_min) / (2.0 * t)).floor() * t,
            self.z_min,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Cascade {
    /// 8.63 : 8.64
    pub bounding_box: BoundingBox,
    /// 8.65
    pub min_max: MinMax,

    /// 8.69
    pub projection: Mat4,
    /// 8.74
    pub cascade_space_transform: Mat4,

    /// 8.71
    pub d: f32,
    /// 8.72
    pub t: f32,
    /// 8.73
    pub s: Vec3,
}

impl Cascade {
    pub fn new(bounding_box: BoundingBox, min_max: MinMax, transpose_light_rotation: Mat4) -> Self {
        let d = bounding_box.map_size();
        let t = d / MAP_SIZE;
        let s = min_max.pos(t);
        let mut projection = Mat4::identity();
        projection.x.x = 2.0 / d;
        projection.y.y = 2.0 / d;
        projection.z.z = 1.0 / (min_max.z_max - min_max.z_min);

        let translation = Mat4::translation::<f32>((-1.0f32 * s).into());
        let cascade_space_transform = transpose_light_rotation * translation;
        //let mut cascade_space_transform = transpose_light_rotation;
        //cascade_space_transform.w = (-1.0f32 * s).vec4_with(1.0);

        Self {
            bounding_box,
            min_max,

            projection,
            cascade_space_transform,

            d,
            t,
            s,
        }
    }

    pub fn update(&mut self, transpose_light_rotation: Mat4, light_space_transform: &Mat4) {
        self.min_max
            .update(&self.bounding_box, light_space_transform);
        self.d = self.bounding_box.map_size();
        self.t = self.d / MAP_SIZE;
        self.s = self.min_max.pos(self.t);

        self.projection.x.x = 2.0 / self.d;
        self.projection.y.y = 2.0 / self.d;
        self.projection.z.z = 1.0 / (self.min_max.z_max - self.min_max.z_min);

        let translation = Mat4::translation::<f32>((-1.0f32 * self.s).into());
        self.cascade_space_transform = transpose_light_rotation * translation
        //self.cascade_space_transform = transpose_light_rotation;
        //self.cascade_space_transform.w = (-1.0f32 * self.s).vec4_with(1.0);
    }

    pub fn resize(&mut self, aspect_ratio: f32, fovy: f32) {
        self.bounding_box.update(aspect_ratio, fovy);
    }

    #[inline]
    pub fn view_projection(&self) -> Mat4 {
        self.projection * self.cascade_space_transform
    }
}

pub struct Infinite {
    pub direction: Vec3,
    pub world_up: Vec3,

    // may not need this idk
    pub fovy: f32,
    pub aspect_ratio: f32,

    pub uvw: Mat3,
    pub transform: Mat4,
    pub transform_transpose: Mat4,
    /// 8.62
    pub light_space_transform: Mat4,

    pub cascades: [Cascade; CASCADE_COUNT],

    pub color: Color,
}

impl Infinite {
    pub fn new(camera: &Camera, direction: Vec3, color: Color) -> Self {
        let mut w = direction.unit_vector();
        w.y *= -1.0;

        let u = w.cross(&camera.world_up).unit_vector();
        let v = u.cross(&w).unit_vector();
        let uvw = Mat3::new(u, v, w);

        let transform = uvw.mat4();
        let transform_transpose = uvw.transpose().mat4();

        let light_space_transform = transform_transpose * camera.transform;

        let fovy = camera.vfov;
        let aspect_ratio = camera.aspect;

        let cascades = {
            let mut arr: [MaybeUninit<Cascade>; CASCADE_COUNT] =
                unsafe { MaybeUninit::uninit().assume_init() };

            for (i, (a, b)) in A_DISTS.iter().zip(B_DISTS.iter()).enumerate() {
                let bounding_box = BoundingBox::new(*a, *b, aspect_ratio, fovy);
                let min_max = MinMax::init(&bounding_box, &light_space_transform);

                arr[i].write(Cascade::new(bounding_box, min_max, transform_transpose));
            }

            unsafe { std::mem::transmute(arr) }
        };

        Self {
            direction,
            world_up: camera.world_up,
            fovy,
            aspect_ratio,
            uvw,

            transform,
            transform_transpose,
            light_space_transform,

            cascades,
            color,
        }
    }

    pub fn update(&mut self, camera: &Camera) {
        self.light_space_transform = self.transform_transpose * camera.transform;

        self.cascades.iter_mut().for_each(|cascade| {
            cascade.update(self.transform_transpose, &self.light_space_transform)
        });
    }

    pub fn resize(&mut self, camera: &Camera) {
        self.aspect_ratio = camera.aspect;
        self.fovy = camera.vfov;

        self.cascades
            .iter_mut()
            .for_each(|cascade| cascade.resize(self.aspect_ratio, self.fovy));
    }
}
