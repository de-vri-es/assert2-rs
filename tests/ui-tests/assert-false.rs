use assert2::assert;

pub fn main() {
	reproducible_panic::install();
	assert!(false)
}
