use core::fmt;
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

use glutin::dpi::PhysicalSize;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vec2 {
	pub x: f32,
	pub y: f32,
}

impl Vec2 {
	pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

	pub const fn new(x: f32, y: f32) -> Self {
		Self { x, y }
	}

	pub fn gl_normalize<P: Into<Vec2>>(self, other: P) -> Self {
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

	pub fn operation<F>(self, f: F) -> Self
	where
		F: Fn(f32) -> f32,
	{
		Self {
			x: f(self.x),
			y: f(self.y),
		}
	}

	pub fn invert(self, x: bool, y: bool) -> Self {
		let x = if x { -1.0 } else { 1.0 };
		let y = if y { -1.0 } else { 1.0 };

		Self {
			x: self.x * x,
			y: self.y * y,
		}
	}

	//FIXME: what should we call this?
	pub fn normalize(&self) -> Self {
		let max = self.x.max(self.y);

		Self {
			x: self.x / max,
			y: self.y / max,
		}
	}

	//TODO: fix code
	pub fn normalize_correct(&self) -> Self {
		let length = ((self.x * self.x) + (self.y * self.y)).sqrt();

		if length == 0.0 {
			return Vec2::ZERO;
		}

		Self {
			x: self.x / length,
			y: self.y / length,
		}
	}

	pub fn distance_with(&self, other: Vec2) -> f32 {
		((other.x - self.x) * (other.x - self.x) + (other.y - self.y) * (other.y - self.y)).sqrt()
	}

	pub fn length(&self) -> f32 {
		((self.x * self.x) + (self.y * self.y)).sqrt()
	}

	pub fn dot(&self, other: Vec2) -> f32 {
		(self.x * other.x) + (self.y * other.y)
	}

	/// Angle of this vector with (0, 1)
	pub fn angle(&self) -> f32 {
		if self.x < 0.0 {
			360.0 - self.dot(Vec2::new(0.0, 1.0)).acos().to_degrees()
		} else {
			self.dot(Vec2::new(0.0, 1.0)).acos().to_degrees()
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
