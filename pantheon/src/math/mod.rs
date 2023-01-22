pub mod mat2;
pub mod mat3;
pub mod mat4;
pub mod vec2;
pub mod vec3;
pub mod vec4;

pub mod quaternion;

pub mod prelude {
    pub use super::quaternion::Quaternion;

    pub use super::mat2::Mat2;
    pub use super::mat3::Mat3;
    pub use super::mat4::Mat4;
    pub use super::vec2::Vec2;
    pub use super::vec3::Vec3;
    pub use super::vec4::Vec4;

    pub use super::Dim;
    pub use super::Vector;
}

pub use quaternion::Quaternion;

pub use mat2::Mat2;
pub use mat3::Mat3;
pub use mat4::Mat4;
pub use vec2::Vec2;
pub use vec3::Vec3;
pub use vec4::Vec4;

unsafe impl bytemuck::Pod for Mat4 {}
unsafe impl bytemuck::Zeroable for Mat4 {}

unsafe impl bytemuck::Pod for Vec4 {}
unsafe impl bytemuck::Zeroable for Vec4 {}

pub enum Dim {
    X,
    Y,
    Z,
    W,
}

pub trait Vector:
    Sized + std::ops::Sub<Output = Self> + Copy + std::ops::Mul<f32, Output = Self>
{
    fn dot(&self, other: &Self) -> f32;

    #[inline]
    fn squared_mag(&self) -> f32 {
        self.dot(self)
    }

    #[inline]
    fn magnitude(&self) -> f32 {
        self.squared_mag().sqrt()
    }

    fn make_comp_mul(&self, other: &Self) -> Self;

    fn comp_mul(&mut self, other: &Self);

    fn make_comp_div(&self, other: &Self) -> Self;

    fn comp_div(&mut self, other: &Self);

    /// checked normalization, only performs scalar multiply of vector if not already mag ~= 1.0
    #[inline]
    fn unit_vector(&self) -> Self {
        let mag = self.magnitude();

        if (mag - 1.0).abs() > f32::EPSILON {
            let scalar = 1.0 / mag;
            *self * scalar
        } else {
            *self
        }
    }
    #[inline]
    fn normalize(&mut self) {
        *self = self.unit_vector();
    }

    fn approx_eq(&self, other: &Self) -> bool {
        (*self - *other).magnitude() <= f32::EPSILON
    }

    fn clamp(&self, min: f32, max: f32) -> Self;
}

#[macro_export]
macro_rules! simd_vector {
    ($vec:ty, $simd:ty) => {
        use std::simd::Simd;
        use std::simd::SimdFloat;

        impl Vector for $vec {
            #[inline]
            fn dot(&self, other: &Self) -> f32 {
                let (s, o): ($simd, $simd) = unsafe { std::mem::transmute((*self, *other)) };
                (s * o).reduce_sum()
            }
            #[inline]
            fn make_comp_mul(&self, other: &Self) -> Self {
                let (s, o): ($simd, $simd) = unsafe { std::mem::transmute((*self, *other)) };
                unsafe { std::mem::transmute(s * o) }
            }
            #[inline]
            fn comp_mul(&mut self, other: &Self) {
                // @SAFETY not safe, see `add_assign`
                unsafe {
                    let ptr: *mut $vec = std::mem::transmute(self);
                    let simd_ptr: *mut $simd = ptr as *mut $simd;
                    let o: $simd = std::mem::transmute(*other);
                    let s = &mut *simd_ptr;
                    *s *= o;
                }
            }
            #[inline]
            fn make_comp_div(&self, other: &Self) -> Self {
                let (s, o): ($simd, $simd) = unsafe { std::mem::transmute((*self, *other)) };
                unsafe { std::mem::transmute(s / o) }
            }
            #[inline]
            fn comp_div(&mut self, other: &Self) {
                // @SAFETY not safe, see `add_assign`
                unsafe {
                    let ptr: *mut $vec = std::mem::transmute(self);
                    let simd_ptr: *mut $simd = ptr as *mut $simd;
                    let o: $simd = std::mem::transmute(*other);
                    let s = &mut *simd_ptr;
                    *s /= o;
                }
            }

            #[inline]
            fn clamp(&self, min: f32, max: f32) -> Self {
                unsafe {
                    let s: $simd = std::mem::transmute(*self);

                    std::mem::transmute(s.simd_clamp(Simd::splat(min), Simd::splat(max)))
                }
            }
        }

        impl Add for $vec {
            type Output = Self;

            #[inline]
            fn add(self, other: Self) -> Self {
                let (s, o): ($simd, $simd) = unsafe { std::mem::transmute((self, other)) };
                unsafe { std::mem::transmute(s + o) }
            }
        }

        impl AddAssign for $vec {
            #[inline]
            fn add_assign(&mut self, other: Self) {
                // @SAFETY it is not. I believe this was segfaulting when `Vec3` had implemented
                // SIMD, not really sure the alternative here. The stranger part was that the Vec3
                // test was passing with this code being hit and not segfaulting, not sure what
                // caused it to happen in real usage
                unsafe {
                    let ptr: *mut $vec = std::mem::transmute(self);
                    let simd_ptr: *mut $simd = ptr as *mut $simd;
                    let o: $simd = std::mem::transmute(other);
                    let s = &mut *simd_ptr;
                    *s += o;
                }
            }
        }

        impl Sub for $vec {
            type Output = Self;

            #[inline]
            fn sub(self, other: Self) -> Self {
                let (s, o): ($simd, $simd) = unsafe { std::mem::transmute((self, other)) };
                unsafe { std::mem::transmute(s - o) }
            }
        }

        impl SubAssign for $vec {
            #[inline]
            fn sub_assign(&mut self, other: Self) {
                // @SAFETY not safe, see `add_assign`
                unsafe {
                    let ptr: *mut $vec = std::mem::transmute(self);
                    let simd_ptr: *mut $simd = ptr as *mut $simd;
                    let o: $simd = std::mem::transmute(other);
                    let s = &mut *simd_ptr;
                    *s -= o;
                }
            }
        }

        impl Mul<$vec> for f32 {
            type Output = $vec;

            #[inline]
            fn mul(self, vec: $vec) -> $vec {
                let s: $simd = unsafe { std::mem::transmute(vec) };

                unsafe { std::mem::transmute(s * Simd::splat(self)) }
            }
        }

        impl<T: Into<f32> + Copy> Mul<T> for $vec {
            type Output = Self;

            #[inline]
            fn mul(self, scalar: T) -> Self {
                let s: $simd = unsafe { std::mem::transmute(self) };

                unsafe { std::mem::transmute(s * Simd::splat(scalar.into())) }
            }
        }

        impl<T: Into<f32> + Copy> MulAssign<T> for $vec {
            #[inline]
            fn mul_assign(&mut self, scalar: T) {
                let s: &mut $simd = unsafe { std::mem::transmute(self) };
                *s *= Simd::splat(scalar.into());
            }
        }
    };
}
