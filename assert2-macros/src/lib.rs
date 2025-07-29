#![cfg_attr(nightly, feature(proc_macro_span))]

//! This macro contains only private procedural macros.
//! See the documentation for [`assert2`](https://docs.rs/assert2/) for the public API.

extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

type FormatArgs = syn::punctuated::Punctuated<syn::Expr, syn::token::Comma>;

mod assert;
mod check;
mod hygiene_bug;
mod whitespace;

#[doc(hidden)]
#[proc_macro]
pub fn assert_impl(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
	hygiene_bug::fix(assert::assert(syn::parse_macro_input!(tokens)).into())
}

#[doc(hidden)]
#[proc_macro]
pub fn check_impl(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
	hygiene_bug::fix(check::check(syn::parse_macro_input!(tokens)).into())
}

/// Parsed arguments for the `check` or `assert` macro.
struct Args {
	/// The path of the `assert2` crate.
	crate_name: syn::Path,

	/// The name of the macro being called.
	macro_name: syn::Expr,

	/// The expression passed to the macro,
	expression: syn::Expr,

	/// Optional extra message (all arguments forwarded to format_args!()).
	format_args: Option<FormatArgs>,
}

impl syn::parse::Parse for Args {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let crate_name = input.parse()?;
		let _comma: syn::token::Comma = input.parse()?;
		let macro_name = input.parse()?;
		let _comma: syn::token::Comma = input.parse()?;
		let expression = syn::Expr::parse_without_eager_brace(input)?;
		let format_args = if input.is_empty() {
			FormatArgs::new()
		} else {
			input.parse::<syn::token::Comma>()?;
			FormatArgs::parse_terminated(input)?
		};

		let format_args = Some(format_args).filter(|x| !x.is_empty());
		Ok(Self {
			crate_name,
			macro_name,
			expression,
			format_args,
		})
	}
}

impl Args {
	fn into_context(self) -> Context {
		let predicates = split_predicates(self.expression);
		let mut fragments = Fragments::new();
		let print_predicates = printable_predicates(&self.crate_name, &predicates, &mut fragments);

		let custom_msg = match self.format_args {
			Some(x) => quote!(::core::option::Option::Some(::core::format_args!(#x))),
			None => quote!(::core::option::Option::None),
		};

		Context {
			crate_name: self.crate_name,
			macro_name: self.macro_name,
			predicates,
			print_predicates,
			fragments,
			custom_msg,
		}
	}
}

struct Context {
	crate_name: syn::Path,
	macro_name: syn::Expr,
	predicates: Vec<(String, syn::Expr)>,
	print_predicates: TokenStream,
	fragments: Fragments,
	custom_msg: TokenStream,
}

fn split_predicates(input: syn::Expr) -> Vec<(String, syn::Expr)> {
	let mut output = Vec::new();
	let mut remaining = vec![(String::new(), input)];
	while let Some((outside_glue, input)) = remaining.pop() {
		match input {
			syn::Expr::Binary(expr) if matches!(expr.op, syn::BinOp::And(_)) => {
				let inside_glue = whitespace::operator_with_whitespace(&expr)
					.unwrap_or_else(|| String::from(" && "));
				remaining.push((inside_glue, *expr.right));
				remaining.push((outside_glue, *expr.left));
			},
			other => output.push((outside_glue, other)),
		}
	}
	output
}

fn printable_predicates(crate_name: &syn::Path, predicates: &[(String, syn::Expr)], fragments: &mut Fragments) -> TokenStream {
	let mut printable_predicates = Vec::new();
	for (glue, predicate) in predicates {
		let expresion = match predicate {
			syn::Expr::Let(expr) => {
				let pattern = expression_to_string(crate_name, expr.pat.to_token_stream(), fragments);
				let expression = expression_to_string(crate_name, expr.expr.to_token_stream(), fragments);
				quote! {
					(
						#glue,
						#crate_name::__assert2_impl::print::Predicate::Let {
							pattern: #pattern,
							expression: #expression,
						},
					)
				}
			},
			syn::Expr::Binary(expr) => {
				let left = expression_to_string(crate_name, expr.left.to_token_stream(), fragments);
				let operator = tokens_to_string(expr.op.to_token_stream(), fragments);
				let right = expression_to_string(crate_name, expr.right.to_token_stream(), fragments);
				quote! {
					(
						#glue,
						#crate_name::__assert2_impl::print::Predicate::Binary {
							left: #left,
							operator: #operator,
							right: #right,
						},
					)
				}
			},
			expr => {
				let expression = expression_to_string(crate_name, expr.to_token_stream(), fragments);
				quote! {
					(
						#glue,
						#crate_name::__assert2_impl::print::Predicate::Bool {
							expression: #expression,
						},
					)
				}
			},
		};
		printable_predicates.push(expresion);
	}
	quote!( &[#(#printable_predicates),*] )
}

fn tokens_to_string(tokens: TokenStream, fragments: &mut Fragments) -> TokenStream {
	#[cfg(feature = "nightly")]
	{
		use syn::spanned::Spanned;
		find_macro_fragments(tokens.clone(), fragments);
		if let Some(s) = tokens.span().unwrap().source_text() {
			return quote!(#s);
		}
	}

	#[cfg(not(feature = "nightly"))]
	{
		let _ = fragments;
	}

	#[cfg(feature = "span-locations")]
	{
		let mut output = String::new();
		let mut end = None;
		let mut streams = vec![(tokens.into_iter(), None::<(char, String, proc_macro2::Span)>)];
		while let Some((mut stream, delimiter)) = streams.pop() {
			let tree = match stream.next() {
				None => {
					if let Some((delimiter, whitespace, delim_span)) = delimiter {
						output.push_str(&whitespace);
						output.push(delimiter);
						end = Some(delim_span);
					}
					continue;
				},
				Some(tree) => tree,
			};
			streams.push((stream, delimiter));

			if let Some(end) = end {
				match whitespace::whitespace_between(end, tree.span()) {
					Some(whitespace) => output.push_str(&whitespace),
					None => {
						print!("Failed to determine whitespace before tree");
						output.push(' ');
					},
				};
			};

			match tree {
				proc_macro2::TokenTree::Ident(ident) => {
					output.push_str(&ident.to_string());
					end = Some(ident.span());
				},
				proc_macro2::TokenTree::Punct(punct) => {
					output.push(punct.as_char());
					end = match punct.spacing() {
						proc_macro2::Spacing::Joint => None,
						proc_macro2::Spacing::Alone => Some(punct.span()),
					};
				},
				proc_macro2::TokenTree::Literal(literal) => {
					output.push_str(&literal.to_string());
					end = Some(literal.span());
				},
				proc_macro2::TokenTree::Group(group) => {
					let (whitespace_open, whitespace_close) = whitespace::whitespace_inside(&group)
						.unwrap_or((String::new(), String::new()));
					let (open, close) = match group.delimiter() {
						proc_macro2::Delimiter::None => ('(', ')'),
						proc_macro2::Delimiter::Parenthesis => ('(', ')'),
						proc_macro2::Delimiter::Brace => ('{', '}'),
						proc_macro2::Delimiter::Bracket => ('[', ']'),
					};
					output.push(open);
					output.push_str(&whitespace_open);
					let stream = group.stream();
					if !stream.is_empty() {
						end = None;
						streams.push((stream.into_iter(), Some((close, whitespace_close, group.span_close()))));
					} else {
						output.push_str(&whitespace_close);
						output.push(close);
						end = Some(group.span_close());
					}
				},
			}

		}

		quote!(#output)
	}

	#[cfg(not(feature = "span-locations"))]
	{
		let tokens = tokens.to_string();
		quote!(#tokens)
	}
}

fn expression_to_string(crate_name: &syn::Path, tokens: TokenStream, fragments: &mut Fragments) -> TokenStream {
	#[cfg(feature = "nightly")]
	{
		let _ = crate_name;
		use syn::spanned::Spanned;
		find_macro_fragments(tokens.clone(), fragments);
		if let Some(s) = tokens.span().unwrap().source_text() {
			return quote!(#s);
		}
	}

	#[cfg(feature = "span-locations")]
	{
		let _ = crate_name;
		tokens_to_string(tokens, fragments)
	}

	#[cfg(not(feature = "span-locations"))]
	{
		let _ = fragments;
		quote!(#crate_name::__assert2_stringify!(#tokens))
	}
}

#[cfg(feature = "nightly")]
fn find_macro_fragments(tokens: TokenStream, f: &mut Fragments) {
	use syn::spanned::Spanned;
	use proc_macro2::{Delimiter, TokenTree};

	for token in tokens {
		if let TokenTree::Group(g) = token {
			if g.delimiter() == Delimiter::None {
				let name = g.span().unwrap().source_text().unwrap_or_else(|| "???".into());
				let contents = g.stream();
				let expansion = contents.span().unwrap().source_text().unwrap_or_else(|| contents.to_string());
				if name != expansion {
					let entry = (name, expansion);
					if !f.list.contains(&entry) {
						f.list.push(entry);
					}
				}
			}
			find_macro_fragments(g.stream(), f);
		}
	}
}


struct Fragments {
	list: Vec<(String, String)>,
}

impl Fragments {
	fn new() -> Self {
		Self { list: Vec::new() }
	}
}

impl quote::ToTokens for Fragments {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let mut t = TokenStream::new();
		for (name, expansion) in &self.list {
			t.extend(quote!((#name, #expansion),));
		}
		tokens.extend(quote!(&[#t]));
	}
}
