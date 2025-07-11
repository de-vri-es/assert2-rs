#![cfg_attr(nightly, feature(proc_macro_span))]

//! This macro contains only private procedural macros.
//! See the documentation for [`assert2`](https://docs.rs/assert2/) for the public API.

extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;

type FormatArgs = Punctuated<syn::Expr, syn::token::Comma>;

#[doc(hidden)]
#[proc_macro]
pub fn check_impl(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
	hygiene_bug::fix(check_or_assert_impl(syn::parse_macro_input!(tokens)).into())
}

mod hygiene_bug;
mod let_assert;

#[doc(hidden)]
#[proc_macro]
pub fn let_assert_impl(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
	hygiene_bug::fix(let_assert::let_assert_impl(syn::parse_macro_input!(tokens)).into())
}

struct Context {
	crate_name: syn::Path,
	macro_name: syn::Expr,
	print_predicates: TokenStream,
	fragments: Fragments,
	custom_msg: TokenStream,
}

/// Real implementation for assert!() and check!().
fn check_or_assert_impl(args: Args) -> TokenStream {
	let mut output = None;

	let predicates = split_predicates(args.expr);
	let mut fragments = Fragments::new();
	let print_predicates = printable_predicates(&args.crate_name, &predicates, &mut fragments);

	let custom_msg = match args.format_args {
		Some(x) => quote!(Some(format_args!(#x))),
		None => quote!(None),
	};

	let context = Context {
		crate_name: args.crate_name,
		macro_name: args.macro_name,
		print_predicates,
		fragments,
		custom_msg,
	};

	for (i, expr) in predicates.into_iter().enumerate() {
		let tokens = match expr {
			syn::Expr::Binary(expr) => check_binary_op(&context, i, expr),
			syn::Expr::Let(expr) => check_let_expr(&context, i, expr),
			expr => check_bool_expr(&context, i , expr),
		};
		output = match output.take() {
			None => Some(tokens),
			Some(output) => Some(quote! { #output.and_then(|()| #tokens) }),
		};
	}
	output.unwrap_or_else(|| quote!(Ok::<(), ()>::(())))
}

fn check_binary_op(
	context: &Context,
	index: usize,
	expr: syn::ExprBinary,
) -> TokenStream {
	match expr.op {
		syn::BinOp::Eq(_) => (),
		syn::BinOp::Lt(_) => (),
		syn::BinOp::Le(_) => (),
		syn::BinOp::Ne(_) => (),
		syn::BinOp::Ge(_) => (),
		syn::BinOp::Gt(_) => (),
		_ => return check_bool_expr(context, index, syn::Expr::Binary(expr)),
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
				Err(())
			}
			_ => Ok(()),
		}
	}
}

fn check_bool_expr(
	context: &Context,
	index: usize,
	expr: syn::Expr,
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
				Err(())
			}
			true => Ok(()),
		}
	}
}

fn check_let_expr(
	context: &Context,
	index: usize,
	expr: syn::ExprLet,
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
		match &(#expr) {
			#pat => Ok(()),
			value => {
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
				Err(())
			}
		}
	}
}

fn split_predicates(input: syn::Expr) -> Vec<syn::Expr> {
	let mut output = Vec::new();
	let mut remaining = vec![input];
	while let Some(input) = remaining.pop() {
		match input {
			syn::Expr::Binary(expr) if matches!(expr.op, syn::BinOp::And(_)) => {
				remaining.push(*expr.right);
				remaining.push(*expr.left);
			},
			other => output.push(other),
		}
	}
	output
}

fn printable_predicates(crate_name: &syn::Path, predicates: &[syn::Expr], fragments: &mut Fragments) -> TokenStream {
	let mut printable_predicates = Vec::new();
	for predicate in predicates {
		let expresion = match predicate {
			syn::Expr::Let(expr) => {
				let pattern = expression_to_string(crate_name, expr.pat.to_token_stream(), fragments);
				let expression = expression_to_string(crate_name, expr.expr.to_token_stream(), fragments);
				quote! {
					#crate_name::__assert2_impl::print::Predicate::Let {
						pattern: #pattern,
						expression: #expression,
					}
				}
			},
			syn::Expr::Binary(expr) => {
				let left = expression_to_string(crate_name, expr.left.to_token_stream(), fragments);
				let operator = tokens_to_string(expr.op.to_token_stream(), fragments);
				let right = expression_to_string(crate_name, expr.right.to_token_stream(), fragments);
				quote! {
					#crate_name::__assert2_impl::print::Predicate::Binary {
						left: #left,
						operator: #operator,
						right: #right,
					}
				}
			},
			expr => {
				let expression = expression_to_string(crate_name, expr.to_token_stream(), fragments);
				quote! {
					#crate_name::__assert2_impl::print::Predicate::Bool {
						expression: #expression,
					}
				}
			},
		};
		printable_predicates.push(expresion);
	}
	quote!( &[#(#printable_predicates),*] )
}

fn tokens_to_string(ts: TokenStream, fragments: &mut Fragments) -> TokenStream {
	#[cfg(nightly)]
	{
		use syn::spanned::Spanned;
		find_macro_fragments(ts.clone(), fragments);
		if let Some(s) = ts.span().unwrap().source_text() {
			return quote!(#s);
		}
	}

	let _ = fragments;

	let tokens = ts.to_string();
	quote!(#tokens)
}

fn expression_to_string(crate_name: &syn::Path, ts: TokenStream, fragments: &mut Fragments) -> TokenStream {
	#[cfg(nightly)]
	{
		use syn::spanned::Spanned;
		find_macro_fragments(ts.clone(), fragments);
		if let Some(s) = ts.span().unwrap().source_text() {
			return quote!(#s);
		}
	}

	let _ = fragments;

	quote!(#crate_name::__assert2_stringify!(#ts))
}

#[cfg(nightly)]
fn find_macro_fragments(ts: TokenStream, f: &mut Fragments) {
	use syn::spanned::Spanned;
	use proc_macro2::{Delimiter, TokenTree};

	for token in ts {
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
