#![feature(const_fn_floating_point_arithmetic)]
mod color;
mod gl;
mod smittenfont;
mod vec2;

use smittenfont::SmittenFont;

use std::{
	cell::Cell,
	collections::{HashMap, HashSet},
	path::Path,
};

use gl::{OpenGl, Texture, TextureColoring, Transform};
use glutin::{
	dpi::PhysicalSize,
	event::{ElementState, Event, VirtualKeyCode, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	platform::run_return::EventLoopExtRunReturn,
	window::{Window, WindowBuilder},
	ContextBuilder, ContextWrapper, PossiblyCurrent,
};

#[cfg(target_os = "linux")]
use glutin::platform::unix::WindowBuilderExtUnix;

pub use color::Color;
pub use gl::SignedDistance;
pub use glutin::event::MouseButton;
pub use vec2::Vec2;

pub type PixelSize = PhysicalSize<u32>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TextureId(u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FontId(u32);

struct InputState {
	down_keys: HashSet<Key>,
	down_scancode: HashSet<u32>,
	mouse_position: Vec2,
}

impl InputState {
	pub fn new() -> Self {
		Self {
			down_keys: HashSet::new(),
			down_scancode: HashSet::new(),
			mouse_position: Vec2::ZERO,
		}
	}

	pub fn set_keycode_state(&mut self, scancode: u32, key: Option<VirtualKeyCode>, down: bool) {
		if down {
			self.down_scancode.insert(scancode);

			if let Some(Ok(key)) = key.map(|v| v.try_into()) {
				self.down_keys.insert(key);
			}
		} else {
			self.down_scancode.remove(&scancode);

			if let Some(Ok(key)) = key.map(|v| v.try_into()) {
				self.down_keys.remove(&key);
			}
		}
	}
}

//TODO: Custom drop that frees resources

pub struct Smitten {
	context: ContextWrapper<PossiblyCurrent, Window>,
	event_loop: EventLoop<()>,

	gl: OpenGl,
	current_color: Cell<Color>,
	current_texture: Cell<Option<TextureId>>,
	texture_coloring: TextureColoring,

	next_textureid: TextureId,
	textures: HashMap<TextureId, Texture>,

	next_fontid: FontId,
	fonts: HashMap<FontId, SmittenFont>,

	input_state: InputState,
}

impl Smitten {
	/// Make a new window and set everyththing up
	pub fn new<P, T>(size: P, title: T, mur: u32) -> Smitten
	where
		P: Into<PixelSize>,
		T: Into<String>,
	{
		let size = size.into();
		let el = EventLoop::new();

		// The wayland app id "pleasefloat" will make the window floating on
		// sway if you have the following in your config:
		// for_window [app_id="pleasefloat"] floating enable
		#[cfg(target_os = "linux")]
		let wb = WindowBuilder::new()
			.with_title(title)
			.with_inner_size(size)
			.with_app_id("pleasefloat".into());

		#[cfg(not(target_os = "linux"))]
		let wb = WindowBuilder::new().with_title(title).with_inner_size(size);

		let wc = ContextBuilder::new()
			.with_vsync(true)
			.build_windowed(wb, &el)
			.unwrap();

		//TODO: Add saftey note
		let context = unsafe { wc.make_current().unwrap() };
		let mut gl = OpenGl::new(&context, Transform::new(size, mur));

		gl.clear_color(Color::rgb(0.0, 0.0, 0.0));
		gl.set_texture_coloring_uniform(TextureColoring::Texture);

		Smitten {
			context,
			event_loop: el,
			gl,
			current_color: Cell::new(Color::rgb(0.0, 0.0, 0.0)),
			current_texture: Cell::new(None),
			texture_coloring: TextureColoring::Texture,
			next_textureid: TextureId(0),
			textures: HashMap::new(),
			next_fontid: FontId(0),
			fonts: HashMap::new(),
			input_state: InputState::new(),
		}
	}

	pub fn events(&mut self) -> Vec<SmittenEvent> {
		let mut events = vec![];
		self.event_loop.run_return(|event, _, flow| {
			Self::add_event(&mut self.input_state, &mut events, event, flow);
		});

		for event in &events {
			match event {
				SmittenEvent::WindowResized(size) => {
					self.context.resize(*size);
					self.gl.resized(size.width, size.height)
				}
				_ => (),
			}
		}

		events
	}

	pub fn clear(&self) {
		self.gl.clear();
	}

	pub fn swap(&self) {
		self.context.swap_buffers().unwrap()
	}

	pub fn clear_color<C: Into<Color>>(&mut self, color: C) {
		self.gl.clear_color(color)
	}

	pub fn texture_coloring(&mut self, flag: bool) {
		let value = if flag {
			TextureColoring::MixTexture
		} else {
			TextureColoring::Texture
		};

		self.texture_coloring = value;
		self.gl.set_texture_coloring_uniform(value);
	}

	fn add_event(
		state: &mut InputState,
		events: &mut Vec<SmittenEvent>,
		event: Event<()>,
		flow: &mut ControlFlow,
	) {
		*flow = ControlFlow::Wait;

		match event {
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::Resized(phys) => events.push(SmittenEvent::WindowResized(phys)),
				WindowEvent::KeyboardInput { input, .. } => match input.state {
					ElementState::Pressed => {
						state.set_keycode_state(input.scancode, input.virtual_keycode, true);

						events.push(SmittenEvent::Keydown {
							scancode: input.scancode,
							key: input.virtual_keycode.map(|v| v.try_into().ok()).flatten(),
						});
					}
					ElementState::Released => {
						state.set_keycode_state(input.scancode, input.virtual_keycode, false);

						events.push(SmittenEvent::Keyup {
							scancode: input.scancode,
							key: input.virtual_keycode.map(|v| v.try_into().ok()).flatten(),
						})
					}
				},
				WindowEvent::CloseRequested => panic!("TOOD: gen- Fix me later"),
				WindowEvent::MouseInput { state, button, .. } => match state {
					ElementState::Pressed => events.push(SmittenEvent::MouseDown { button }),
					ElementState::Released => events.push(SmittenEvent::MouseUp { button }),
				},
				WindowEvent::CursorMoved { position, .. } => {
					state.mouse_position = Vec2::new(position.x as f32, position.y as f32)
				}
				_ => (),
			},
			Event::DeviceEvent { event, .. } => match event {
				_ => (),
			},
			Event::MainEventsCleared => {
				*flow = ControlFlow::Exit;
			}
			_ => (),
		}
	}

	pub fn make_texture<P: AsRef<Path>>(&mut self, path: P) -> TextureId {
		let tex = Texture::from_file(&self.gl, path);
		let id = self.next_textureid;

		self.textures.insert(id, tex);
		self.next_textureid.0 += 1;

		id
	}

	pub fn make_font<P: AsRef<Path>>(&mut self, path: P) -> FontId {
		self.gl.bind_program();
		let font = SmittenFont::from_file(&self.gl, path);
		let id = self.next_fontid;

		self.fonts.insert(id, font);
		self.next_fontid.0 += 1;

		id
	}

	pub fn write<S: Into<String>, P: Into<Anchored>>(
		&self,
		font: FontId,
		text: S,
		pos: P,
		color: Color,
		scale: f32,
	) {
		let size = 64.0 * scale;
		let string = text.into();
		let font = self.fonts.get(&font).unwrap();

		// We're about to override this
		self.current_texture.set(None);

		self.gl.bind_program();
		self.gl
			.set_texture_coloring_uniform(TextureColoring::MixTexture);
		self.gl.set_color_uniform(color);

		unsafe { font.packed.texture.bind(&self.gl) };

		// New layout code
		let mut ascent = 0.0f32;
		let mut descent = 0.0f32;
		let mut off_x = 0.0f32;

		for ch in string.chars() {
			let metrics = font.font.metrics(ch, size);

			// ascent is the bit of the glyph above the baseline. ymin is
			// negative if there is descent.
			ascent = ascent.max(metrics.height as f32 + metrics.ymin as f32);
			descent = descent.min(metrics.ymin as f32);

			off_x += metrics.advance_width;
		}

		let mur_size = self.gl.transform.mur_size;
		let mur = |f: f32| -> f32 { f / mur_size as f32 };
		let unmur = |f: f32| -> f32 { f * mur_size as f32 };

		let width = off_x;
		let height = ascent + descent.abs();
		let text_dim = Vec2::new(width, height);
		let text_hdim = text_dim / 2;
		let baseline = descent.abs();

		let pos = pos
			.into()
			.resolve(text_dim.operation(mur), &self.gl.transform);
		let pos = pos.operation(unmur);

		let mut offset_x = 0.0;
		for ch in string.chars() {
			let metrics = font.font.metrics(ch, size);

			// Dimensioning
			let dim = Vec2::new(metrics.width as f32, metrics.height as f32);

			let gl_dim = self.gl.transform.pixel_vec_to_opengl(dim);

			// Positioning
			let x = (metrics.width as f32 / 2.0) + offset_x + metrics.xmin as f32;
			let y = (metrics.height as f32 / 2.0) + baseline + metrics.ymin as f32;

			offset_x += metrics.advance_width;

			let glyph_pos = Vec2::new(x - text_hdim.x, y - text_hdim.y);

			let gl_pos = self.gl.transform.pixel_vec_to_opengl(glyph_pos + pos);

			self.gl.gen_draw_rectangle_raw_coords(
				gl_pos,
				gl_dim,
				&font.packed.characters.get(&ch).unwrap().rect,
			)
		}

		// End new layour code
	}

	// Draw a rectangle at `pos` murs (center) which is `dim` murs in dimension.
	pub fn rect<P, D, R>(&self, pos: P, dim: D, draw: R)
	where
		P: Into<Vec2>,
		D: Into<Vec2>,
		R: Into<Draw>,
	{
		let draw = draw.into();
		match draw {
			Draw::Color(c) => {
				self.gl.set_texture_coloring_uniform(TextureColoring::Color);

				if self.current_color.get() != c {
					self.gl.set_color_uniform(c);
					self.current_color.set(c);
				}
			}
			Draw::Texture(tid) => match self.current_texture.get() {
				Some(cur) if cur == tid => (),
				Some(_) => self.bind_texture(tid),
				None => self.bind_texture(tid),
			},
		}

		self.gl.draw_rectangle(pos.into(), dim.into());

		if let Draw::Color(_) = draw {
			self.gl.set_texture_coloring_uniform(self.texture_coloring);
		}
	}

	pub fn anchored_rect<A, D, R>(&self, pos: A, dim: D, draw: R)
	where
		A: Into<Anchored>,
		D: Into<Vec2>,
		R: Into<Draw>,
	{
		let dim = dim.into();
		let pos = pos.into().resolve(dim, &self.gl.transform);
		self.rect(pos, dim, draw)
	}

	pub fn sdf(&self, sdf: SignedDistance) {
		self.gl.draw_sdf(sdf)
	}

	fn bind_texture(&self, tid: TextureId) {
		self.gl.bind_program();
		match self.textures.get(&tid) {
			Some(tex) => {
				unsafe { tex.bind(&self.gl) }
				self.current_texture.set(Some(tid));
			}
			None => todo!(),
		}
	}

	pub fn is_key_down(&self, key: Key) -> bool {
		self.input_state.down_keys.contains(&key)
	}

	pub fn is_scancode_down(&self, scancode: u32) -> bool {
		self.input_state.down_scancode.contains(&scancode)
	}

	pub fn mouse_position(&self) -> Vec2 {
		self.gl
			.transform
			.window_vec_to_murs(self.input_state.mouse_position)
	}

	/// Returns the mouse position as pixels in reference to the center of the window
	pub fn mouse_position_absolute(&self) -> Vec2 {
		let half_dim = self.gl.transform.screen_vec / 2;

		// Mouse is weid-coordinates, fix it
		let mut mouse = self.input_state.mouse_position;
		mouse.y = self.gl.transform.screen_vec.y - mouse.y;

		mouse - half_dim
	}
}

pub enum SmittenEvent {
	WindowResized(PixelSize),
	Keydown { scancode: u32, key: Option<Key> },
	Keyup { scancode: u32, key: Option<Key> },
	MouseDown { button: MouseButton },
	MouseUp { button: MouseButton },
}

#[derive(Copy, Clone, Debug)]
pub enum Draw {
	Color(Color),
	Texture(TextureId),
}

impl From<Color> for Draw {
	fn from(clr: Color) -> Draw {
		Draw::Color(clr)
	}
}

impl From<TextureId> for Draw {
	fn from(tid: TextureId) -> Draw {
		Draw::Texture(tid)
	}
}

#[derive(Copy, Clone, Debug)]
pub enum Anchored {
	Vertical {
		vert: VerticalAnchor,
		hori: f32,
	},
	Horizontal {
		vert: f32,
		hori: HorizontalAnchor,
	},
	Both {
		vert: VerticalAnchor,
		hori: HorizontalAnchor,
	},
	Neither(Vec2),
}

impl Anchored {
	pub fn resolve(&self, dim: Vec2, trans: &Transform) -> Vec2 {
		let top = trans.mur_half_dimensions.y - (dim.y / 2.0);
		let btm = -trans.mur_half_dimensions.y + (dim.y / 2.0);
		let lft = -trans.mur_half_dimensions.x + (dim.x / 2.0);
		let rht = trans.mur_half_dimensions.x - (dim.x / 2.0);

		let v = |v: &VerticalAnchor| match v {
			VerticalAnchor::Top(off) => top + *off,
			VerticalAnchor::Center(off) => *off,
			VerticalAnchor::Bottom(off) => btm + *off,
		};

		let h = |h: &HorizontalAnchor| match h {
			HorizontalAnchor::Left(off) => lft + *off,
			HorizontalAnchor::Center(off) => *off,
			HorizontalAnchor::Right(off) => rht + *off,
		};

		match self {
			Anchored::Vertical { vert, hori } => Vec2::new(*hori, v(vert)),
			Anchored::Horizontal { vert, hori } => Vec2::new(h(hori), *vert),
			Anchored::Both { vert, hori } => Vec2::new(h(hori), v(vert)),
			Anchored::Neither(v) => v.clone(),
		}
	}
}

impl From<(f32, VerticalAnchor)> for Anchored {
	fn from(t: (f32, VerticalAnchor)) -> Self {
		Self::Vertical {
			vert: t.1,
			hori: t.0,
		}
	}
}

impl From<(HorizontalAnchor, f32)> for Anchored {
	fn from(t: (HorizontalAnchor, f32)) -> Self {
		Self::Horizontal {
			vert: t.1,
			hori: t.0,
		}
	}
}

impl From<(HorizontalAnchor, VerticalAnchor)> for Anchored {
	fn from(t: (HorizontalAnchor, VerticalAnchor)) -> Self {
		Self::Both {
			vert: t.1,
			hori: t.0,
		}
	}
}

impl From<(f32, f32)> for Anchored {
	fn from(t: (f32, f32)) -> Self {
		Self::Neither(t.into())
	}
}

impl From<Vec2> for Anchored {
	fn from(t: Vec2) -> Self {
		Self::Neither(t)
	}
}

#[derive(Copy, Clone, Debug)]
pub enum VerticalAnchor {
	Top(f32),
	Bottom(f32),
	Center(f32),
}

#[derive(Copy, Clone, Debug)]
pub enum HorizontalAnchor {
	Left(f32),
	Center(f32),
	Right(f32),
}

#[rustfmt::skip]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Key {
	A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
	Row1, Row2, Row3, Row4, Row5, Row6, Row7, Row8, Row9, Row0,
	Escape, Space
}

impl From<Key> for VirtualKeyCode {
	fn from(k: Key) -> Self {
		match k {
			Key::A => VirtualKeyCode::A,
			Key::B => VirtualKeyCode::B,
			Key::C => VirtualKeyCode::C,
			Key::D => VirtualKeyCode::D,
			Key::E => VirtualKeyCode::E,
			Key::F => VirtualKeyCode::F,
			Key::G => VirtualKeyCode::G,
			Key::H => VirtualKeyCode::H,
			Key::I => VirtualKeyCode::I,
			Key::J => VirtualKeyCode::J,
			Key::K => VirtualKeyCode::K,
			Key::L => VirtualKeyCode::L,
			Key::M => VirtualKeyCode::M,
			Key::N => VirtualKeyCode::N,
			Key::O => VirtualKeyCode::O,
			Key::P => VirtualKeyCode::P,
			Key::Q => VirtualKeyCode::Q,
			Key::R => VirtualKeyCode::R,
			Key::S => VirtualKeyCode::S,
			Key::T => VirtualKeyCode::T,
			Key::U => VirtualKeyCode::U,
			Key::V => VirtualKeyCode::V,
			Key::W => VirtualKeyCode::W,
			Key::X => VirtualKeyCode::X,
			Key::Y => VirtualKeyCode::Y,
			Key::Z => VirtualKeyCode::Z,
			Key::Row1 => VirtualKeyCode::Key1,
			Key::Row2 => VirtualKeyCode::Key2,
			Key::Row3 => VirtualKeyCode::Key3,
			Key::Row4 => VirtualKeyCode::Key4,
			Key::Row5 => VirtualKeyCode::Key5,
			Key::Row6 => VirtualKeyCode::Key6,
			Key::Row7 => VirtualKeyCode::Key7,
			Key::Row8 => VirtualKeyCode::Key8,
			Key::Row9 => VirtualKeyCode::Key9,
			Key::Row0 => VirtualKeyCode::Key0,
			Key::Escape => VirtualKeyCode::Escape,
			Key::Space => VirtualKeyCode::Space,
		}
	}
}

impl TryFrom<VirtualKeyCode> for Key {
	type Error = ();

	fn try_from(v: VirtualKeyCode) -> Result<Self, Self::Error> {
		match v {
			VirtualKeyCode::A => Ok(Key::A),
			VirtualKeyCode::B => Ok(Key::B),
			VirtualKeyCode::C => Ok(Key::C),
			VirtualKeyCode::D => Ok(Key::D),
			VirtualKeyCode::E => Ok(Key::E),
			VirtualKeyCode::F => Ok(Key::F),
			VirtualKeyCode::G => Ok(Key::G),
			VirtualKeyCode::H => Ok(Key::H),
			VirtualKeyCode::I => Ok(Key::I),
			VirtualKeyCode::J => Ok(Key::J),
			VirtualKeyCode::K => Ok(Key::K),
			VirtualKeyCode::L => Ok(Key::L),
			VirtualKeyCode::M => Ok(Key::M),
			VirtualKeyCode::N => Ok(Key::N),
			VirtualKeyCode::O => Ok(Key::O),
			VirtualKeyCode::P => Ok(Key::P),
			VirtualKeyCode::Q => Ok(Key::Q),
			VirtualKeyCode::R => Ok(Key::R),
			VirtualKeyCode::S => Ok(Key::S),
			VirtualKeyCode::T => Ok(Key::T),
			VirtualKeyCode::U => Ok(Key::U),
			VirtualKeyCode::V => Ok(Key::V),
			VirtualKeyCode::W => Ok(Key::W),
			VirtualKeyCode::X => Ok(Key::X),
			VirtualKeyCode::Y => Ok(Key::Y),
			VirtualKeyCode::Z => Ok(Key::Z),
			VirtualKeyCode::Key1 => Ok(Key::Row1),
			VirtualKeyCode::Key2 => Ok(Key::Row2),
			VirtualKeyCode::Key3 => Ok(Key::Row3),
			VirtualKeyCode::Key4 => Ok(Key::Row4),
			VirtualKeyCode::Key5 => Ok(Key::Row5),
			VirtualKeyCode::Key6 => Ok(Key::Row6),
			VirtualKeyCode::Key7 => Ok(Key::Row7),
			VirtualKeyCode::Key8 => Ok(Key::Row8),
			VirtualKeyCode::Key9 => Ok(Key::Row9),
			VirtualKeyCode::Key0 => Ok(Key::Row0),
			VirtualKeyCode::Escape => Ok(Key::Escape),
			VirtualKeyCode::Space => Ok(Key::Space),
			_ => Err(()),
		}
	}
}
