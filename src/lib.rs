#![feature(proc_macro_hygiene)]
#![feature(specialization)]

//! [`assert!(...)`](macro.assert.html) and [`check!(...)`](macro.check.html) macros inspired by Catch2.
//!
//! This crate is currently a work in progress.
//! It relies on a nightly compiler with the `proc_macro_hygiene`, `proc_macro_span` and `specialization` features.
//! As a user of the crate, you also need to enable the `proc_macro_hygiene` feature.
//!
//! Unlike the asserts in standard library, there is no difference between `assert`, `assert_eq` or others.
//! Instead, you write all checks with the same [`assert!(...)`](macro.assert.html) or [`check!(...)`](macro.check.html) macros.
//! The macros recognize what kind of expression you wrote and will provide a useful error message with colored output when the check fails.
//!
//! Also unlike the standard `std::assert`, the macros from this crate show both the original expression and the value of the expression.
//! The standard library asserts only show the value of the expression.
//!
//! Finally, in addition to boolean expressions, you can test if a value matches a pattern by putting a `let` expression in the macro.
//! A quick example:
//!
//! ```
//! # #![feature(proc_macro_hygiene)]
//! # use assert2::check;
//! # use std::fs::File;
//! check!(6 == 2 * 3);
//! check!(true || false);
//! check!(let Err(_) = File::open("/non/existing/file"));
//! ```
//!
//! # Sample output
//!
//! ```should_panic
//! # #![feature(proc_macro_hygiene)]
//! # use assert2::check;
//! check!(6 + 1 <= 2 * 3);
//! ```
//!
//! ![Assertion error](https://github.com/de-vri-es/assert2-rs/blob/406f0d065e56db6e3f94c6e2d34b0f2c5b8f0f9f/binary-operator.png)
//!
//! ```should_panic
//! # #![feature(proc_macro_hygiene)]
//! # use assert2::check;
//! check!(true && false);
//! ```
//!
//! ![Assertion error](https://github.com/de-vri-es/assert2-rs/blob/406f0d065e56db6e3f94c6e2d34b0f2c5b8f0f9f/boolean-expression.png)
//!
//! ```should_panic
//! # #![feature(proc_macro_hygiene)]
//! # use assert2::check;
//! # use std::fs::File;
//! check!(let Ok(_) = File::open("/non/existing/file"));
//! ```
//!
//! ![Assertion error](https://github.com/de-vri-es/assert2-rs/blob/406f0d065e56db6e3f94c6e2d34b0f2c5b8f0f9f/pattern-match.png)
//!
//! # `assert` vs `check`
//! The crate provides two macros: `check!(...)` and `assert!(...)`.
//! The main difference is that check doesn't immediately panic.
//! Instead, it will print the assertion error and fail the test.
//! This allows you to run multiple checks, and can help to paint a clearer picture why a test failed.
//!
//! Currently, `check` uses a scope guard to delay the panic until the current scope ends.
//! Ideally, `check` should not panic until the whole test body has finished.
//! Even better would be if `check` doesn't panic at all, but simply marks the test as failed.
//! If this becomes possible in the future, the `check` macro will change, so *you should not rely on `check` to panic*.
//!
//! # Controlling colors.
//!
//! You can force colored output on or off by setting the `CLICOLOR` environment variable.
//! Set `CLICOLOR=1` to forcibly enable colors, or `CLICOLORS=0` to disable them.
//! If the environment variable is unset or set to `auto`, output will be colored if it is going to a terminal.

pub use assert2_macros::assert;
pub use assert2_macros::check;

mod maybe_debug;

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
