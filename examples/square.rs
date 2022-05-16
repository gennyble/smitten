use smitten::{self, Color, Smitten, VirtualKeyCode};

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

		// Draw a square at 0,0 (position is from the center) and make it 10x10 murs large.
		smitty.rect((0, 0), (10, 10), Color::rgb(0.1, 0.3, 0.5));

		// Swap buffers
		smitty.swap();
	}
}
