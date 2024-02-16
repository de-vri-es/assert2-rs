#![allow(clippy::nonminimal_bool)]

use assert2::check;
use assert2::let_assert;
use std::fs::File;
use std::io::ErrorKind;

fn main() {
	check!(6 + 1 <= 2 * 3);
	check!(true && false);
	check!(let Ok(_) = File::open("/non/existing/file"));

	let_assert!(Err(e) = File::open("/non/existing/file"));
	check!(e.kind() == ErrorKind::PermissionDenied);

	#[derive(Debug, Eq, PartialEq)]
	struct Pet {
		name: String,
		age: u32,
		kind: String,
		shaved: bool,
	}

	let scrappy = Pet {
		name: "Scrappy".into(),
		age: 7,
		kind: "Bearded Collies".into(),
		shaved: false,
	};

	let coco = Pet {
		name: "Coco".into(),
		age: 7,
		kind: "Bearded Collies".into(),
		shaved: true,
	};
	check!(scrappy == coco);

	check!(Some(1) == Some(11));
}
