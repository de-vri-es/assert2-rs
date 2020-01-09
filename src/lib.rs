#![feature(proc_macro_hygiene)]
#![feature(specialization)]

mod maybe_debug;

#[doc(hidden)]
pub mod print;

pub use check_macros::assert;
pub use check_macros::check;

#[doc(hidden)]
pub struct FailGuard<T: FnMut()>(pub T);

impl<T: FnMut()> Drop for FailGuard<T> {
	fn drop(&mut self) {
		if !std::thread::panicking() {
			(self.0)()
		}
	}
}
