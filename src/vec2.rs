use core::fmt;
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

use glutin::dpi::PhysicalSize;

#[derive(Copy, Clone, Debug)]
pub struct Vec2 {
	pub x: f32,
	pub y: f32,
}

impl Vec2 {
	pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

	pub fn new(x: f32, y: f32) -> Self {
		Self { x, y }
	}

	pub fn normalize<P: Into<Vec2>>(self, other: P) -> Self {
		let other = other.into();
		Self {
			x: (self.x * 2.0) / other.x - 1.0,
			y: ((other.y - self.y) * 2.0) / other.y - 1.0,
		}
	}

	pub fn abs(self) -> Self {
		Self {
			x: self.x.abs(),
			y: self.y.abs(),
		}
	}
}

impl Add for Vec2 {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
		}
	}
}

impl AddAssign for Vec2 {
	fn add_assign(&mut self, rhs: Self) {
		self.x += rhs.x;
		self.y += rhs.y;
	}
}

impl Sub for Vec2 {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x - rhs.x,
			y: self.y - rhs.y,
		}
	}
}

impl SubAssign for Vec2 {
	fn sub_assign(&mut self, rhs: Self) {
		self.x -= rhs.x;
		self.y -= rhs.y;
	}
}

impl Mul<f32> for Vec2 {
	type Output = Vec2;

	fn mul(self, rhs: f32) -> Self::Output {
		Self {
			x: self.x * rhs,
			y: self.y * rhs,
		}
	}
}

impl Mul<u32> for Vec2 {
	type Output = Vec2;

	fn mul(self, rhs: u32) -> Self::Output {
		self * rhs as f32
	}
}

impl Div<f32> for Vec2 {
	type Output = Vec2;

	fn div(self, rhs: f32) -> Self::Output {
		Self {
			x: self.x / rhs,
			y: self.y / rhs,
		}
	}
}

impl Div<u32> for Vec2 {
	type Output = Vec2;

	fn div(self, rhs: u32) -> Self::Output {
		self / rhs as f32
	}
}

impl Div for Vec2 {
	type Output = Self;

	fn div(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x / rhs.x,
			y: self.y / rhs.y,
		}
	}
}

impl From<(f32, f32)> for Vec2 {
	fn from(t: (f32, f32)) -> Self {
		Self { x: t.0, y: t.1 }
	}
}

impl From<(i32, i32)> for Vec2 {
	fn from(t: (i32, i32)) -> Self {
		Self {
			x: t.0 as f32,
			y: t.1 as f32,
		}
	}
}

impl From<(u32, u32)> for Vec2 {
	fn from(t: (u32, u32)) -> Self {
		Self {
			x: t.0 as f32,
			y: t.1 as f32,
		}
	}
}

impl From<PhysicalSize<f32>> for Vec2 {
	fn from(p: PhysicalSize<f32>) -> Self {
		Self {
			x: p.width,
			y: p.height,
		}
	}
}

impl From<PhysicalSize<u32>> for Vec2 {
	fn from(p: PhysicalSize<u32>) -> Self {
		Self {
			x: p.width as f32,
			y: p.height as f32,
		}
	}
}

impl fmt::Display for Vec2 {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "({}, {})", self.x, self.y)
	}
}
