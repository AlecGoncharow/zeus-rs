use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Debug, Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: impl Into<f32>, y: impl Into<f32>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }

    pub fn origin() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn squared_mag(&self) -> f32 {
        self.dot(self)
    }

    pub fn magnitude(&self) -> f32 {
        self.squared_mag().sqrt()
    }
}

impl<T: Into<f32>> From<(T, T)> for Vec2 {
    fn from(tuple: (T, T)) -> Self {
        Self::new(tuple.0, tuple.1)
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;

    fn mul(self, vec: Vec2) -> Vec2 {
        Vec2 {
            x: vec.x * self,
            y: vec.y * self,
        }
    }
}

impl<T: Into<f32> + Copy> Mul<T> for Vec2 {
    type Output = Self;

    fn mul(self, scalar: T) -> Self {
        Self {
            x: self.x * scalar.into(),
            y: self.y * scalar.into(),
        }
    }
}

impl<T: Into<f32> + Copy> MulAssign<T> for Vec2 {
    fn mul_assign(&mut self, scalar: T) {
        *self = Self {
            x: self.x * scalar.into(),
            y: self.y * scalar.into(),
        }
    }
}
