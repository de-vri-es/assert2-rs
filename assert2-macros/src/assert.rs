use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};

use crate::Context;
use crate::FormatArgs;
use crate::Fragments;

pub struct Args {
	crate_name: syn::Path,
	macro_name: syn::Expr,
	expression: syn::Expr,
	format_args: Option<FormatArgs>,
}

pub fn assert(args: Args) -> TokenStream {
	let custom_msg = match args.format_args {
		Some(x) => quote!(Some(format_args!(#x))),
		None => quote!(None),
	};

	let predicates = super::split_predicates(args.expression);
	let mut fragments = Fragments::new();
	let print_predicates = super::printable_predicates(&args.crate_name, &predicates, &mut fragments);

	let context = Context {
		crate_name: args.crate_name,
		macro_name: args.macro_name,
		print_predicates,
		fragments,
		custom_msg,
	};

	let mut output = TokenStream::new();
	for (i, predicate) in predicates.into_iter().enumerate() {
		let assertion = match predicate {
			syn::Expr::Let(expr) => assert_let_expr(&context, i, expr),
			syn::Expr::Binary(expr) => assert_binary_expr(&context, i, expr),
			expr => assert_bool_expr(&context, i, expr),
		};
		output.extend(assertion);
	}
	output
}

fn assert_let_expr(
	context: &super::Context,
	index: usize,
	let_expr: syn::ExprLet,
) -> TokenStream {
	let pattern = &*let_expr.pat;
	let expression = &*let_expr.expr;
	let value = quote_spanned!{ Span::mixed_site() => value };

	let Context {
		crate_name,
		macro_name,
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
					expression: &value,
				},
				fragments: #fragments,
			}.print();
			panic!("assertion failed");
		};
	}
}

fn assert_binary_expr(
	context: &super::Context,
	index: usize,
	expr: syn::ExprBinary,
) -> TokenStream {
	let check = super::check_binary_op(context, index, expr, quote! { Ok::<(), ()>(()) });
	quote! {
		match #check {
			Ok(()) => (),
			Err(()) => panic!("assertion failed"),
		}
	}
}

fn assert_bool_expr(
	context: &super::Context,
	index: usize,
	expr: syn::Expr,
) -> TokenStream {
	let check = super::check_bool_expr(context, index, expr, quote! { Ok::<(), ()>(()) });
	quote! {
		match #check {
			Ok(()) => (),
			Err(()) => panic!("assertion failed"),
		}
	}
}

impl syn::parse::Parse for Args {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let crate_name = input.parse()?;
		let _comma = input.parse::<syn::token::Comma>()?;
		let macro_name = input.parse()?;
		let _comma = input.parse::<syn::token::Comma>()?;
		let expression = syn::Expr::parse_without_eager_brace(input)?;

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
			expression,
			format_args,
		})
	}
}
