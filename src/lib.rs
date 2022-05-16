use std::sync::RwLock;

use glutin::{
	dpi::PhysicalSize,
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	platform::{run_return::EventLoopExtRunReturn, unix::WindowBuilderExtUnix},
	window::{Window, WindowBuilder},
	ContextBuilder, ContextWrapper, PossiblyCurrent,
};

type PixelSize = PhysicalSize<u32>;

pub struct Smitten {
	context: ContextWrapper<PossiblyCurrent, Window>,
	event_loop: EventLoop<()>,
}

impl Smitten {
	/// Make a new window and set everyththing up
	pub fn new<P, T>(size: P, title: T) -> Smitten
	where
		P: Into<PixelSize>,
		T: Into<String>,
	{
		let el = EventLoop::new();

		// The wayland app id "pleasefloat" will make the window floating on
		// sway if you have the following in your config:
		// for_window [app_id="pleasefloat"] floating enable
		let wb = WindowBuilder::new()
			.with_title(title)
			.with_app_id("pleasefloat".into())
			.with_inner_size(size.into());

		let wc = ContextBuilder::new()
			.with_vsync(true)
			.build_windowed(wb, &el)
			.unwrap();

		//TODO: Add saftey note
		let context = unsafe { wc.make_current().unwrap() };

		Smitten {
			context,
			event_loop: el,
		}
	}

	pub fn events(&mut self) {
		let mut events = vec![];
		self.event_loop
			.run_return(|event, _, flow| Self::add_event(&mut events, event, flow));
	}

	fn add_event(events: &mut Vec<SmittenEvent>, event: Event<()>, flow: &mut ControlFlow) {
		*flow = ControlFlow::Wait;

		match event {
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::Resized(phys) => events.push(SmittenEvent::WindowResized(phys)),
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
}

enum SmittenEvent {
	WindowResized(PixelSize),
}
