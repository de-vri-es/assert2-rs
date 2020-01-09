#![feature(proc_macro_span)]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

/// Assert that an expression evaluates to true or matches a pattern.
///
/// Use a `let` expression to test an expression against a pattern: `assert!(let pattern = expr)`.
/// For other tests, just give a boolean expression to the macro: `assert!(1 + 2 == 2)`.
///
/// If the expression evaluates to false or if the pattern doesn't match,
/// an assertion failure is printed and the macro panics instantly.
///
/// Use [`check!`](macro.check.html) if you still want further checks to be executed.
#[proc_macro]
pub fn assert(tokens: TokenStream) -> TokenStream {
	match check_impl(syn::parse_macro_input!(tokens), true) {
		Ok(x) => x.into(),
		Err(e) => e.to_compile_error().into(),
	}
}

/// Check if an expression evaluates to true or matches a pattern.
///
/// Use a `let` expression to test an expression against a pattern: `check!(let pattern = expr)`.
/// For other tests, just give a boolean expression to the macro: `check!(1 + 2 == 2)`.
///
/// If the expression evaluates to false or if the pattern doesn't match,
/// an assertion failure is printed but the macro does not panic immediately.
/// The check macro will cause the running test to fail eventually.
///
/// Use [`assert!`](macro.assert.html) if you want the test to panic instantly.
///
/// Currently, this macro uses a scope guard to delay the panic.
/// However, this may change in the future if there is a way to signal a test failure without panicking.
/// **Do not rely on `check!()` to panic**.
#[proc_macro]
pub fn check(tokens: TokenStream) -> TokenStream {
	match check_impl(syn::parse_macro_input!(tokens), false) {
		Ok(x) => x.into(),
		Err(e) => e.to_compile_error().into(),
	}
}

fn check_impl(expression: syn::Expr, instant_panic: bool) -> syn::Result<proc_macro2::TokenStream> {
	match expression {
		syn::Expr::Binary(expr) => check_binary_op(expr, instant_panic),
		syn::Expr::Let(expr) => check_let_expr(expr, instant_panic),
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
				::assert2::print::binary_failure("assert", &left, &right, #op_str, #left_str, #right_str, file!(), line!(), column!());
				panic!("assertion failed");
			}
		})
	} else {
		Ok(quote! {
			let left = #left;
			let right = #right;
			let guard;
			if !(left #op right) {
				::assert2::print::binary_failure("check", &left, &right, #op_str, #left_str, #right_str, file!(), line!(), column!());
				guard = Some(::assert2::FailGuard(|| panic!("assertion failed")));
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
				::assert2::print::bool_failure("assert", &value, #expr_str, file!(), line!(), column!());
				panic!("assertion failed");
			}
		})
	} else {
		Ok(quote! {
			let value : bool = #expr;
			let guard;
			if !value {
				::assert2::print::bool_failure("check", &value, #expr_str, file!(), line!(), column!());
				guard = Some(::assert2::FailGuard(|| panic!("assertion failed")));
			} else {
				guard = None;
			}
		})
	}
}

fn check_let_expr(expr: syn::ExprLet, instant_panic: bool) -> syn::Result<proc_macro2::TokenStream> {
	let syn::ExprLet { pat, expr, let_token, eq_token, .. } = expr;

	let pat_str = spanned_to_string(&pat);
	let expr_str = spanned_to_string(&expr);

	if instant_panic {
		Ok(quote! {
			let value = #expr;
			if #let_token #pat #eq_token &value {
				// Nothing to do here.
			} else {
				::assert2::print::match_failure("assert", &value, #pat_str, #expr_str, file!(), line!(), column!());
				panic!("assertion failed");
			}
		})
	} else {
		Ok(quote! {
			let value = #expr;
			let guard;
			if #let_token #pat #eq_token &value {
				guard = None;
			} else {
				::assert2::print::match_failure("check", &value, #pat_str, #expr_str, file!(), line!(), column!());
				guard = Some(::assert2::FailGuard(|| panic!("assertion failed")));
			}
		})
	}
}

fn spanned_to_string<T: quote::ToTokens + ?Sized>(node: &T) -> String {
	node.span().unwrap().source_text().unwrap_or_else(|| node.to_token_stream().to_string())
}
