#![feature(proc_macro_hygiene)]
#![feature(specialization)]

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
