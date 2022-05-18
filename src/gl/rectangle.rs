use glow::{Buffer, HasContext, VertexArray};

use crate::Vec2;

pub struct Rectangle {
	vao: VertexArray,
	vbo: Buffer,
	ebo: Buffer,
}

impl Rectangle {
	pub fn new(gl: &glow::Context, size: Vec2) -> Self {
		Self::with_texture_coordinates(gl, size, (0.0, 0.0).into(), (1.0, 1.0).into())
	}

	pub fn with_texture_coordinates(
		gl: &glow::Context,
		size: Vec2,
		tex_pos: Vec2,
		tex_dim: Vec2,
	) -> Self {
		let hx = size.x / 2.0;
		let hy = size.y / 2.0;

		let tex_far = tex_pos + tex_dim;

		#[rustfmt::skip]
        let verticies = [
            hx, hy, tex_far.x, tex_pos.y, // top right
            hx, -hy, tex_far.x, tex_far.y, // bottom right
            -hx, -hy, tex_pos.x, tex_far.y, // bottom left,
            -hx, hy, tex_pos.x, tex_pos.y, // top left
        ];

		#[rustfmt::skip]
        let indicies = [
            0, 1, 3,
            1, 2, 3u8
        ];

		let mut verticie_buffer = vec![];
		for vertex in verticies {
			verticie_buffer.extend_from_slice(&vertex.to_le_bytes());
		}

		let (vao, vbo, ebo) = unsafe {
			let vao = gl.create_vertex_array().unwrap();
			gl.bind_vertex_array(Some(vao));

			let vbo = gl.create_buffer().unwrap();
			gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
			gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &verticie_buffer, glow::DYNAMIC_DRAW);

			let ebo = gl.create_buffer().unwrap();
			gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
			gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, &indicies, glow::DYNAMIC_DRAW);

			gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 4 * 4, 0);
			gl.enable_vertex_attrib_array(0);

			gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 4 * 4, 2 * 4);
			gl.enable_vertex_attrib_array(1);

			gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
			gl.bind_buffer(glow::ARRAY_BUFFER, None);
			gl.bind_vertex_array(None);

			(vao, vbo, ebo)
		};

		Self { vao, vbo, ebo }
	}

	pub unsafe fn bind(&self, gl: &glow::Context) {
		gl.bind_vertex_array(Some(self.vao));
		gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.ebo));
	}

	pub unsafe fn delete(&self, gl: &glow::Context) {
		gl.delete_vertex_array(self.vao);
	}
}
