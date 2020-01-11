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

impl<T> IsMaybeNotDebug for &T {
	type Wrapper = MaybeNotDebug<T>;

	fn __assert2_wrap_debug(&self) -> MaybeNotDebug<T> {
		MaybeNotDebug(std::marker::PhantomData)
	}
}

pub struct MaybeNotDebug<T: ?Sized>(std::marker::PhantomData<T>);

impl<T> std::fmt::Debug for MaybeNotDebug<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "<object of type {}>", std::any::type_name::<T>())
	}
}
