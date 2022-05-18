mod rectangle;
mod texture;
mod transform;

pub use rectangle::Rectangle;
pub use texture::Texture;
pub use transform::Transform;

use std::{cell::RefCell, path::Path as FilePath, rc::Rc};

use glow::{HasContext, Program};
use glutin::{window::Window, ContextWrapper, PossiblyCurrent};

use crate::{Color, Vec2};

pub struct OpenGl {
	gl: Rc<glow::Context>,
	transform: Transform,
	program: Program,
	clear_color: Color,
	draw_rect: Rectangle,
}

impl OpenGl {
	pub fn new(context: &ContextWrapper<PossiblyCurrent, Window>, transform: Transform) -> Self {
		let gl = unsafe {
			glow::Context::from_loader_function(|s| context.get_proc_address(s) as *const _)
		};

		// dirty hack to allow skip on this. Without let _ it would be an expression, and rustfmt::skip
		// on expressions is nightly
		#[rustfmt::skip]
        let _ = unsafe {
            // Repeat the texture if the coords go outside [0,1]
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);

            // Scale textures (mipmap?) using nearest neighbour which is better for pixely things
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);

            // Tell transparency do work how we'd expect.
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        };

		//TODO: This will only work in the smitten repo. Maybe we try to open them and fallback to ours if we fail?
		let program = unsafe { Self::create_program(&gl) };

		unsafe {
			gl.use_program(Some(program));
		}

		let draw_rect = Rectangle::new(&gl, (2.0, 2.0).into());

		Self {
			gl: Rc::new(gl),
			transform,
			program,
			clear_color: Color::rgba(0.0, 0.0, 0.0, 1.0),
			draw_rect,
		}
	}

	pub fn gl(&self) -> &glow::Context {
		&self.gl
	}

	pub fn clear_color<C: Into<Color>>(&mut self, color: C) {
		let c = color.into();
		self.clear_color = c;
		unsafe { self.gl.clear_color(c.r, c.g, c.b, c.a) }
	}

	pub fn clear(&self) {
		unsafe { self.gl.clear(glow::COLOR_BUFFER_BIT) }
	}

	pub fn resized(&mut self, width: u32, height: u32) {
		unsafe { self.gl.viewport(0, 0, width as i32, height as i32) }
		self.transform
			.resized(glutin::dpi::PhysicalSize { width, height })
	}

	unsafe fn create_program(gl: &glow::Context) -> Program {
		let program = gl.create_program().expect("Failed to create program");

		let shader_soruces = [
			(
				glow::VERTEX_SHADER,
				include_str!("../../shaders/texture.vert"),
			),
			(
				glow::FRAGMENT_SHADER,
				include_str!("../../shaders/texture.frag"),
			),
		];

		let mut shaders = vec![];
		for (stype, source) in shader_soruces.iter() {
			let shader = gl.create_shader(*stype).expect("Failed to create shader");
			gl.shader_source(shader, source);
			gl.compile_shader(shader);

			if !gl.get_shader_compile_status(shader) {
				panic!("{}", gl.get_shader_info_log(shader));
			}

			gl.attach_shader(program, shader);
			shaders.push(shader);
		}

		gl.link_program(program);
		if !gl.get_program_link_status(program) {
			panic!("{}", gl.get_program_info_log(program));
		}

		// Shaders are compiled and linked with the program, we don't need them anymore
		for shader in shaders {
			gl.detach_shader(program, shader);
			gl.delete_shader(shader);
		}

		program
	}

	pub fn set_color_uniform(&self, color: Color) {
		unsafe {
			let uniform = self.gl.get_uniform_location(self.program, "Color");
			self.gl
				.uniform_4_f32(uniform.as_ref(), color.r, color.g, color.b, color.a);
		}
	}

	pub fn draw_rectangle(&self, pos: Vec2, dim: Vec2) {
		// The rectangle we use to draw, self.draw_rect, spans from (OpenGL Normalized Coordinates)
		// -1,1 to 1,-1. That means any scale we appply via our little uniform will be 2x, as it
		// multiplies both verticies away from the center.

		let gl_pos = self.transform.vec_to_opengl(pos);
		let gl_dim = self.transform.vec_to_opengl(dim / 2);

		unsafe {
			//self.gl.use_program(Some(self.program));

			let uniform_position = self.gl.get_uniform_location(self.program, "WorldPosition");
			let uniform_scale = self.gl.get_uniform_location(self.program, "Scale");
			self.gl
				.uniform_2_f32(uniform_position.as_ref(), gl_pos.x, gl_pos.y);
			self.gl
				.uniform_2_f32(uniform_scale.as_ref(), gl_dim.x, gl_dim.y);

			self.draw_rect.bind(&self.gl);
			self.gl
				.draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_BYTE, 0);
		}
	}

	pub fn draw_fullscreen(&self) {
		unsafe {
			//self.gl.use_program(Some(self.program));

			let uniform_position = self.gl.get_uniform_location(self.program, "WorldPosition");
			let uniform_scale = self.gl.get_uniform_location(self.program, "Scale");
			self.gl.uniform_2_f32(uniform_position.as_ref(), 0.0, 0.0);
			self.gl.uniform_2_f32(uniform_scale.as_ref(), 1.0, 1.0);

			self.draw_rect.bind(&self.gl);
			self.gl
				.draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_BYTE, 0);
		}
	}
}

impl Drop for OpenGl {
	fn drop(&mut self) {
		unsafe {
			self.gl.delete_program(self.program);
			self.draw_rect.delete(&self.gl);
		}
	}
}
