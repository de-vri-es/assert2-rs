use std::fmt::Debug;

pub trait IsDebug {
	fn __assert2_wrap_debug(&self) -> &Self;
}

pub trait IsMaybeNotDebug {
	type Wrapper;

	fn __assert2_wrap_debug(&self) -> Self::Wrapper;
}

impl<T: Debug> IsDebug for T {
	fn __assert2_wrap_debug(&self) -> &T {
		self
	}
}

impl<'a, T> IsMaybeNotDebug for &'a T {
	type Wrapper = MaybeNotDebug<'a, T>;

	fn __assert2_wrap_debug(&self) -> MaybeNotDebug<'a, T> {
		MaybeNotDebug(*self)
	}
}

pub struct MaybeNotDebug<'a, T: ?Sized>(&'a T);

impl<'a, T> std::fmt::Debug for MaybeNotDebug<'a, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "<object of type {}>", std::any::type_name::<T>())
	}
}
