//! All-purpose [`assert!(...)`](macro.assert.html) and [`check!(...)`](macro.check.html) macros, inspired by [Catch2](https://github.com/catchorg/Catch2).
//!
//! This crate is currently a work in progress.
//! It relies on a nightly compiler with the `proc_macro_span` and `specialization` features.
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
//! ![Assertion error](https://github.com/de-vri-es/assert2-rs/raw/406f0d065e56db6e3f94c6e2d34b0f2c5b8f0f9f/binary-operator.png)
//!
//! ----------
//!
//! ```should_panic
//! # use assert2::check;
//! check!(true && false);
//! ```
//!
//! ![Assertion error](https://github.com/de-vri-es/assert2-rs/raw/406f0d065e56db6e3f94c6e2d34b0f2c5b8f0f9f/boolean-expression.png)
//!
//! ----------
//!
//! ```should_panic
//! # use assert2::check;
//! # use std::fs::File;
//! check!(let Ok(_) = File::open("/non/existing/file"));
//! ```
//!
//! ![Assertion error](https://github.com/de-vri-es/assert2-rs/raw/406f0d065e56db6e3f94c6e2d34b0f2c5b8f0f9f/pattern-match.png)
//!
//! # `assert` vs `check`
//! The crate provides two macros: `check!(...)` and `assert!(...)`.
//! The main difference is that `check` doesn't immediately panic.
//! Instead, it will print the assertion error and fail the test.
//! This allows you to run multiple checks and can help to determine the reason of a test failure more easily.
//!
//! Currently, `check` uses a scope guard to delay the panic until the current scope ends.
//! Ideally, `check` doesn't panic at all, but only signals that a test case has failed.
//! If this becomes possible in the future, the `check` macro will change, so **you should not rely on `check` to panic**.
//!
//! # Controlling colored output.
//!
//! You can force colored output on or off by setting the `CLICOLOR` environment variable.
//! Set `CLICOLOR=1` to forcibly enable colors, or `CLICOLORS=0` to disable them.
//! If the environment variable is unset or set to `auto`, output will be colored if it is going to a terminal.

use proc_macro_hack::proc_macro_hack;

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
#[proc_macro_hack]
pub use assert2_macros::assert;

#[proc_macro_hack]
pub use assert2_macros::check_impl;

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
		let _guard = ::assert2::check_impl!($($tokens)*);
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
