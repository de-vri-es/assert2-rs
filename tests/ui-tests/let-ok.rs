use assert2::assert;

fn main() {
	reproducible_panic::install();
	assert!(let Ok(foo) = Result::<i32, &str>::Err("Oh no!"));
	println!("foo + 1 = {}", foo + 1);
}
