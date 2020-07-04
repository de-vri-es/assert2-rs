use assert2::assert;
use assert2::check;
use assert2::debug_assert;

#[test]
fn check_pass() {
	check!(1 == 1);
	check!(1 == 1, "{}", "math broke");
	check!(1 == 1, "{}", "math broke",);
	check!(true && true);
	check!(true && true, "{}", "logic broke");
	check!(true && true, "{}", "logic broke",);
	check!(let Ok(10) = Result::<i32, i32>::Ok(10));
	check!(let Ok(10) = Result::<i32, i32>::Ok(10), "{}", "rust broke");
	check!(let Ok(10) = Result::<i32, i32>::Ok(10), "{}", "rust broke",);
}

#[test]
fn assert_pass() {
	assert!(1 == 1);
	assert!(1 == 1, "{}", "math broke");
	assert!(1 == 1, "{}", "math broke",);
	assert!(true && true);
	assert!(true && true, "{}", "logic broke");
	assert!(true && true, "{}", "logic broke",);
	assert!(let Ok(10) = Result::<i32, i32>::Ok(10));
	assert!(let Ok(10) = Result::<i32, i32>::Ok(10), "{}", "rust broke");
	assert!(let Ok(10) = Result::<i32, i32>::Ok(10), "{}", "rust broke",);
}

#[test]
fn debug_assert_pass() {
	debug_assert!(1 == 1);
	debug_assert!(1 == 1, "{}", "math broke");
	debug_assert!(1 == 1, "{}", "math broke",);
	debug_assert!(true && true);
	debug_assert!(true && true, "{}", "logic broke");
	debug_assert!(true && true, "{}", "logic broke",);
	debug_assert!(let Ok(10) = Result::<i32, i32>::Ok(10));
	debug_assert!(let Ok(10) = Result::<i32, i32>::Ok(10), "{}", "rust broke");
	debug_assert!(let Ok(10) = Result::<i32, i32>::Ok(10), "{}", "rust broke",);
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
struct I(i32);

#[test]
fn check_non_debug() {
	check!(I(1) == I(1));
	check!(I(1) == I(1), "{}", "math broke");
	check!(I(1) == I(1), "{}", "math broke",);
	check!(!!(true && true));
	check!(!!(true && true), "{}", "logic broke");
	check!(!!(true && true), "{}", "logic broke",);
	check!(let I(10) = I(10));
	check!(let I(10) = I(10), "{}", "rust broke");
	check!(let I(10) = I(10), "{}", "rust broke",);
}

#[test]
fn assert_non_debug() {
	assert!(I(1) == I(1));
	assert!(I(1) == I(1), "{}", "math broke");
	assert!(I(1) == I(1), "{}", "math broke",);
	assert!(!!(true && true));
	assert!(!!(true && true), "{}", "logic broke");
	assert!(!!(true && true), "{}", "logic broke",);
	assert!(let I(10) = I(10));
	assert!(let I(10) = I(10), "{}", "rust broke");
	assert!(let I(10) = I(10), "{}", "rust broke",);
}

#[test]
fn debug_refs() {
	// Also check that references work.
	// These tests are important because we use auto-deref specialization
	// to support non-debug types.
	check!(&1 == &1);
	check!(&&1 == &&1);
	check!(&&&&&&&1 == &&&&&&&1);
	check!(let 10 = &10);
	check!(let 10 = & &10);
	assert!(&1 == &1);
	assert!(&&1 == &&1);
	assert!(&&&&&&&1 == &&&&&&&1);
	assert!(let 10 = &10);
	assert!(let 10 = & &10);
}

#[test]
fn non_debug_refs() {
	// Also check that references work.
	// These tests are important because we use auto-deref specialization
	// to support non-debug types.
	check!(&I(1) == &I(1));
	check!(&&I(1) == &&I(1));
	check!(&&&&&&&I(1) == &&&&&&&I(1));
	check!(let I(10) = &I(10));
	check!(let I(10) = & &I(10));
	assert!(&I(1) == &I(1));
	assert!(&&I(1) == &&I(1));
	assert!(&&&&&&&I(1) == &&&&&&&I(1));
	assert!(let I(10) = &I(10));
	assert!(let I(10) = & &I(10));
}

#[test]
fn test_hygiene_bug() {
	// Test to see if we work around a hygiene bug in the Rust compiler.
	// See https://github.com/rust-lang/rust/issues/74036
	// and https://github.com/rust-lang/rust/issues/67062
	macro_rules! assert_square {
		($val:expr, $expected:expr) => {
			assert!($val * $val == $expected)
		};
	};

	fn foo(a: i32, b: i32) -> i32 {
		a + b
	}

	macro_rules! assert_foo {
		($($args:expr),*; $expected:expr) => {
			assert!(foo($($args),*) == $expected)
		};
	}

	// If the hygiene bug is not fixed, this evaluates as 2 + 2 * 2 + 2 instead of (2 + 2) * (2 + 2).
	// So it will fail with assert!(8 == 16).
	assert_square!(2 + 2, 16);

	// Ensure function arguments don't get wrongly turned into tuples by the workaround.
	assert_foo!(2, 3; 5);
}

#[test]
fn no_copy() {
	let a = String::new();
	let b = String::new();
	assert2::debug_assert!(a == b);
	assert2::debug_assert!(a == b);
	drop(a);
	drop(b);
}

macro_rules! test_panic {
	($name:ident, $($expr:tt)*) => {
		#[test]
		#[should_panic]
		fn $name() {
			$($expr)*;
		}
	}
}

test_panic!(panic_check1, check!(1 == 2));
test_panic!(panic_check2, check!(1 == 2, "{}", "math broke"));
test_panic!(panic_check3, check!(true && false));
test_panic!(panic_check4, check!(true && false, "{}", "logic broke"));
test_panic!(panic_check5, check!(let Ok(_) = Result::<i32, i32>::Err(10)));
test_panic!(panic_check6, check!(let Ok(_) = Result::<i32, i32>::Err(10), "{}", "rust broke"));

test_panic!(panic_assert1, assert!(1 == 2));
test_panic!(panic_assert2, assert!(1 == 2, "{}", "math broke"));
test_panic!(panic_assert3, assert!(true && false));
test_panic!(panic_assert4, assert!(true && false, "{}", "logic broke"));
test_panic!(panic_assert5, assert!(let Ok(_) = Result::<i32, i32>::Err(10)));
test_panic!(panic_assert6, assert!(let Ok(_) = Result::<i32, i32>::Err(10), "{}", "rust broke"));
