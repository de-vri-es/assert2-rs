#![allow(deprecated)]

use assert2::{assert, let_assert};

#[test]
fn let_assert_still_works() {
	let_assert!(Some(x) = Some(10));
	assert!(x == 10);
}
