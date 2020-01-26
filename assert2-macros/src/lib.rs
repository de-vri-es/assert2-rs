#![cfg_attr(nightly, feature(proc_macro_span))]

//! This macro contains only private procedural macros.
//! See the documentation for [`assert2`](https://docs.rs/assert2/) for the public API.

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;
use quote::quote;
use syn::punctuated::Punctuated;

type FormatArgs = Punctuated<syn::Expr, syn::token::Comma>;

#[proc_macro_hack]
#[doc(hidden)]
pub fn check_impl(tokens: TokenStream) -> TokenStream {
	check_or_assert_impl(syn::parse_macro_input!(tokens)).into()
}

#[cfg(feature = "let-assert")]
mod let_assert;

#[cfg(feature = "let-assert")]
#[proc_macro]
#[doc(hidden)]
pub fn let_assert_impl(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let_assert::let_assert_impl(syn::parse_macro_input!(tokens)).into()
}

/// Real implementation for assert!() and check!().
fn check_or_assert_impl(args: Args) -> proc_macro2::TokenStream {
	match args.expr {
		syn::Expr::Binary(expr) => check_binary_op(args.macro_name, expr, args.format_args),
		syn::Expr::Let(expr) => check_let_expr(args.macro_name, expr, args.format_args),
		expr => check_bool_expr(args.macro_name, expr, args.format_args),
	}
}

fn check_binary_op(macro_name: syn::Expr, expr: syn::ExprBinary, format_args: Option<FormatArgs>) -> proc_macro2::TokenStream {
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
	let left_expr = spanned_to_string(&left);
	let right_expr = spanned_to_string(&right);
	let op_str = spanned_to_string(&op);

	let custom_msg = match format_args {
		Some(x) => quote!(Some(format_args!(#x))),
		None => quote!(None),
	};

	quote! {
		{
			let left = #left;
			let right = #right;
			if !(left #op right) {
				use ::assert2::maybe_debug::{IsDebug, IsMaybeNotDebug};
				let left = (&&::assert2::maybe_debug::Wrap(&left)).__assert2_maybe_debug().wrap(&left);
				let right = (&& ::assert2::maybe_debug::Wrap(&right)).__assert2_maybe_debug().wrap(&right);
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
					}
				}.print();
				Err(())
			} else {
				Ok(())
			}
		}
	}
}

fn check_bool_expr(macro_name: syn::Expr, expr: syn::Expr, format_args: Option<FormatArgs>) -> proc_macro2::TokenStream {
	let expr_str = spanned_to_string(&expr);

	let custom_msg = match format_args {
		Some(x) => quote!(Some(format_args!(#x))),
		None => quote!(None),
	};

	quote! {
		{
			let value: bool = #expr;
			if !value {
				::assert2::print::FailedCheck {
					macro_name: #macro_name,
					file: file!(),
					line: line!(),
					column: column!(),
					custom_msg: #custom_msg,
					expression: ::assert2::print::BooleanExpr {
						expression: #expr_str,
					}
				}.print();
				Err(())
			} else {
				Ok(())
			}
		}
	}
}

fn check_let_expr(macro_name: syn::Expr, expr: syn::ExprLet, format_args: Option<FormatArgs>) -> proc_macro2::TokenStream {
	let syn::ExprLet {
		pat,
		expr,
		let_token,
		eq_token,
		..
	} = expr;

	let pat_str = spanned_to_string(&pat);
	let expr_str = spanned_to_string(&expr);

	let custom_msg = match format_args {
		Some(x) => quote!(Some(format_args!(#x))),
		None => quote!(None),
	};

	quote! {
		{
			let value = #expr;
			if #let_token #pat #eq_token &value {
				Ok(())
			} else {
				use ::assert2::maybe_debug::{IsDebug, IsMaybeNotDebug};
				let value = (&&::assert2::maybe_debug::Wrap(&value)).__assert2_maybe_debug().wrap(&value);
				::assert2::print::FailedCheck {
					macro_name: #macro_name,
					file: file!(),
					line: line!(),
					column: column!(),
					custom_msg: #custom_msg,
					expression: ::assert2::print::MatchExpr {
						value: &value,
						pattern: #pat_str,
						expression: #expr_str,
					}
				}.print();
				Err(())
			}
		}
	}
}

fn spanned_to_string<T: quote::ToTokens + ?Sized>(node: &T) -> String {
	#[cfg(nightly)]
	{
		use syn::spanned::Spanned;
		if let Some(s) = node.span().unwrap().source_text() {
			return s;
		}
	}
	node.to_token_stream().to_string()
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
