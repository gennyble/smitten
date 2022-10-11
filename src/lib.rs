mod color;
mod gl;
mod vec2;

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
pub use vec2::Vec2;

pub type PixelSize = PhysicalSize<u32>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TextureId(u32);

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

	down_keys: HashSet<Key>,
	down_scancode: HashSet<u32>,
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
			down_keys: HashSet::new(),
			down_scancode: HashSet::new(),
		}
	}

	/// Get the screen size in pixels
	pub fn screen_pixels(&self) -> Vec2 {
		self.gl.transform.screen_vec
	}

	/// Get the screen size in murs
	pub fn screen_murs(&self) -> Vec2 {
		self.gl.transform.mur_dimensions
	}

	pub fn events(&mut self) -> Vec<SmittenEvent> {
		let mut events = vec![];
		self.event_loop
			.run_return(|event, _, flow| Self::add_event(&mut events, event, flow));

		for event in &events {
			match event {
				SmittenEvent::WindowResized(size) => {
					self.context.resize(*size);
					self.gl.resized(size.width, size.height)
				}
				SmittenEvent::Keydown { scancode, key } => {
					self.down_scancode.insert(*scancode);

					if let Some(Ok(key)) = key.map(|v| v.try_into()) {
						self.down_keys.insert(key);
					}
				}
				SmittenEvent::Keyup { scancode, key } => {
					self.down_scancode.remove(scancode);

					if let Some(Ok(key)) = key.map(|v| v.try_into()) {
						self.down_keys.remove(&key);
					}
				}
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

	fn add_event(events: &mut Vec<SmittenEvent>, event: Event<()>, flow: &mut ControlFlow) {
		*flow = ControlFlow::Wait;

		match event {
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::Resized(phys) => events.push(SmittenEvent::WindowResized(phys)),
				WindowEvent::KeyboardInput { input, .. } => match input.state {
					ElementState::Pressed => events.push(SmittenEvent::Keydown {
						scancode: input.scancode,
						key: input.virtual_keycode,
					}),
					ElementState::Released => events.push(SmittenEvent::Keyup {
						scancode: input.scancode,
						key: input.virtual_keycode,
					}),
				},
				WindowEvent::CloseRequested => panic!("TOOD: gen- Fix me later"),
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
		self.down_keys.contains(&key)
	}

	pub fn is_scancode_down(&self, scancode: u32) -> bool {
		self.down_scancode.contains(&scancode)
	}
}

pub enum SmittenEvent {
	WindowResized(PixelSize),
	Keydown {
		scancode: u32,
		key: Option<VirtualKeyCode>,
	},
	Keyup {
		scancode: u32,
		key: Option<VirtualKeyCode>,
	},
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

#[rustfmt::skip]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Key {
	A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
	Row1, Row2, Row3, Row4, Row5, Row6, Row7, Row8, Row9, Row0,
	Escape,
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
			_ => Err(()),
		}
	}
}
