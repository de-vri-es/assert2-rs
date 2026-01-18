fn main() {
	reproducible_panic::install();
	assert2::assert!(
		let Some(x) = Some(3 + 2)
		&& x == 7
		&& let None = Some(5).filter(|&x| x == 6)
		&& true
	);
	println!("x = {x}")
}
