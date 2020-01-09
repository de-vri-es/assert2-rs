#![feature(proc_macro_hygiene)]
#![feature(specialization)]

//! `assert!(...)` and `check!(...)` macros inspired by Catch2.
//!
//! This crate is currently a work in progress.
//! It relies on a nightly compiler with the `proc_macro_hygiene`, `proc_macro_span` and `specialization` features.

mod maybe_debug;

#[doc(hidden)]
pub mod print;

pub use check_macros::assert;
pub use check_macros::check;

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
