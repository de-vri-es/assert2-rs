use proc_macro2::TokenStream;
use quote::{quote, ToTokens as _};

use crate::{Args, Context};

/// Real implementation for check!().
///
/// Check can not capture placeholders from patterns in the outer scope,
/// since it continues even if the check fails.
///
/// But it does support capturing and additional testing with `&&` chains.
pub(crate) fn check(args: Args) -> TokenStream {
	let context = args.into_context();

	let mut assertions = quote! { ::core::result::Result::Ok::<(), ()>(()) };
	for (i, (_glue, expr)) in context.predicates.iter().enumerate().rev() {
		assertions = match expr {
			syn::Expr::Binary(expr) => check_binary_op(&context, i, expr, assertions),
			syn::Expr::Let(expr) => check_let_expr(&context, i, expr, assertions),
			expr => check_bool_expr(&context, i , expr, assertions),
		};
	}

	assertions
}

pub(crate) fn check_binary_op(
	context: &Context,
	index: usize,
	expr: &syn::ExprBinary,
	next_predicate: TokenStream,
) -> TokenStream {
	match expr.op {
		syn::BinOp::Eq(_) => (),
		syn::BinOp::Lt(_) => (),
		syn::BinOp::Le(_) => (),
		syn::BinOp::Ne(_) => (),
		syn::BinOp::Ge(_) => (),
		syn::BinOp::Gt(_) => (),
		_ => return check_bool_expr(context, index, expr, next_predicate),
	};

	let syn::ExprBinary { left, right, op, .. } = &expr;
	let op_str = op.to_token_stream().to_string();

	let Context {
		crate_name,
		macro_name,
		predicates: _,
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
					expansion: #crate_name::__assert2_impl::print::Expansion::Binary {
						left: (&left as &dyn ::core::fmt::Debug),
						right: (&right as &dyn ::core::fmt::Debug),
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

pub(crate) fn check_bool_expr(
	context: &Context,
	index: usize,
	expr: &impl quote::ToTokens,
	next_predicate: TokenStream,
) -> TokenStream {
	let Context {
		crate_name,
		macro_name,
		predicates: _,
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

pub(crate) fn check_let_expr(
	context: &Context,
	index: usize,
	expr: &syn::ExprLet,
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
		predicates: _,
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
						expression: &value as &dyn ::core::fmt::Debug,
					},
					fragments: #fragments,
					custom_msg: #custom_msg,
				}.print();
				::core::result::Result::Err(())
			}
		}
	}
}
