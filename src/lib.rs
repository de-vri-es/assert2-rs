#![allow(clippy::needless_lifetimes)]

//! All-purpose [`assert!(...)`](macro.assert.html) and [`check!(...)`](macro.check.html) macros, inspired by [Catch2](https://github.com/catchorg/Catch2).
//! There is also a [`debug_assert!(...)`](macro.debug_assert.html) macro that is disabled on optimized builds by default.
//!
//! # Why these macros?
//!
//! These macros offer some benefits over the assertions from the standard library:
//!   * The macros parse your expression to detect comparisons and adjust the error message accordingly.
//!     No more `assert_eq!(a, b)` or `assert_ne!(c, d)`, just write `assert!(1 + 1 == 2)`, or even `assert!(1 + 1 > 1)`!
//!     They also split on the `&&` operator to show you which predicate failed.
//!   * You can test for pattern matches: `assert!(let Err(e) = File::open("/non/existing/file"))`.
//!   * The macros support [`let` chains](https://blog.rust-lang.org/2025/06/26/Rust-1.88.0/#let-chains) (even with compilers older than Rust 1.88).
//!   * The `assert!(...)` macro makes `let` bindings available in the calling scope, so you can use the matched value after the assertion.
//!   * The `check` macro can be used to perform multiple checks before panicking.
//!   * The macros provide more information than the standard `std::assert!()` when the assertion fails.
//!   * Colored failure messages with diffs!
//!
//! The macros also accept additional arguments for a custom message, so it is fully compatible with `std::assert`.
//! This means that you can import the macro as a drop in replacement:
//! ```
//! use assert2::assert;
//! ```
//!
//! # Examples
//!
//! ```should_panic
//! # use assert2::check;
//! check!(6 + 1 <= 2 * 3);
//! ```
//!
//! ![Output](https://raw.githubusercontent.com/de-vri-es/assert2-rs/ba98984a32d6381e6710e34eb1fb83e65e851236/binary-operator.png)
//!
//! ----------
//!
//! ```should_panic
//! # use assert2::check;
//! # use assert2::let_assert;
//! # use std::fs::File;
//! # use std::io::ErrorKind;
//! # #[derive(Debug, Eq, PartialEq)]
//! # struct Pet {
//! #   name: String,
//! #   age: u32,
//! #   kind: String,
//! #   shaved: bool,
//! # }
//! # let scrappy = Pet {
//! #   name: "Scrappy".into(),
//! #   age: 7,
//! #   kind: "Bearded Collie".into(),
//! #   shaved: false,
//! # };
//! # let coco = Pet {
//! #   name: "Coco".into(),
//! #   age: 7,
//! #   kind: "Bearded Collie".into(),
//! #   shaved: true,
//! # };
//! check!(scrappy == coco);
//! ```
//!
//! ![Output](https://raw.githubusercontent.com/de-vri-es/assert2-rs/54ee3141e9b23a0d9038697d34f29f25ef7fe810/multiline-diff.png)
//!
//! ----------
//!
//! ```should_panic
//! # use assert2::check;
//! check!((3, Some(4)) == [1, 2, 3].iter().size_hint());
//! ```
//!
//! ![Output](https://raw.githubusercontent.com/de-vri-es/assert2-rs/54ee3141e9b23a0d9038697d34f29f25ef7fe810/single-line-diff.png)
//!
//! ----------
//!
//! ```should_panic
//! # use assert2::check;
//! # use std::fs::File;
//! check!(let Ok(_) = File::open("/non/existing/file"));
//! ```
//!
//! ![Output](https://raw.githubusercontent.com/de-vri-es/assert2-rs/54ee3141e9b23a0d9038697d34f29f25ef7fe810/pattern-match.png)
//!
//! ----------
//!
//! ```should_panic
//! # use assert2::check;
//! # use assert2::assert;
//! # use std::fs::File;
//! # use std::io::ErrorKind;
//! assert!(let Err(e) = File::open("/non/existing/file"));
//! check!(e.kind() == ErrorKind::PermissionDenied);
//! ```
//!
//! ![Output](https://github.com/de-vri-es/assert2-rs/blob/54ee3141e9b23a0d9038697d34f29f25ef7fe810/let-assert.png?raw=true)
//!
//! ----------
//!
//! ```should_panic
//! # use assert2::check;
//! # use assert2::let_assert;
//! # use std::fs::File;
//! # use std::io::ErrorKind;
//! check!(let Err(e) = File::open("/non/existing/file") && e.kind() == ErrorKind::PermissionDenied);
//! ```
//!
//! ![Output](https://github.com/de-vri-es/assert2-rs/blob/54ee3141e9b23a0d9038697d34f29f25ef7fe810/let-assert.png?raw=true)
//!
//! # `assert` vs `check`
//! The crate provides two macros: `check!(...)` and `assert!(...)`.
//! The main difference is that `check` is really intended for test cases and doesn't immediately panic.
//! Instead, it will print the assertion error and fail the test.
//! This allows you to run multiple checks and can help to determine the reason of a test failure more easily.
//! The `assert` macro on the other hand simply prints the error and panics,
//! and can be used outside of tests just as well.
//!
//! Currently, `check` uses a scope guard to delay the panic until the current scope ends.
//! Ideally, `check` doesn't panic at all, but only signals that a test case has failed.
//! If this becomes possible in the future, the `check` macro will change, so **you should not rely on `check` to panic**.
//!
//! # Difference between stable and nightly.
//! If available, the crate uses the `proc_macro_span` feature to get the original source code.
//! On stable and beta, it falls back to stringifying the expression.
//! This makes the output a bit more readable on nightly.
//!
//! # The `let_assert!()` macro
//! You can also use the [`let_assert!(...)`](macro.let_assert.html).
//! It is very similar to `assert!(let ...)`,
//! but all placeholders will be made available as variables in the calling scope.
//!
//! This allows you to run additional checks on the captured variables.
//!
//! For example:
//!
//! ```
//! # fn main() {
//! # use assert2::let_assert;
//! # use assert2::check;
//! # struct Foo {
//! #  name: &'static str,
//! # }
//! # enum Error {
//! #   InvalidName(InvalidNameError),
//! # }
//! # struct InvalidNameError {
//! #   name: &'static str,
//! # }
//! # impl Foo {
//! #   fn try_new(name: &'static str) -> Result<Self, Error> {
//! #     if name == "bar" {
//! #       Ok(Self { name })
//! #     } else {
//! #       Err(Error::InvalidName(InvalidNameError { name }))
//! #     }
//! #   }
//! #   fn name(&self) -> &'static str {
//! #     self.name
//! #   }
//! # }
//! # impl InvalidNameError {
//! #   fn name(&self) -> &'static str {
//! #     self.name
//! #   }
//! #   fn to_string(&self) -> String {
//! #     format!("invalid name: {}", self.name)
//! #   }
//! # }
//! let_assert!(Ok(foo) = Foo::try_new("bar"));
//! check!(foo.name() == "bar");
//!
//! let_assert!(Err(Error::InvalidName(e)) = Foo::try_new("bogus name"));
//! check!(e.name() == "bogus name");
//! check!(e.to_string() == "invalid name: bogus name");
//! # }
//! ```
//!
//! # Controlling the output format.
//!
//! As an end-user, you can influence the way that `assert2` formats failed assertions by changing the `ASSERT2` environment variable.
//! You can specify any combination of options, separated by a comma.
//! The supported options are:
//! * `auto`: Automatically select the compact or pretty `Debug` format for an assertion based on the length (default).
//! * `pretty`: Always use the pretty `Debug` format for assertion messages (`{:#?}`).
//! * `compact`: Always use the compact `Debug` format for assertion messages (`{:?}`).
//! * `no-color`: Disable colored output, even when the output is going to a terminal.
//! * `color`: Enable colored output, even when the output is not going to a terminal.
//!
//! For example, you can run the following command to force the use of the compact `Debug` format with colored output:
//! ```shell
//! ASSERT2=compact,color cargo test
//! ```
//!
//! If neither the `color` or the `no-color` options are set,
//! then `assert2` follows the [clicolors specification](https://bixense.com/clicolors/):
//!
//!  * `NO_COLOR != 0` or `CLICOLOR == 0`: Write plain output without color codes.
//!  * `CLICOLOR != 0`: Write colored output when the output is going to a terminal.
//!  * `CLICOLOR_FORCE != 0`:  Write colored output even when it is not going to a terminal.

#[doc(hidden)]
pub mod __assert2_impl;

/// Assert that an expression evaluates to true or matches a pattern.
///
/// Use a `let` expression to test an expression against a pattern: `assert!(let pattern = expr)`.
/// For other tests, just give a boolean expression to the macro: `assert!(1 + 2 == 2)`.
///
/// If the expression evaluates to false or if the pattern doesn't match,
/// an assertion failure is printed and the macro panics instantly.
///
/// Use [`check!`](macro.check.html) if you still want further checks to be executed.
///
/// # Custom messages
/// You can pass additional arguments to the macro.
/// These will be used to print a custom message in addition to the normal message.
///
/// ```
/// # use assert2::assert;
/// assert!(3 * 4 == 12, "Oh no, math is broken! 1 + 1 == {}", 1 + 1);
/// ```
#[macro_export]
macro_rules! assert {
	($($tokens:tt)*) => {
		$crate::__assert2_impl::assert_impl!($crate, "assert", $($tokens)*)
	}
}

/// Check if an expression evaluates to true or matches a pattern.
///
/// Use a `let` expression to test an expression against a pattern: `check!(let pattern = expr)`.
/// For other tests, just give a boolean expression to the macro: `check!(1 + 2 == 2)`.
///
/// If the expression evaluates to false or if the pattern doesn't match,
/// an assertion failure is printed but the macro does not panic immediately.
/// The check macro will cause the running test to fail eventually.
///
/// Use [`assert!`](macro.assert.html) if you want the test to panic instantly.
///
/// Currently, this macro uses a scope guard to delay the panic.
/// However, this may change in the future if there is a way to signal a test failure without panicking.
/// **Do not rely on `check!()` to panic**.
///
/// # Custom messages
/// You can pass additional arguments to the macro.
/// These will be used to print a custom message in addition to the normal message.
///
/// ```
/// # use assert2::check;
/// check!(3 * 4 == 12, "Oh no, math is broken! 1 + 1 == {}", 1 + 1);
/// ```
#[macro_export]
macro_rules! check {
	($($tokens:tt)*) => {
		let _guard = match $crate::__assert2_impl::check_impl!($crate, "check", $($tokens)*) {
			::core::result::Result::Ok(_) => ::core::option::Option::None,
			::core::result::Result::Err(_) => {
				::core::option::Option::Some($crate::__assert2_impl::FailGuard(|| ::core::panic!("check failed")))
			},
		};
	}
}

/// Assert that an expression evaluates to true or matches a pattern.
///
/// This macro supports the same checks as [`assert`](macro.assert.html), but they are only executed if debug assertions are enabled.
///
/// As with [`std::debug_assert`](https://doc.rust-lang.org/stable/std/macro.debug_assert.html),
/// the expression is still type checked if debug assertions are disabled.
///
#[macro_export]
macro_rules! debug_assert {
	($($tokens:tt)*) => {
		if ::core::cfg!(debug_assertions) {
			$crate::__assert2_impl::assert_impl!($crate, "debug_assert", $($tokens)*);
		}
	}
}

/// Assert that an expression matches a pattern.
///
/// This is very similar to `assert!(let pattern = expression)`,
/// except that this macro makes all placeholders available in the calling scope.
/// This can be used to assert a pattern match,
/// and then run more checks on the captured variables.
///
/// For example:
/// ```
/// # use assert2::let_assert;
/// # use assert2::check;
/// # fn main() {
/// # struct Foo {
/// #  name: &'static str,
/// # }
/// # enum Error {
/// #   InvalidName(InvalidNameError),
/// # }
/// # struct InvalidNameError {
/// #   name: &'static str,
/// # }
/// # impl Foo {
/// #   fn try_new(name: &'static str) -> Result<Self, Error> {
/// #     if name == "bar" {
/// #       Ok(Self { name })
/// #     } else {
/// #       Err(Error::InvalidName(InvalidNameError { name }))
/// #     }
/// #   }
/// #   fn name(&self) -> &'static str {
/// #     self.name
/// #   }
/// # }
/// # impl InvalidNameError {
/// #   fn name(&self) -> &'static str {
/// #     self.name
/// #   }
/// #   fn to_string(&self) -> String {
/// #     format!("invalid name: {}", self.name)
/// #   }
/// # }
/// let_assert!(Ok(foo) = Foo::try_new("bar"));
/// check!(foo.name() == "bar");
///
/// let_assert!(Err(Error::InvalidName(e)) = Foo::try_new("bogus name"));
/// check!(e.name() == "bogus name");
/// check!(e.to_string() == "invalid name: bogus name");
/// # }
/// ```
#[macro_export]
#[deprecated(since = "0.4.0", note = "use `assert2::assert!(let ...)` instead")]
macro_rules! let_assert {
	($($tokens:tt)*) => {
		$crate::__assert2_impl::assert_impl!($crate, "let_assert", let $($tokens)*);
	}
}

#[doc(hidden)]
#[macro_export]
macro_rules! __assert2_stringify {
	($e:expr) => {
		// Stringifying as an expression gives nicer output
		// than stringifying a raw list of token trees.
		$crate::__assert2_core_stringify!($e)
	};
	($($t:tt)*) => {
		$crate::__assert2_core_stringify!($($t)*)
	};
}

#[doc(hidden)]
pub use core::stringify as __assert2_core_stringify;
