extern crate proc_macro;

use proc_macro::Group;
use proc_macro::Ident;
use proc_macro::Punct;
use proc_macro::TokenStream;
use proc_macro::TokenTree;

#[derive(Debug)]
pub struct Placeholder {
	pub original: Punct,
	pub replacement: Ident,
}

struct Replacer {
	placeholder: Option<Placeholder>,
}

impl Replacer {
	fn new() -> Self {
		Self { placeholder: None }
	}

	fn visit_stream(&mut self, stream: TokenStream) -> syn::Result<TokenStream> {
		let mut result = TokenStream::new();
		for tree in stream.into_iter() {
			result.extend(std::iter::once(self.visit_tree(tree)?));
		}
		Ok(result)
	}

	fn visit_tree(&mut self, tree: TokenTree) -> syn::Result<TokenTree> {
		match tree {
			TokenTree::Group(x)   => self.visit_group(x),
			TokenTree::Ident(x)   => Ok(TokenTree::Ident(x)),
			TokenTree::Punct(x)   => self.visit_punct(x),
			TokenTree::Literal(x) => Ok(TokenTree::Literal(x)),
		}
	}

	fn visit_group(&mut self, group: Group) -> syn::Result<TokenTree> {
		let stream = self.visit_stream(group.stream())?;
		let mut result = Group::new(group.delimiter(), stream);
		result.set_span(group.span());
		Ok(TokenTree::Group(result))
	}

	fn visit_punct(&mut self, punct: Punct) -> syn::Result<TokenTree> {
		if punct.as_char() != '#' {
			Ok(TokenTree::Punct(punct))
		} else if self.placeholder.is_some() {
			Err(syn::Error::new(punct.span().into(), "found multiple placeholders in pattern"))
		} else {
			// TODO: punct.span() should be def_site().located_at(punct) once stabilized.
			// Currently, our placeholder can conflict with other placeholders in the pattern.
			let replacement = Ident::new("__ret", punct.span());
			self.placeholder = Some(Placeholder {
				original: punct,
				replacement: replacement.clone(),
			});

			Ok(TokenTree::Ident(replacement))
		}
	}
}

pub fn replace_let_placeholder(stream: TokenStream) -> syn::Result<(TokenStream, Option<Placeholder>)> {
	// Look at the third token to check if this is a let expression.
	// Macro starts with the macro name and a comma, so third token tree is the start of the expression.
	let first_token = stream.clone().into_iter().skip(2).next();
	let is_let_expr = match first_token {
		Some(TokenTree::Ident(x)) => x.to_string() == "let",
		_ => false,
	};

	// If it is, replace `#` with a placeholder to be returned.
	if is_let_expr {
		let mut replacer = Replacer::new();
		let stream = replacer.visit_stream(stream)?;
		Ok((stream, replacer.placeholder))
	} else {
		Ok((stream, None))
	}
}
