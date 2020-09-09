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
}
