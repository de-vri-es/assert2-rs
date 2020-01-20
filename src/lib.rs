#![allow(clippy::needless_lifetimes)]

//! All-purpose [`assert!(...)`](macro.assert.html) and [`check!(...)`](macro.check.html) macros, inspired by [Catch2](https://github.com/catchorg/Catch2).
//! There is also a [`debug_assert!(...)`](macro.debug_assert.html) macro that is disabled on optimized builds by default.
//!
//! # Why these macros?
//!
//! These macros offer some benefits over the assertions from the standard library:
//!   * The macros parse your expression to detect comparisons and adjust the error message accordingly.
//!     No more `assert_eq` or `assert_ne`, just write `assert!(1 + 1 == 2)`, or even `assert!(1 + 1 > 1)`!
//!   * You can test for pattern matches: `assert!(let Err(_) = File::open("/non/existing/file"))`.
//!   * The `check` macro can be used to perform multiple checks before panicking.
//!   * The macros provide more information when the assertion fails.
//!   * Colored failure messages!
//!
//! The macros also accept additional arguments for a custom message, so it is fully comptabible with `std::assert`.
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
//! This makes the output a bit more readable on nightly,
//! but the differences are limited to the displayed expression.
//!
//! # Controlling colored output.
//!
//! You can force colored output on or off by setting the `CLICOLOR` environment variable.
//! Set `CLICOLOR=1` to forcibly enable colors, or `CLICOLORS=0` to disable them.
//! If the environment variable is unset or set to `auto`, output will be colored if it is going to a terminal.

use proc_macro_hack::proc_macro_hack;

#[doc(hidden)]
#[proc_macro_hack]
pub use assert2_macros::check_impl;

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
/// # use ::assert2::assert;
/// assert!(3 * 4 == 12, "Oh no, math is broken! 1 + 1 == {}", 1 + 1);
/// ```
#[macro_export]
macro_rules! assert {
	($($tokens:tt)*) => {
		match ::assert2::check_impl!("assert", $($tokens)*) {
			Ok(x) => x,
			Err(()) => panic!("assertion failed"),
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
/// # use ::assert2::check;
/// check!(3 * 4 == 12, "Oh no, math is broken! 1 + 1 == {}", 1 + 1);
/// ```
#[macro_export]
macro_rules! check {
	($($tokens:tt)*) => {
		let _guard = match ::assert2::check_impl!("check", $($tokens)*) {
			Ok(_) => None,
			Err(_) => {
				Some(::assert2::FailGuard(|| panic!("check failed")))
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
			if let Err(()) = ::assert2::check_impl!("debug_assert", $($tokens)*) {
				panic!("assertion failed");
			}
		}
	}
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
