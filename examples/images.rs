use assert2::check;
use std::fs::File;

fn main() {
	check!(6 + 1 <= 2 * 3);
	check!(true && false);
	check!(let Ok(_) = File::open("/non/existing/file"));
}
