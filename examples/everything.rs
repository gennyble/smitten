use smitten::{self, Color, Key, SignedDistance, Smitten, Vec2};

fn main() {
	let mut smitty = Smitten::new((720, 480), "Square", 24);
	let whoisshe = smitty.make_texture("examples/whoisshe.png");

	loop {
		let _events = smitty.events();

		// Quit on escape
		if smitty.is_key_down(Key::Escape) {
			break;
		}

		// Clear the screen
		smitty.clear();

		smitty.rect((-2, -2), (1, 1), Color::rgb(0.5, 0.1, 0.3));
		smitty.rect((2, 2), (5, 5), whoisshe);

		smitty.sdf(SignedDistance::line_segment(
			(-2, 2),
			(2, -2),
			2,
			Color::rgb(0.1, 0.3, 0.5),
		));

		smitty.sdf(SignedDistance::Circle {
			center: Vec2::new(-1.0, 0.0),
			radius: 5,
			color: Color::grey(0.8),
		});

		smitty.sdf(SignedDistance::Circle {
			center: Vec2::new(1.0, 0.0),
			radius: 5,
			color: Color::grey(0.8),
		});

		// Swap buffers
		smitty.swap();
	}
}
