use assert2::{check, check_scope, capture, info, let_assert};
use std::fs::File;
use std::io::ErrorKind;

fn main() {
	check_scope! {
		check!(6 + 1 <= 2 * 3);
		check!(true && false);

		{
			info!("opening file");
			check!(let Ok(_) = File::open("/non/existing/file"));
		}

		let_assert!(Err(e) = File::open("/non/existing/file"));
		check!(e.kind() == ErrorKind::PermissionDenied);

		let data = [2, 4, 6, 8, 9];
		for i in 0..data.len() {
			capture!(i);
			capture!(data[i]);
			check!(data[i] % 2 == 0);
		}
	}
}
