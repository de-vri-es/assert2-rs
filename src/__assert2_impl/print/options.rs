/// End-user configurable options for `assert2`.
#[derive(Copy, Clone)]
pub struct AssertOptions {
	/// The expansion format for variables.
	pub expand: ExpansionFormat,

	/// If true, use colors in the output.
	pub color: bool,
}

impl AssertOptions {
	/// Get the global options for `assert2`.
	///
	/// The default format is `ExpansionFormat::Auto`.
	/// This can be overridden by adding the `pretty` or `compact` option to the `ASSERT2` environment variable.
	///
	/// By default, colored output is enabled if `stderr` is conntected to a terminal.
	/// If the `CLICOLOR` environment variable is set to `0`, colored output is disabled by default.
	/// If the `CLICOLOR_FORCE` environment variable is set to something other than `0`,
	/// color is enabled by default, even if `stderr` is not connected to a terminal.
	/// The `color` and `no-color` options in the `ASSERT2` environment variable unconditionally enable and disable colored output.
	///
	/// Multiple options can be combined in the `ASSERT2` environment variable by separating them with a comma.
	/// Whitespace around the comma is ignored.
	/// For example: `ASSERT2=color,pretty` to force colored output and the pretty debug format.
	///
	pub fn get() -> AssertOptions {
		use std::sync::RwLock;

		static STYLE: RwLock<Option<AssertOptions>> = RwLock::new(None);
		loop {
			// If it's already initialized, just return it.
			if let Some(style) = *STYLE.read().unwrap() {
				return style;
			}

			// Style wasn't set yet, so try to get a write lock to initialize the style.
			match STYLE.try_write() {
				// If we fail to get a write lock, another thread is already initializing the style,
				// so we just loop back to the start of the function and try the read lock again.
				Err(_) => continue,

				// If we get the write lock it is up to use to initialize the style.
				Ok(mut style) => {
					let style = style.get_or_insert_with(AssertOptions::from_env);
					if style.color {
						yansi::Paint::enable()
					} else {
						yansi::Paint::disable()
					}
					return *style;
				},
			}
		}
	}

	/// Parse the options from the `ASSERT2` environment variable.
	fn from_env() -> Self {
		// If there is no valid `ASSERT2` environment variable, default to an empty string.
		let format = std::env::var_os("ASSERT2");
		let format = format.as_ref()
			.and_then(|x| x.to_str())
			.unwrap_or("");

		// Start with the defaults.
		let mut output = Self {
			expand: ExpansionFormat::Auto,
			color: should_color(),
		};

		// And modify them based on the options in the environment variables.
		for word in format.split(',') {
			let word = word.trim();
			if word.eq_ignore_ascii_case("pretty") {
				output.expand = ExpansionFormat::Pretty;
			} else if word.eq_ignore_ascii_case("compact") {
				output.expand = ExpansionFormat::Compact;
			} else if word.eq_ignore_ascii_case("color") {
				output.color = true;
			} else if word.eq_ignore_ascii_case("no-color") {
				output.color = false;
			}
		}

		output
	}
}

/// The expansion format for `assert2`.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ExpansionFormat {
	/// Automatically choose compact or pretty output depending on the values.
	///
	/// If the compact debug format for all involved variables is short enough, the compact format is used.
	/// Otherwise, all variables are expanded using the pretty debug format.
	Auto,

	/// Expand variables using the pretty debug format (as with `format!("{:#?}", ..."`).
	Pretty,

	/// Expand variables using the compact debug format (as with `format!("{:?}", ..."`).
	Compact,
}

impl ExpansionFormat {
	/// Check if the format forces the pretty debug format.
	pub fn force_pretty(self) -> bool {
		self == Self::Pretty
	}

	/// Check if the format forces the compact debug format.
	pub fn force_compact(self) -> bool {
		self == Self::Compact
	}

	/// Expand all items according to the style.
	pub fn expand_all<const N: usize>(self, values: [&dyn std::fmt::Debug; N]) -> [String; N] {
		if !self.force_pretty() {
			let expanded = values.map(|x| format!("{x:?}"));
			if self.force_compact() ||  Self::is_compact_good(&expanded) {
				return expanded
			}
		}
		values.map(|x| format!("{x:#?}"))
	}

	/// Heuristicly determine if a compact debug representation is good for all expanded items.
	pub fn is_compact_good(expanded: &[impl AsRef<str>]) -> bool {
		for value in expanded {
			if value.as_ref().len() > 40 {
				return false;
			}
		}
		for value in expanded {
			if value.as_ref().contains('\n') {
				return false;
			}
		}
		true
	}

}

/// Check if the clicolors spec thinks we should use colors.
fn should_color() -> bool {
	use std::ffi::OsStr;

	/// Check if an environment variable has a false-like value.
	///
	/// Returns `false` if the variable is empty.
	fn is_false(value: impl AsRef<OsStr>) -> bool {
		let value = value.as_ref();
		value == "0" || value.eq_ignore_ascii_case("false") || value.eq_ignore_ascii_case("no")
	}

	fn is_true(value: impl AsRef<OsStr>) -> bool {
		let value = value.as_ref();
		value == "1" || value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("yes")
	}

	#[allow(clippy::if_same_then_else)] // shut up clippy
	if std::env::var_os("NO_COLOR").is_some_and(is_true) {
		false
	} else if std::env::var_os("CLICOLOR").is_some_and(is_false) {
		false
	} else if std::env::var_os("CLICOLOR_FORCE").is_some_and(is_true) {
		true
	} else {
		use is_terminal::IsTerminal;
		std::io::stderr().is_terminal()
	}
}
