use proc_macro::{Delimiter, Group, TokenStream, TokenTree};

pub fn fix(ts: TokenStream) -> TokenStream {
	ts.into_iter()
		.map(|t| match t {
			TokenTree::Group(g) => {
				let mut fixed = Group::new(
					match g.delimiter() {
						Delimiter::None => Delimiter::Parenthesis,
						d => d,
					},
					fix(g.stream()),
				);
				fixed.set_span(g.span());
				TokenTree::Group(fixed)
			}
			t => t,
		})
		.collect()
}
