use smitten::{self, Color, SignedDistance, Smitten, Vec2, VirtualKeyCode};

fn main() {
	let mut smitty = Smitten::new((720, 480), "Square", 24);

	loop {
		let _events = smitty.events();

		// Quit on escape
		if smitty.is_key_down(VirtualKeyCode::Escape) {
			break;
		}

		// Clear the screen
		smitty.clear();

		smitty.sdf(SignedDistance::Circle {
			center: Vec2::new(-2.0, -2.0),
			radius: 48,
			color: Color::rgb(0.1, 0.3, 0.7),
		});

		smitty.sdf(SignedDistance::LineSegment {
			color: Color::rgb(0.1, 0.3, 0.7),
			start: Vec2::new(5.0, 5.0),
			thickness: 5,
			end: Vec2::new(0.0, 0.0),
		});

		// Swap buffers
		smitty.swap();
	}
}
