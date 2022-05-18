use std::path::Path;

use glow::{HasContext, NativeTexture};
use image::io::Reader as ImageReader;

use super::OpenGl;

pub struct Texture {
	texture: NativeTexture,
}

impl Texture {
	pub fn from_file<P: AsRef<Path>>(ogl: &OpenGl, path: P) -> Self {
		let img = ImageReader::open(path)
			.unwrap()
			.decode()
			.unwrap()
			.to_rgba8();

		let gl = ogl.gl();
		let texture = unsafe {
			let tex = gl.create_texture().unwrap();
			gl.bind_texture(glow::TEXTURE_2D, Some(tex));
			gl.tex_image_2d(
				glow::TEXTURE_2D,
				0,
				glow::RGBA as i32,
				img.width() as i32,
				img.height() as i32,
				0,
				glow::RGBA,
				glow::UNSIGNED_BYTE,
				Some(img.to_vec().as_slice()),
			);
			gl.generate_mipmap(glow::TEXTURE_2D);

			tex
		};

		Self { texture }
	}

	pub unsafe fn bind(&self, ogl: &OpenGl) {
		ogl.gl().bind_texture(glow::TEXTURE_2D, Some(self.texture));
	}

	pub unsafe fn delete(&self, ogl: &OpenGl) {
		ogl.gl().delete_texture(self.texture)
	}
}
