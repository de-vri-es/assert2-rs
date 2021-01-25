use std::fmt::Debug;

pub struct Wrap<'a, T: ?Sized>(pub &'a T);

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

impl<T: Debug + ?Sized> IsDebug for &Wrap<'_, T> {}
impl<T: ?Sized> IsMaybeNotDebug for Wrap<'_, T> {}

pub struct DebugTag;
pub struct MaybeNotDebugTag;

impl DebugTag {
	pub fn wrap<T: ?Sized>(self, v: &T) -> &T {
		v
	}
}

impl MaybeNotDebugTag {
	pub fn wrap<'a, T: ?Sized>(self, v: &'a T) -> MaybeNotDebug<'a, T> {
		MaybeNotDebug(v)
	}
}

pub struct MaybeNotDebug<'a, T: ?Sized>(&'a T);

impl<'a, T: ?Sized> std::fmt::Debug for MaybeNotDebug<'a, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "<object of type {}>", std::any::type_name::<T>())
	}
}
