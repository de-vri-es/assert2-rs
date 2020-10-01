use std::cell::Cell;

thread_local!(static CHECK_STATUS: CheckStatus = CheckStatus::default());

#[derive(Default)]
struct CheckStatus {
	scopes_registered: Cell<usize>,
	panic: Cell<bool>,
}

pub fn register_check_scope() {
	CHECK_STATUS.with(|status| {
		let scopes = status.scopes_registered.get();
		status.scopes_registered.set(scopes + 1);
	});
}

pub fn close_check_scope() {
	let panic = CHECK_STATUS.with(|status| {
		let scopes = status.scopes_registered.get();
		status.scopes_registered.set(scopes - 1);
		status.panic.take()
	});
	if panic {
		panic!("check failed");
	}
}

pub fn check_failed<F: FnOnce() + 'static>(panic: F) -> Option<ScopeGuard<F>> {
	CHECK_STATUS.with(|status| {
		if status.scopes_registered.get() > 0 {
			status.panic.set(true);
			None
		} else {
			Some(ScopeGuard::new(panic))
		}
	})
}

pub struct ScopeGuard<F: FnOnce()> {
	on_drop: Option<F>,
}

impl<F: FnOnce()> ScopeGuard<F> {
	pub fn new(on_drop: F) -> Self {
		let on_drop = Some(on_drop);
		Self { on_drop }
	}
}

impl<F: FnOnce()> Drop for ScopeGuard<F> {
	fn drop(&mut self) {
		(self.on_drop.take().unwrap())()
	}
}

#[macro_export]
macro_rules! check_scope {
	{ $($body:tt)* } => {
		{
			$crate::check_scope::register_check_scope();
			let guard = $crate::check_scope::ScopeGuard::new($crate::check_scope::close_check_scope);
			$($body)*
		}
	}
}
