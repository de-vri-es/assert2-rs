#![feature(proc_macro_hygiene)]
#![feature(specialization)]

//! `assert!(...)` and `check!(...)` macros inspired by Catch2.
//!
//! This crate is currently a work in progress.
//! It relies on a nightly compiler with the `proc_macro_hygiene`, `proc_macro_span` and `specialization` features.
//!
//! As a user of the crate, you also need to enable the `proc_macro_hygiene` feature.
//!
//! # Example
//! ```should_panic
//! #![feature(proc_macro_hygiene)]
//! use assert2::check;
//!
//! # fn main() {
//! let mut vec = Vec::new();
//! vec.push(12);
//!
//! check!(vec.len() == 2);
//! check!(&vec == &vec![10]);
//! # }
//! ```
//!
//! ![Example output](https://github.com/de-vri-es/assert2-rs/blob/v0.0.3/example.png)

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
