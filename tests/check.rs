#![feature(proc_macro_hygiene)]

use assert2::check;

#[derive(Eq, PartialEq)]
struct Foo(u32);

#[test]
fn pass() {
	assert2::check!(1 == 1);
	assert2::check!(true && true);
	assert2::check!(!(false && true));
}

#[test]
#[should_panic]
fn check_panic1() {
	check!(1 == 2);
}

#[test]
#[should_panic]
fn check_panic2() {
	check!(true && false);
}
