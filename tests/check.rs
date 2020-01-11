use assert2::check;
use assert2::assert;

#[derive(Eq, PartialEq)]
struct Foo(u32);

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
