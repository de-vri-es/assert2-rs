use std::fmt::Debug;

pub struct Wrap<'a, T>(pub &'a T);

pub trait IsDebug {
	fn __assert2_maybe_debug(&self) -> DebugTag {
		DebugTag
	}
}

pub trait IsMaybeNotDebug {
	fn __assert2_maybe_debug(&self) -> MaybeNotDebugTag {
		MaybeNotDebugTag
	}
}

impl<T: Debug> IsDebug for &Wrap<'_, T> {}
impl<T> IsMaybeNotDebug for Wrap<'_, T> {}

pub struct DebugTag;
pub struct MaybeNotDebugTag;

impl DebugTag {
	pub fn wrap<T>(self, v: &T) -> &T {
		v
	}
}

impl MaybeNotDebugTag {
	pub fn wrap<'a, T>(self, v: &'a T) -> MaybeNotDebug<'a, T> {
		MaybeNotDebug(v)
	}
}

pub struct MaybeNotDebug<'a, T>(&'a T);

impl<'a, T> std::fmt::Debug for MaybeNotDebug<'a, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "<object of type {}>", std::any::type_name::<T>())
	}
}

#[macro_export]
macro_rules! wrap {
	(&$var:ident) => {
		use ::assert2::maybe_debug::{IsDebug, IsMaybeNotDebug};
		let wrap = ::assert2::maybe_debug::Wrap($var);
		(&&wrap).__assert2_maybe_debug().wrap(wrap)
	}
}
