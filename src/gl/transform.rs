use glutin::dpi::PhysicalSize;

use crate::vec2::Vec2;

// File pulled from https://github.com/gennyble/notsure/blob/main/src/main.rs#L30

/// OpenGL's normalized coordinates are relative to each axis, which would make
/// sizing something like a square quite difficult. So we have our own unit of
/// measurement, Murs.
pub struct Transform {
	dpi_scale: f32,
	screen_size: PhysicalSize<u32>,
	pub screen_vec: Vec2,
	pub mur_dimensions: Vec2,
	pub mur_half_dimensions: Vec2,
	pub mur_size: u32,
}

impl Transform {
	/// The mur_size is the number of pixels per Mur.
	pub fn new(screen_size: PhysicalSize<u32>, mur_size: u32) -> Self {
		let mur_dimensions = Vec2::from(screen_size) / mur_size;

		Self {
			dpi_scale: 1.0,
			screen_size,
			screen_vec: screen_size.into(),
			mur_half_dimensions: mur_dimensions / 2.0,
			mur_dimensions,
			mur_size,
		}
	}

	pub fn resized(&mut self, screen_size: PhysicalSize<u32>) {
		self.screen_size = screen_size;
		self.screen_vec = screen_size.into();
	}

	pub fn vec_to_opengl(&self, vec: Vec2) -> Vec2 {
		//vec.y *= -1.0;
		(vec * self.mur_size) / (self.screen_vec / 2)
	}

	pub fn window_vec_to_murs(&self, mut vec: Vec2) -> Vec2 {
		vec.y = self.screen_vec.y - vec.y;
		vec = (vec - (self.screen_vec / 2)) / (self.screen_vec);
		vec.x *= self.mur_dimensions.x;
		vec.y *= self.mur_dimensions.y;
		vec
	}

	pub fn vec_to_pixels(&self, vec: Vec2) -> Vec2 {
		vec * self.mur_size + self.screen_vec / 2
	}

	pub fn pixels_to_mur(&self, pixels: u32) -> f32 {
		pixels as f32 / self.mur_size as f32
	}
}
