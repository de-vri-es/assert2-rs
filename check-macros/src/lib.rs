#![feature(proc_macro_span)]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

#[proc_macro]
pub fn check(tokens: TokenStream) -> TokenStream {
	match check_impl(syn::parse_macro_input!(tokens)) {
		Ok(x) => x.into(),
		Err(e) => e.to_compile_error().into(),
	}
}

fn check_impl(expression: syn::Expr) -> syn::Result<proc_macro2::TokenStream> {
	match expression {
		syn::Expr::Binary(expr) => check_binary_op(expr),
		expr => check_bool_expr(expr),
	}
}

fn check_binary_op(expr: syn::ExprBinary) -> syn::Result<proc_macro2::TokenStream> {
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
		_ => return check_bool_expr(syn::Expr::Binary(expr)),
	}

	Ok(quote! {
		let left = #left;
		let right = #right;
		if !(left #op right) {
			::check::print::binary_failure(&left, &right, #op_str, #left_str, #right_str, file!(), line!(), column!());
			eprintln!();
			panic!("assertion failed");
		}
	})
}

fn check_bool_expr(expr: syn::Expr) -> syn::Result<proc_macro2::TokenStream> {
	let expr_str = spanned_to_string(&expr);

	Ok(quote! {
		let value : bool = #expr;
		if !value {
			::check::print::bool_failure(&value, #expr_str, file!(), line!(), column!());
			eprintln!();
			panic!("assertion failed");
		}
	})
}

fn spanned_to_string<T: quote::ToTokens + ?Sized>(node: &T) -> String {
	node.span().unwrap().source_text().unwrap_or_else(|| node.to_token_stream().to_string())
}
