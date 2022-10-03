use std::{
	collections::HashMap,
	fs::File,
	io::{BufWriter, Read},
	path::Path,
};

use fontdue::Font;
use image::ImageBuffer;

use crate::{
	gl::{OpenGl, Rectangle, Texture},
	Vec2,
};

pub struct SmittenFont {
	pub font: Font,
	pub packed: PackedFont,
}

impl SmittenFont {
	pub fn from_file<P: AsRef<Path>>(gl: &OpenGl, path: P) -> Self {
		let font = parse_font_file(path);
		let packed = layout_texture(&font, gl);

		Self { font, packed }
	}
}

fn layout_texture(font: &Font, gl: &OpenGl) -> PackedFont {
	let size = 64.0f32;

	let width = (size * 17.0).ceil() as usize;
	let height = (size * 7.0).ceil() as usize;

	let mut characters = HashMap::new();

	let mut image = vec![0; width * height * 4];

	for ch_index in 32..128 {
		let ch = char::from_u32(ch_index).unwrap();
		let (metrics, raster) = font.rasterize(ch, size);

		let (x_over, y_down) = {
			let idx = ch_index as usize - 32;
			let x = idx % 16;
			let y = idx / 16;

			(x * size as usize, y * size as usize)
		};

		for x in 0..metrics.width {
			for y in 0..metrics.height {
				let raster_idx = y * metrics.width + x;
				let image_idx = ((y_down + y) * width + (x_over + x)) * 4;

				image[image_idx] = 255;
				image[image_idx + 1] = 255;
				image[image_idx + 2] = 255;
				image[image_idx + 3] = raster[raster_idx];
			}
		}

		let size = Vec2::new(metrics.width as f32, metrics.height as f32);
		let texture_position =
			Vec2::new(x_over as f32 / width as f32, y_down as f32 / height as f32);
		let texture_dimensions = Vec2::new(
			metrics.width as f32 / width as f32,
			metrics.height as f32 / height as f32,
		);

		println!("{ch} {texture_position} {texture_dimensions}");

		let rect = Rectangle::with_texture_coordinates(
			gl.gl(),
			/*size.normalize(),*/ Vec2::new(2.0, 2.0),
			texture_position,
			texture_dimensions,
		);

		characters.insert(
			ch,
			PackedCharacter {
				size,
				texture_position,
				texture_dimensions,
				rect,
			},
		);
	}

	let texture = Texture::rgba8(gl, width, height, &image);

	let file = File::create("font.png").unwrap();
	let ref mut w = BufWriter::new(file);
	let mut encoder = png::Encoder::new(w, width as u32, height as u32);
	encoder.set_color(png::ColorType::Rgba);
	encoder.set_depth(png::BitDepth::Eight);
	encoder
		.write_header()
		.unwrap()
		.write_image_data(&image)
		.unwrap();

	PackedFont {
		texture,
		characters,
	}
}

pub struct PackedFont {
	pub texture: Texture,
	pub characters: HashMap<char, PackedCharacter>,
}

pub struct PackedCharacter {
	pub size: Vec2,
	texture_position: Vec2,
	texture_dimensions: Vec2,

	pub rect: Rectangle,
}

pub fn parse_font_file<P: AsRef<Path>>(path: P) -> Font {
	/*let mut file = File::open(path.as_ref()).unwrap();
	let mut buffer = vec![];
	file.read_to_end(&mut buffer).unwrap();*/
	let buffer = include_bytes!("Hack-Regular.ttf");

	parse_font(buffer)
}

pub fn parse_font(data: &[u8]) -> Font {
	Font::from_bytes(data, Default::default()).unwrap()
}
