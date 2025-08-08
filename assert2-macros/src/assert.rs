use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};

use crate::Args;
use crate::Context;
use crate::check::{check_binary_op, check_bool_expr};

pub(crate) fn assert(args: Args) -> TokenStream {
	let context = args.into_context();

	let mut output = TokenStream::new();
	for (i, (_glue, predicate)) in context.predicates.iter().enumerate() {
		let assertion = match predicate {
			syn::Expr::Let(expr) => assert_let_expr(&context, i, expr),
			syn::Expr::Binary(expr) => assert_binary_expr(&context, i, expr),
			expr => assert_bool_expr(&context, i, expr),
		};
		output.extend(assertion);
	}
	output
}

fn assert_binary_expr(
	context: &super::Context,
	index: usize,
	expr: &syn::ExprBinary,
) -> TokenStream {
	let check = check_binary_op(context, index, expr, quote! { ::core::result::Result::Ok::<(), ()>(()) });
	quote! {
		match #check {
			::core::result::Result::Ok(()) => (),
			::core::result::Result::Err(()) => ::core::panic!("assertion failed"),
		}
	}
}

fn assert_bool_expr(
	context: &super::Context,
	index: usize,
	expr: &syn::Expr,
) -> TokenStream {
	let check = check_bool_expr(context, index, expr, quote! { ::core::result::Result::Ok::<(), ()>(()) });
	quote! {
		match #check {
			::core::result::Result::Ok(()) => (),
			::core::result::Result::Err(()) => ::core::panic!("assertion failed"),
		}
	}
}

fn assert_let_expr(
	context: &super::Context,
	index: usize,
	let_expr: &syn::ExprLet,
) -> TokenStream {
	let pattern = &*let_expr.pat;
	let expression = &*let_expr.expr;
	let value = quote_spanned!{ Span::mixed_site() => value };

	let Context {
		crate_name,
		macro_name,
		predicates: _,
		print_predicates,
		fragments,
		custom_msg,
	} = context;

	quote! {
		let #value = #expression;
		let #pattern = #value else {
			#[allow(unused)]
			use #crate_name::__assert2_impl::maybe_debug::{IsDebug, IsMaybeNotDebug};
			let value = (&&#crate_name::__assert2_impl::maybe_debug::Wrap(&#value)).__assert2_maybe_debug().wrap(&#value);
			#crate_name::__assert2_impl::print::FailedCheck {
				macro_name: #macro_name,
				file: file!(),
				line: line!(),
				column: column!(),
				custom_msg: #custom_msg,
				predicates: #print_predicates,
				failed: #index,
				expansion: #crate_name::__assert2_impl::print::Expansion::Let {
					expression: &value as &dyn ::core::fmt::Debug,
				},
				fragments: #fragments,
			}.print();
			::core::panic!("assertion failed");
		};
	}
}
