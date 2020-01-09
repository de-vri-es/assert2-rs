#![feature(proc_macro_span)]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

/// Check if an expression evaluates to true.
///
/// If it does not, an assertion failure is printend,
/// but any remaining code in the same scope will still execute.
/// When the scope ends, the test will panick.
///
/// Use [`assert!`](macro.assert.html) if you want the test to panic instantly.
#[proc_macro]
pub fn check(tokens: TokenStream) -> TokenStream {
	match check_impl(syn::parse_macro_input!(tokens), false) {
		Ok(x) => x.into(),
		Err(e) => e.to_compile_error().into(),
	}
}

/// Assert that an expression evaluates to true.
///
/// If it does not, an assertion failure is printed and the test panics instantly.
///
/// Use [`check!`](macro.check.html) if you still want further checks to be executed.
#[proc_macro]
pub fn assert(tokens: TokenStream) -> TokenStream {
	match check_impl(syn::parse_macro_input!(tokens), true) {
		Ok(x) => x.into(),
		Err(e) => e.to_compile_error().into(),
	}
}

fn check_impl(expression: syn::Expr, instant_panic: bool) -> syn::Result<proc_macro2::TokenStream> {
	match expression {
		syn::Expr::Binary(expr) => check_binary_op(expr, instant_panic),
		expr => check_bool_expr(expr, instant_panic),
	}
}

fn check_binary_op(expr: syn::ExprBinary, instant_panic: bool) -> syn::Result<proc_macro2::TokenStream> {
	let syn::ExprBinary { left, right, op, .. } = &expr;
	let left_str = spanned_to_string(&left);
	let right_str = spanned_to_string(&right);
	let op_str = spanned_to_string(&op);

	match op {
		syn::BinOp::Eq(_)  => (),
		syn::BinOp::Lt(_)  => (),
		syn::BinOp::Le(_)  => (),
		syn::BinOp::Ne(_)  => (),
		syn::BinOp::Ge(_)  => (),
		syn::BinOp::Gt(_)  => (),
		_ => return check_bool_expr(syn::Expr::Binary(expr), instant_panic),
	}

	if instant_panic {
		Ok(quote! {
			if !(left #op right) {
				::check::print::binary_failure("assert", &left, &right, #op_str, #left_str, #right_str, file!(), line!(), column!());
				panic!("assertion failed");
			}
		})
	} else {
		Ok(quote! {
			let left = #left;
			let right = #right;
			let guard;
			if !(left #op right) {
				::check::print::binary_failure("check", &left, &right, #op_str, #left_str, #right_str, file!(), line!(), column!());
				guard = Some(::check::FailGuard(|| panic!("assertion failed")));
			} else {
				guard = None;
			}
		})
	}
}

fn check_bool_expr(expr: syn::Expr, instant_panic: bool) -> syn::Result<proc_macro2::TokenStream> {
	let expr_str = spanned_to_string(&expr);

	if instant_panic {
		Ok(quote! {
			let value : bool = #expr;
			if !value {
				::check::print::bool_failure("assert", &value, #expr_str, file!(), line!(), column!());
				panic!("assertion failed");
			}
		})
	} else {
		Ok(quote! {
			let value : bool = #expr;
			let guard;
			if !value {
				::check::print::bool_failure("check", &value, #expr_str, file!(), line!(), column!());
				eprintln!();
				guard = Some(::check::FailGuard(|| panic!("assertion failed")));
			} else {
				guard = None;
			}
		})
	}
}

fn spanned_to_string<T: quote::ToTokens + ?Sized>(node: &T) -> String {
	node.span().unwrap().source_text().unwrap_or_else(|| node.to_token_stream().to_string())
}
