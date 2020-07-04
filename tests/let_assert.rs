mod tests {
	use assert2::assert;
	use assert2::let_assert;

	#[test]
	fn basic_match() {
		// Test a basic match.
		let_assert!(Some(x) = Some(10));
		assert!(x == 10);
	}

	#[test]
	fn basic_match_ref() {
		// Test a basic match on a reference.
		let_assert!(Some(x) = &Some(10));
		assert!(x == &10);
	}

	#[test]
	fn anonymous_placeholders() {
		// Make sure _ placeholders are ignored.
		let_assert!((_, _, _) = (10, 11, 12));
		let_assert!((x, _, y) = (13, 14, 15));
		assert!(x == 13);
		assert!(y == 15);
	}

	#[test]
	fn underscore_prefixed_placeholders() {
		// But _name placeholders are not ignored.
		let_assert!((_x, _, _y) = (13, 14, 15));
		assert!(_x == 13);
		assert!(_y == 15);
	}

	#[test]
	fn consume() {
		let_assert!(Some(x) = Some(String::from("foo")));
		assert!(x == "foo");
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

	test_panic!(panic_let_assert1, let_assert!(Ok(_x) = Result::<i32, i32>::Err(10)));
	test_panic!(
		panic_let_assert2,
		let_assert!(Ok(_x) = Result::<i32, i32>::Err(10), "{}", "rust broke")
	);
}
