use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote, quote_spanned};

use crate::expression_to_string;
use crate::tokens_to_string;
use crate::FormatArgs;
use crate::Fragments;

pub struct Args {
	crate_name: syn::Path,
	macro_name: syn::Expr,
	pattern: syn::Pat,
	expression: syn::Expr,
	format_args: Option<FormatArgs>,
}

pub fn let_assert_impl(args: Args) -> TokenStream {
	let Args {
		crate_name,
		macro_name,
		pattern,
		expression,
		format_args,
	} = args;

	let mut fragments = Fragments::new();
	let pat_str = tokens_to_string(pattern.to_token_stream(), &mut fragments);

	let expr_str = expression_to_string(&crate_name, expression.to_token_stream(), &mut fragments);
	let custom_msg = match format_args {
		Some(x) => quote!(Some(format_args!(#x))),
		None => quote!(None),
	};

	let value = quote_spanned!{ Span::mixed_site() => value };

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
				expression: #crate_name::__assert2_impl::print::MatchExpr {
					print_let: false,
					value: &value,
					pattern: #pat_str,
					expression: #expr_str,
				},
				fragments: #fragments,
			}.print();
			panic!("assertion failed");
		};
	}
}

impl syn::parse::Parse for Args {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let crate_name = input.parse()?;
		let _comma = input.parse::<syn::token::Comma>()?;
		let macro_name = input.parse()?;
		let _comma = input.parse::<syn::token::Comma>()?;
		let pattern =  syn::Pat::parse_multi_with_leading_vert(input)?;
		let _eq_token = input.parse::<syn::token::Eq>()?;
		let expression = input.parse()?;

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
			pattern,
			expression,
			format_args,
		})
	}
}
