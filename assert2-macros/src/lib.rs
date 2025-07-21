#![cfg_attr(nightly, feature(proc_macro_span))]

//! This macro contains only private procedural macros.
//! See the documentation for [`assert2`](https://docs.rs/assert2/) for the public API.

extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{punctuated::Punctuated, spanned::Spanned};

type FormatArgs = Punctuated<syn::Expr, syn::token::Comma>;

#[doc(hidden)]
#[proc_macro]
pub fn check_impl(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
	hygiene_bug::fix(check(syn::parse_macro_input!(tokens)).into())
}

mod assert;
mod hygiene_bug;

#[doc(hidden)]
#[proc_macro]
pub fn assert_impl(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
	hygiene_bug::fix(assert::assert(syn::parse_macro_input!(tokens)).into())
}

struct Context {
	crate_name: syn::Path,
	macro_name: syn::Expr,
	print_predicates: TokenStream,
	fragments: Fragments,
	custom_msg: TokenStream,
}

/// Real implementation for check!().
///
/// Check can not capture placeholders from patterns in the outer scope,
/// since it continues even if the check fails.
///
/// But it does support capturing and additional testing with `&&` chains.
fn check(args: Args) -> TokenStream {
	let predicates = split_predicates(args.expr);
	let mut fragments = Fragments::new();
	let print_predicates = printable_predicates(&args.crate_name, &predicates, &mut fragments);

	let custom_msg = match args.format_args {
		Some(x) => quote!(::core::option::Option::Some(::core::format_args!(#x))),
		None => quote!(::core::option::Option::None),
	};

	let context = Context {
		crate_name: args.crate_name,
		macro_name: args.macro_name,
		print_predicates,
		fragments,
		custom_msg,
	};

	let mut assertions = quote! { ::core::result::Result::Ok::<(), ()>(()) };
	for (i, (_glue, expr)) in predicates.into_iter().enumerate().rev() {
		assertions = match expr {
			syn::Expr::Binary(expr) => check_binary_op(&context, i, expr, assertions),
			syn::Expr::Let(expr) => check_let_expr(&context, i, expr, assertions),
			expr => check_bool_expr(&context, i , expr, assertions),
		};
	}

	assertions
}

fn check_binary_op(
	context: &Context,
	index: usize,
	expr: syn::ExprBinary,
	next_predicate: TokenStream,
) -> TokenStream {
	match expr.op {
		syn::BinOp::Eq(_) => (),
		syn::BinOp::Lt(_) => (),
		syn::BinOp::Le(_) => (),
		syn::BinOp::Ne(_) => (),
		syn::BinOp::Ge(_) => (),
		syn::BinOp::Gt(_) => (),
		_ => return check_bool_expr(context, index, syn::Expr::Binary(expr), next_predicate),
	};

	let syn::ExprBinary { left, right, op, .. } = &expr;
	let op_str = op.to_token_stream().to_string();

	let Context {
		crate_name,
		macro_name,
		print_predicates,
		fragments,
		custom_msg,
	} = context;

	quote! {
		match (&(#left), &(#right)) {
			(left, right) if !(left #op right) => {
				use #crate_name::__assert2_impl::maybe_debug::{IsDebug, IsMaybeNotDebug};
				let left = (&&#crate_name::__assert2_impl::maybe_debug::Wrap(left)).__assert2_maybe_debug().wrap(left);
				let right = (&&#crate_name::__assert2_impl::maybe_debug::Wrap(right)).__assert2_maybe_debug().wrap(right);
				#crate_name::__assert2_impl::print::FailedCheck {
					macro_name: #macro_name,
					file: file!(),
					line: line!(),
					column: column!(),
					predicates: #print_predicates,
					failed: #index,
					expansion: #crate_name::__assert2_impl::print::Expansion::Binary{
						left: &left,
						right: &right,
						operator: #op_str,
					},
					fragments: #fragments,
					custom_msg: #custom_msg,
				}.print();
				::core::result::Result::Err(())
			},
			_ => {
				#next_predicate
			},
		}
	}
}

fn check_bool_expr(
	context: &Context,
	index: usize,
	expr: syn::Expr,
	next_predicate: TokenStream,
) -> TokenStream {
	let Context {
		crate_name,
		macro_name,
		print_predicates,
		fragments,
		custom_msg,
	} = context;

	quote! {
		match #expr {
			true => {
				#next_predicate
			},
			false => {
				#crate_name::__assert2_impl::print::FailedCheck {
					macro_name: #macro_name,
					file: file!(),
					line: line!(),
					column: column!(),
					predicates: #print_predicates,
					failed: #index,
					expansion: #crate_name::__assert2_impl::print::Expansion::Bool,
					fragments: #fragments,
					custom_msg: #custom_msg,
				}.print();
				::core::result::Result::Err(())
			},
		}
	}
}

fn check_let_expr(
	context: &Context,
	index: usize,
	expr: syn::ExprLet,
	next_predicate: TokenStream,
) -> TokenStream {
	let syn::ExprLet {
		pat,
		expr,
		..
	} = expr;

	let Context {
		crate_name,
		macro_name,
		print_predicates,
		fragments,
		custom_msg,
	} = context;

	quote! {
		match (#expr) {
			#pat => {
				#next_predicate
			},
			ref value => {
				use #crate_name::__assert2_impl::maybe_debug::{IsDebug, IsMaybeNotDebug};
				let value = (&&#crate_name::__assert2_impl::maybe_debug::Wrap(value)).__assert2_maybe_debug().wrap(value);
				#crate_name::__assert2_impl::print::FailedCheck {
					macro_name: #macro_name,
					file: file!(),
					line: line!(),
					column: column!(),
					predicates: #print_predicates,
					failed: #index,
					expansion: #crate_name::__assert2_impl::print::Expansion::Let {
						expression: &value,
					},
					fragments: #fragments,
					custom_msg: #custom_msg,
				}.print();
				::core::result::Result::Err(())
			}
		}
	}
}

fn split_predicates(input: syn::Expr) -> Vec<(String, syn::Expr)> {
	let mut output = Vec::new();
	let mut remaining = vec![(String::new(), input)];
	while let Some((outside_glue, input)) = remaining.pop() {
		match input {
			syn::Expr::Binary(expr) if matches!(expr.op, syn::BinOp::And(_)) => {
				let inside_glue = operator_with_whitespace(&expr)
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
	#[cfg(not(feature = "nightly"))]
	{
		let _ = fragments;
	}

	#[cfg(feature = "nightly")]
	{
		use syn::spanned::Spanned;
		find_macro_fragments(tokens.clone(), fragments);
		if let Some(s) = tokens.span().unwrap().source_text() {
			return quote!(#s);
		}
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
				match whitespace_between(end, tree.span()) {
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
					let (whitespace_open, whitespace_close) = whitespace_inside(&group)
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

/// Get the operator of a binary expression with surrounding whitespace (if possible).
fn operator_with_whitespace(expr: &syn::ExprBinary) -> Option<String> {
	let left_spacing = whitespace_between(expr.left.span(), expr.op.span())?;
	let right_spacing = whitespace_between(expr.op.span(), expr.right.span())?;
	Some(format!("{}{}{}", left_spacing, expr.op.into_token_stream(), right_spacing))
}

/// Get the whitespace inside a group, between the delimiters and the contents.
#[cfg(feature = "span-locations")]
fn whitespace_inside(group: &proc_macro2::Group) -> Option<(String, String)> {
	#[cfg(not(feature = "span-locations"))]
	{
		_ = group;
		None
	}

	#[cfg(feature = "span-locations")]
	#[allow(clippy::incompatible_msrv)] // code enabled only for comptabile compilers
	{
		let content = group.stream();
		if content.is_empty() {
			let group_start = group.span().unwrap().start();
			let group_end = group.span().unwrap().end();
			let spacing = if group_end.line() == group_start.line() {
				" ".repeat((group_end.column() - 1).checked_sub(group_start.column() + 1)?)
			} else {
				let lines = group_end.line().checked_sub(group_start.line())?;
				let spaces = group_end.column() - 2;
				let mut output = String::with_capacity(lines + spaces);
				for _ in 0..lines { output.push('\n') };
				for _ in 0..spaces { output.push(' ') };
				output
			};
			return Some((spacing, String::new()));
		}

		let group_start = group.span().unwrap().start();
		let group_end = group.span().unwrap().end();
		let content_span = content.span();
		let content_start = content_span.unwrap().start();
		let content_end = content_span.unwrap().end();

		if group_start.file() != content_start.file() || group_end.file() != content_end.file() {
			return None;
		}

		let spacing_start = if group_start.line() == content_start.line() {
			" ".repeat(content_start.column().checked_sub(group_start.column() + 1)?)
		} else {
			let lines = content_start.line().checked_sub(group_start.line())?;
			let spaces = content_start.column() - 1;
			let mut output = String::with_capacity(lines + spaces);
			for _ in 0..lines { output.push('\n') };
			for _ in 0..spaces { output.push(' ') };
			output
		};

		let spacing_end = if content_end.line() == group_end.line() {
			" ".repeat((group_end.column() - 1).checked_sub(content_end.column())?)
		} else {
			let lines = group_end.line().checked_sub(content_end.line())?;
			let spaces = group_end.column() - 2;
			let mut output = String::with_capacity(lines + spaces);
			for _ in 0..lines { output.push('\n') };
			for _ in 0..spaces { output.push(' ') };
			output
		};

		Some((spacing_start, spacing_end))
	}
}

/// Get the source code between two spans (if possible).
fn whitespace_between(a: Span, b: Span) -> Option<String> {
	#[cfg(not(feature = "span-locations"))]
	{
		let _ = (a, b);
		None
	}
	#[cfg(feature = "span-locations")]
	#[allow(clippy::incompatible_msrv)] // code enabled only for comptabile compilers
	{
		let span_a = a.span().unwrap().end();
		let span_b = b.span().unwrap().start();

		if span_a.file() != span_b.file() {
			return None;
		}
		if span_a.line() == span_b.line() {
			let spaces = match span_b.column().checked_sub(span_a.column()) {
				None => {
					return None;
				}
				Some(x) => x,
			};
			return Some(" ".repeat(spaces));
		}

		let lines = match span_b.line().checked_sub(span_a.line()) {
				None => {
					return None;
				}
				Some(x) => x,
		};
		let spaces = span_b.column().saturating_sub(1);
		let mut output = String::with_capacity(lines + spaces);
		for _ in 0..lines {
			output.push('\n');
		}
		for _ in 0..spaces {
			output.push(' ');
		}
		Some(output)
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

struct Args {
	crate_name: syn::Path,
	macro_name: syn::Expr,
	expr: syn::Expr,
	format_args: Option<FormatArgs>,
}

impl syn::parse::Parse for Args {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let crate_name = input.parse()?;
		let _comma: syn::token::Comma = input.parse()?;
		let macro_name = input.parse()?;
		let _comma: syn::token::Comma = input.parse()?;
		let expr = syn::Expr::parse_without_eager_brace(input)?;
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
			expr,
			format_args,
		})
	}
}
