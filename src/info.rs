use std::sync::Mutex;
use yansi::Paint;

thread_local!(static INFO: Mutex<Vec<Message>> = Mutex::new(Vec::new()));

/// The message for [`Info`]
enum Message {
	String(String),
	Capture(Capture),
}

/// A captured expression.
struct Capture {
	/// The expression.
	expression: &'static str,

	/// The pretty printed Debug form of the value.
	value: String,
}

/// Push a string message to the thread local stack of information messages.
pub fn push_message(message: String) -> InfoGuard {
	INFO.with(|info| info.lock().unwrap().push(Message::String(message)));
	InfoGuard
}

/// Push a captured expression to the thread local stack of information messages.
pub fn push_capture(expression: &'static str, value: String) -> InfoGuard {
	let capture = Capture { expression, value };
	INFO.with(|info| info.lock().unwrap().push(Message::Capture(capture)));
	InfoGuard
}

/// Remove the last entry from the thread local stack of information messages.
pub fn pop_info() {
	INFO.with(|info| info.lock().unwrap().pop());
}

/// A scope guard that pops the last info message when dropped.
pub struct InfoGuard;

impl Drop for InfoGuard {
	fn drop(&mut self) {
		pop_info()
	}
}

pub fn info_count() -> usize {
	INFO.with(|info| info.lock().unwrap().len())
}

/// Print and clear the thread local stack of information messages.
pub fn print_info() {
	INFO.with(|info| {
		let info = info.lock().unwrap();
		for info in info.iter() {
			eprintln!("  {}", info);
		};
	})
}

/// Add an informational message to be printed on the next assertion failures.
///
/// The message will only be printed once and are bound to the current scope.
/// At scope exit, the message is cleared regardless if it was printed or not.
#[macro_export]
macro_rules! info {
	($($args:tt)*) => {
		let guard = $crate::info::push_message(format!($($args)*));
	};
}

/// Capture an expression to be printed on assertion failures.
///
/// The message will only be printed once and are bound to the current scope.
/// At scope exit, the message is cleared regardless if it was printed or not.
#[macro_export]
macro_rules! capture {
	($expr:expr) => {
		let value = $crate::info::indent(&format!("{:#?}", $expr));
		let guard = $crate::info::push_capture(stringify!($expr), value);
	}
}

pub fn indent(input: &str) -> String {
	input.replace("\n", "\n  ")
}

impl std::fmt::Display for Message {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::String(x) => x.fmt(f),
			Self::Capture(x) => x.fmt(f),
		}
	}
}

impl std::fmt::Display for Capture {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{expr} {eq} {value}",
			expr = self.expression,
			eq = Paint::blue("=").bold(),
			value = self.value,
		)
	}
}
