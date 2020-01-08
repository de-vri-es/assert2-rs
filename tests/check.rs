#![feature(proc_macro_hygiene)]

use check::check;

#[derive(Eq, PartialEq)]
struct Foo(u32);

#[test]
fn check() {
	check!(1 == 1);
	check!(true && true);
	check!(!(false && true));
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
