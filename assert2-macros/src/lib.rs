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

/// Real implementation for assert!() and check!().
fn check_or_assert_impl(args: Args) -> TokenStream {
	match args.expr {
		syn::Expr::Binary(expr) => check_binary_op(args.macro_name, expr, args.format_args),
		syn::Expr::Let(expr) => check_let_expr(args.macro_name, expr, args.format_args),
		expr => check_bool_expr(args.macro_name, expr, args.format_args),
	}
}

fn check_binary_op(macro_name: syn::Expr, expr: syn::ExprBinary, format_args: Option<FormatArgs>) -> TokenStream {
	match expr.op {
		syn::BinOp::Eq(_) => (),
		syn::BinOp::Lt(_) => (),
		syn::BinOp::Le(_) => (),
		syn::BinOp::Ne(_) => (),
		syn::BinOp::Ge(_) => (),
		syn::BinOp::Gt(_) => (),
		_ => return check_bool_expr(macro_name, syn::Expr::Binary(expr), format_args),
	};

	let syn::ExprBinary { left, right, op, .. } = &expr;
	let mut fragments = Fragments::new();
	let left_expr = tokens_to_string(left.to_token_stream(), &mut fragments);
	let right_expr = tokens_to_string(right.to_token_stream(), &mut fragments);
	let op_str = tokens_to_string(op.to_token_stream(), &mut fragments);

	let custom_msg = match format_args {
		Some(x) => quote!(Some(format_args!(#x))),
		None => quote!(None),
	};

	quote! {
		match (&(#left), &(#right)) {
			(left, right) if !(left #op right) => {
				use ::assert2::maybe_debug::{IsDebug, IsMaybeNotDebug};
				let left = (&&::assert2::maybe_debug::Wrap(left)).__assert2_maybe_debug().wrap(left);
				let right = (&&::assert2::maybe_debug::Wrap(right)).__assert2_maybe_debug().wrap(right);
				::assert2::print::FailedCheck {
					macro_name: #macro_name,
					file: file!(),
					line: line!(),
					column: column!(),
					custom_msg: #custom_msg,
					expression: ::assert2::print::BinaryOp {
						left: &left,
						right: &right,
						operator: #op_str,
						left_expr: #left_expr,
						right_expr: #right_expr,
					},
					fragments: #fragments,
				}.print();
				Err(())
			}
			_ => Ok(()),
		}
	}
}

fn check_bool_expr(macro_name: syn::Expr, expr: syn::Expr, format_args: Option<FormatArgs>) -> TokenStream {
	let mut fragments = Fragments::new();
	let expr_str = tokens_to_string(expr.to_token_stream(), &mut fragments);

	let custom_msg = match format_args {
		Some(x) => quote!(Some(format_args!(#x))),
		None => quote!(None),
	};

	quote! {
		match #expr {
			false => {
				::assert2::print::FailedCheck {
					macro_name: #macro_name,
					file: file!(),
					line: line!(),
					column: column!(),
					custom_msg: #custom_msg,
					expression: ::assert2::print::BooleanExpr {
						expression: #expr_str,
					},
					fragments: #fragments,
				}.print();
				Err(())
			}
			true => Ok(()),
		}
	}
}

fn check_let_expr(macro_name: syn::Expr, expr: syn::ExprLet, format_args: Option<FormatArgs>) -> TokenStream {
	let syn::ExprLet {
		pat,
		expr,
		..
	} = expr;

	let mut fragments = Fragments::new();
	let pat_str = tokens_to_string(pat.to_token_stream(), &mut fragments);
	let expr_str = tokens_to_string(expr.to_token_stream(), &mut fragments);

	let custom_msg = match format_args {
		Some(x) => quote!(Some(format_args!(#x))),
		None => quote!(None),
	};

	quote! {
		match &(#expr) {
			#pat => Ok(()),
			value => {
				use ::assert2::maybe_debug::{IsDebug, IsMaybeNotDebug};
				let value = (&&::assert2::maybe_debug::Wrap(value)).__assert2_maybe_debug().wrap(value);
				::assert2::print::FailedCheck {
					macro_name: #macro_name,
					file: file!(),
					line: line!(),
					column: column!(),
					custom_msg: #custom_msg,
					expression: ::assert2::print::MatchExpr {
						print_let: true,
						value: &value,
						pattern: #pat_str,
						expression: #expr_str,
					},
					fragments: #fragments,
				}.print();
				Err(())
			}
		}
	}
}

fn tokens_to_string(ts: TokenStream, _fragments: &mut Fragments) -> TokenStream {
	#[cfg(nightly)]
	{
		use syn::spanned::Spanned;
		find_macro_fragments(ts.clone(), _fragments);
		if let Some(s) = ts.span().unwrap().source_text() {
			return quote!(#s);
		}
	}
	quote!(::assert2::stringify!(#ts))
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
	macro_name: syn::Expr,
	expr: syn::Expr,
	format_args: Option<FormatArgs>,
}

impl syn::parse::Parse for Args {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let macro_name = input.parse()?;
		let _comma: syn::token::Comma = input.parse()?;
		let expr = input.parse()?;
		let format_args = if input.is_empty() {
			FormatArgs::new()
		} else {
			input.parse::<syn::token::Comma>()?;
			FormatArgs::parse_terminated(input)?
		};

		let format_args = Some(format_args).filter(|x| !x.is_empty());
		Ok(Self {
			macro_name,
			expr,
			format_args,
		})
	}
}
