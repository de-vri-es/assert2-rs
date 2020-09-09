#![cfg_attr(feature = "doc-cfg", feature(doc_cfg))]
#![allow(clippy::needless_lifetimes)]

//! All-purpose [`assert!(...)`](macro.assert.html) and [`check!(...)`](macro.check.html) macros, inspired by [Catch2](https://github.com/catchorg/Catch2).
//! There is also a [`debug_assert!(...)`](macro.debug_assert.html) macro that is disabled on optimized builds by default.
//! As cherry on top there is a [`let_assert!(...)`](macro.let_assert.html) macro that lets you test a pattern while capturing parts of it.
//!
//! # Why these macros?
//!
//! These macros offer some benefits over the assertions from the standard library:
//!   * The macros parse your expression to detect comparisons and adjust the error message accordingly.
//!     No more `assert_eq` or `assert_ne`, just write `assert!(1 + 1 == 2)`, or even `assert!(1 + 1 > 1)`!
//!   * You can test for pattern matches: `assert!(let Err(_) = File::open("/non/existing/file"))`.
//!   * You can capture parts of the pattern for further testing by using the `let_assert!(...)` macro.
//!   * The `check` macro can be used to perform multiple checks before panicking.
//!   * The macros provide more information when the assertion fails.
//!   * Colored failure messages!
//!
//! The macros also accept additional arguments for a custom message, so it is fully compatible with `std::assert`.
//! That means you don't have to worry about overwriting the standard `assert` with `use assert2::assert`.
//!
//! # Examples
//!
//! ```should_panic
//! # use assert2::check;
//! check!(6 + 1 <= 2 * 3);
//! ```
//!
//! ![Assertion error](https://github.com/de-vri-es/assert2-rs/raw/2db44c46e4580ec87d2881a698815e1ec5fcdf3f/binary-operator.png)
//!
//! ----------
//!
//! ```should_panic
//! # use assert2::check;
//! check!(true && false);
//! ```
//!
//! ![Assertion error](https://github.com/de-vri-es/assert2-rs/raw/2db44c46e4580ec87d2881a698815e1ec5fcdf3f/boolean-expression.png)
//!
//! ----------
//!
//! ```should_panic
//! # use assert2::check;
//! # use std::fs::File;
//! check!(let Ok(_) = File::open("/non/existing/file"));
//! ```
//!
//! ![Assertion error](https://github.com/de-vri-es/assert2-rs/raw/2db44c46e4580ec87d2881a698815e1ec5fcdf3f/pattern-match.png)
//!
//! ----------
//!
//! ```should_panic
//! # use assert2::check;
//! # use assert2::let_assert;
//! # use std::fs::File;
//! # use std::io::ErrorKind;
//! let_assert!(Err(e) = File::open("/non/existing/file"));
//! check!(e.kind() == ErrorKind::PermissionDenied);
//! ```
//! ![Assertion error](https://github.com/de-vri-es/assert2-rs/raw/573a686d1f19e0513cb235df38d157defdadbec0/let-assert.png)
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
//!
//! # Controlling colored output.
//!
//! Colored output can be controlled using environment variables,
//! as per the [clicolors spec](https://bixense.com/clicolors/):
//!
//!  * `CLICOLOR != 0`: ANSI colors are supported and should be used when the program isn't piped.
//!  * `CLICOLOR == 0`: Don't output ANSI color escape codes.
//!  * `CLICOLOR_FORCE != 0`: ANSI colors should be enabled no matter what.

#[doc(hidden)]
pub use assert2_macros::check_impl;

#[doc(hidden)]
pub use assert2_macros::let_assert_impl;

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
		if let Err(()) = $crate::check_impl!($crate, "assert", $($tokens)*) {
			panic!("assertion failed");
		}
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
		let _guard = match $crate::check_impl!($crate, "check", $($tokens)*) {
			Ok(_) => None,
			Err(_) => {
				Some($crate::FailGuard(|| panic!("check failed")))
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
			if let Err(()) = $crate::check_impl!($crate, "debug_assert", $($tokens)*) {
				panic!("assertion failed");
			}
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
macro_rules! let_assert {
	($($tokens:tt)*) => {
		$crate::let_assert_impl!($crate, "let_assert", $($tokens)*);
	}
}

#[doc(hidden)]
#[macro_export]
macro_rules! stringify {
	($e:expr) => {
		// Stringifying as an expression gives nicer output
		// than stringifying a raw list of token trees.
		stringify!($e)
	};
	($($t:tt)*) => {
		stringify!($($t)*)
	};
}

#[doc(hidden)]
pub mod maybe_debug;

#[doc(hidden)]
pub mod print;

/// Scope guard to panic when a check!() fails.
///
/// The panic is done by a lambda passed to the guard,
/// so that the line information points to the check!() invocation.
#[doc(hidden)]
pub struct FailGuard<T: FnMut()>(pub T);

impl<T: FnMut()> Drop for FailGuard<T> {
	fn drop(&mut self) {
		if !std::thread::panicking() {
			(self.0)()
		}
	}
}
