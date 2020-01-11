#![feature(proc_macro_span)]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

type FormatArgs = Punctuated<syn::Expr, syn::token::Comma>;

#[proc_macro_hack]
pub fn assert(tokens: TokenStream) -> TokenStream {
	match check_or_assert_impl(syn::parse_macro_input!(tokens), true) {
		Ok(x) => x.into(),
		Err(e) => e.to_compile_error().into(),
	}
}

#[proc_macro_hack]
pub fn check_impl(tokens: TokenStream) -> TokenStream {
	match check_or_assert_impl(syn::parse_macro_input!(tokens), false) {
		Ok(x) => x.into(),
		Err(e) => e.to_compile_error().into(),
	}
}

/// Real implementation for assert!() and check!().
fn check_or_assert_impl(args: Args, instant_panic: bool) -> syn::Result<proc_macro2::TokenStream> {
	match args.expr {
		syn::Expr::Binary(expr) => check_binary_op(expr, args.format_args, instant_panic),
		syn::Expr::Let(expr) => check_let_expr(expr, args.format_args, instant_panic),
		expr => check_bool_expr(expr, args.format_args, instant_panic),
	}
}

fn check_binary_op(expr: syn::ExprBinary, format_args: FormatArgs, instant_panic: bool) -> syn::Result<proc_macro2::TokenStream> {
	match expr.op {
		syn::BinOp::Eq(_)  => (),
		syn::BinOp::Lt(_)  => (),
		syn::BinOp::Le(_)  => (),
		syn::BinOp::Ne(_)  => (),
		syn::BinOp::Ge(_)  => (),
		syn::BinOp::Gt(_)  => (),
		_ => return check_bool_expr(syn::Expr::Binary(expr), format_args, instant_panic),
	};

	let syn::ExprBinary { left, right, op, .. } = &expr;
	let left_str = spanned_to_string(&left);
	let right_str = spanned_to_string(&right);
	let op_str = spanned_to_string(&op);
	let extra_print = extra_print(format_args);

	if instant_panic {
		Ok(quote! {
			{
				let left = #left;
				let right = #right;
				if !(left #op right) {
					::assert2::print::binary_failure("assert", &left, &right, #op_str, #left_str, #right_str, file!(), line!(), column!());
					#extra_print
					panic!("assertion failed");
				}
			}
		})
	} else {
		Ok(quote! {
			{
				let left = #left;
				let right = #right;
				if !(left #op right) {
					::assert2::print::binary_failure("check", &left, &right, #op_str, #left_str, #right_str, file!(), line!(), column!());
					#extra_print
					Some(::assert2::FailGuard(|| panic!("assertion failed")))
				} else {
					None
				}
			}
		})
	}
}

fn check_bool_expr(expr: syn::Expr, format_args: FormatArgs, instant_panic: bool) -> syn::Result<proc_macro2::TokenStream> {
	let expr_str = spanned_to_string(&expr);
	let extra_print = extra_print(format_args);

	if instant_panic {
		Ok(quote! {
			{
				let value: bool = #expr;
				if !value {
					::assert2::print::bool_failure("assert", &value, #expr_str, file!(), line!(), column!());
					#extra_print
					panic!("assertion failed");
				}
			}
		})
	} else {
		Ok(quote! {
			{
				let value: bool = #expr;
				if !value {
					::assert2::print::bool_failure("check", &value, #expr_str, file!(), line!(), column!());
					#extra_print
					Some(::assert2::FailGuard(|| panic!("assertion failed")))
				} else {
					None
				}
			}
		})
	}
}

fn check_let_expr(expr: syn::ExprLet, format_args: FormatArgs, instant_panic: bool) -> syn::Result<proc_macro2::TokenStream> {
	let syn::ExprLet { pat, expr, let_token, eq_token, .. } = expr;

	let pat_str = spanned_to_string(&pat);
	let expr_str = spanned_to_string(&expr);
	let extra_print = extra_print(format_args);

	if instant_panic {
		Ok(quote! {
			{
				let value = #expr;
				if #let_token #pat #eq_token &value {
					// Nothing to do here.
				} else {
					::assert2::print::match_failure("assert", &value, #pat_str, #expr_str, file!(), line!(), column!());
					#extra_print
					panic!("assertion failed");
				}
			}
		})
	} else {
		Ok(quote! {
			{
				let value = #expr;
				if #let_token #pat #eq_token &value {
					None
				} else {
					::assert2::print::match_failure("check", &value, #pat_str, #expr_str, file!(), line!(), column!());
					#extra_print
					Some(::assert2::FailGuard(|| panic!("assertion failed")))
				}
			}
		})
	}
}

fn spanned_to_string<T: quote::ToTokens + ?Sized>(node: &T) -> String {
	node.span().unwrap().source_text().unwrap_or_else(|| node.to_token_stream().to_string())
}

struct Args {
	expr: syn::Expr,
	format_args: FormatArgs,
}

impl syn::parse::Parse for Args {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let expr = input.parse()?;
		let format_args = if input.is_empty() {
			FormatArgs::new()
		} else {
			input.parse::<syn::token::Comma>()?;
			FormatArgs::parse_terminated(input)?
		};
		Ok(Self { expr, format_args })
	}
}

fn extra_print(format_args: FormatArgs) -> proc_macro2::TokenStream {
	if format_args.is_empty() {
		quote! {
			eprintln!();
		}
	} else {
		quote! {
			::assert2::print::user_message_prefix();
			eprintln!(#format_args);
			eprintln!();
		}
	}
}
