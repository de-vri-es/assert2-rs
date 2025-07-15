#![allow(clippy::disallowed_names)]
#![no_implicit_prelude]

use ::assert2::{assert, check};

enum Foo {
	A(u32),
}

#[test]
fn assert_macro_works_without_prelude() {
	assert!(2 + 3 == 5);
	assert!(2 + 3 == 5, "custom message: {}", 5);
	assert!(let Foo::A(x) = Foo::A(2 + 3) && x == 5);
	assert!(let Foo::A(x) = Foo::A(2 + 3) && x == 5, "custom message: {}", "hello");
	assert!(true);
	assert!(true, "custom message: {}", "hello");
}

#[test]
fn check_macro_works_without_prelude() {
	check!(2 + 3 == 5);
	check!(2 + 3 == 5, "custom message: {}", 5);
	check!(let Foo::A(x) = Foo::A(2 + 3) && x == 5);
	check!(let Foo::A(x) = Foo::A(2 + 3) && x == 5, "custom message: {}", "hello");
	check!(true);
	check!(true, "custom message: {}", "hello");
}
