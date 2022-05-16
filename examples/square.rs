use smitten::{self, Smitten};

fn main() {
	let mut smitty = Smitten::new((640, 480), "Square");
	loop {
		smitty.events()
	}
}
