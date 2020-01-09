#![feature(proc_macro_hygiene)]

use check::check;

fn main() {
	let mut vec = Vec::new();
	vec.push(1);

	{
		check!(&vec == &vec![]);
		eprintln!("This still executes!");
	}

	eprintln!("But this does not.");
}
