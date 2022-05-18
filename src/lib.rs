mod color;
mod gl;
mod vec2;

use std::{
	cell::Cell,
	collections::{HashMap, HashSet},
	path::Path,
	sync::RwLock,
};

use gl::{OpenGl, Texture, Transform};
use glow::HasContext;
use glutin::{
	dpi::PhysicalSize,
	event::{ElementState, Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	platform::{run_return::EventLoopExtRunReturn, unix::WindowBuilderExtUnix},
	window::{Window, WindowBuilder},
	ContextBuilder, ContextWrapper, PossiblyCurrent,
};

pub use color::Color;
pub use vec2::Vec2;

pub use glutin::event::VirtualKeyCode;

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

	next_textureid: TextureId,
	textures: HashMap<TextureId, Texture>,

	down_keys: HashSet<VirtualKeyCode>,
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
		let wb = WindowBuilder::new()
			.with_title(title)
			.with_app_id("pleasefloat".into())
			.with_inner_size(size);

		let wc = ContextBuilder::new()
			.with_vsync(true)
			.build_windowed(wb, &el)
			.unwrap();

		//TODO: Add saftey note
		let context = unsafe { wc.make_current().unwrap() };
		let mut gl = OpenGl::new(&context, Transform::new(size, mur));

		gl.clear_color(Color::rgb(0.0, 0.0, 0.0));

		Smitten {
			context,
			event_loop: el,
			gl,
			current_color: Cell::new(Color::rgb(0.0, 0.0, 0.0)),
			current_texture: Cell::new(None),
			next_textureid: TextureId(0),
			textures: HashMap::new(),
			down_keys: HashSet::new(),
			down_scancode: HashSet::new(),
		}
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

					if let Some(key) = key {
						self.down_keys.insert(*key);
					}
				}
				SmittenEvent::Keyup { scancode, key } => {
					self.down_scancode.remove(scancode);

					if let Some(key) = key {
						self.down_keys.remove(key);
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
		match draw.into() {
			Draw::Color(c) => {
				if self.current_texture.get().is_some() {
					unsafe { self.gl.gl().bind_texture(glow::TEXTURE_2D, None) };
					self.current_texture.set(None);
				}

				if self.current_color.get() != c {
					self.gl.set_color_uniform(c);
					self.current_color.set(c);
					println!("Colour set!");
				}
			}
			Draw::Texture(tid) => match self.current_texture.get() {
				Some(cur) if cur == tid => (),
				Some(_) => self.bind_texture(tid),
				None => self.bind_texture(tid),
			},
		}

		self.gl.draw_rectangle(pos.into(), dim.into())
	}

	fn bind_texture(&self, tid: TextureId) {
		match self.textures.get(&tid) {
			Some(tex) => {
				unsafe { tex.bind(&self.gl) }
				self.current_texture.set(Some(tid));
			}
			None => todo!(),
		}
	}

	pub fn is_key_down(&self, key: VirtualKeyCode) -> bool {
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

impl Into<Draw> for Color {
	fn into(self) -> Draw {
		Draw::Color(self)
	}
}

impl Into<Draw> for TextureId {
	fn into(self) -> Draw {
		Draw::Texture(self)
	}
}
