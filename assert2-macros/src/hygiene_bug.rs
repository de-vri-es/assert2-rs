use proc_macro::{Delimiter, Group, TokenStream, TokenTree};

/// Fix-up a token stream to work around a hygiene bug in the Rust compiler.
///
/// This turns all none-delimited groups into parenthesis,
/// so that their precedence remains correct.
///
/// See https://github.com/rust-lang/rust/issues/74036
/// and https://github.com/rust-lang/rust/issues/67062
pub fn fix(tokens: TokenStream) -> TokenStream {
	tokens.into_iter()
		.map(|token| match token {
			TokenTree::Group(group) => {
				let mut fixed = Group::new(
					match group.delimiter() {
						Delimiter::None => Delimiter::Parenthesis,
						delimiter => delimiter,
					},
					fix(group.stream()),
				);
				fixed.set_span(group.span());
				TokenTree::Group(fixed)
			}
			token => token,
		})
		.collect()
}
