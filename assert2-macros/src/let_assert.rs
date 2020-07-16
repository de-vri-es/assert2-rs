use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;

use crate::expression_to_string;
use crate::tokens_to_string;
use crate::FormatArgs;
use crate::Fragments;

pub struct Args {
	macro_name: syn::Expr,
	pattern: syn::Pat,
	eq_token: syn::token::Eq,
	expression: syn::Expr,
	format_args: Option<FormatArgs>,
}

pub fn let_assert_impl(args: Args) -> TokenStream {
	let Args {
		macro_name,
		pattern,
		eq_token,
		expression,
		format_args,
	} = args;

	let placeholders = collect_placeholders(&pattern);

	let mut fragments = Fragments::new();
	let pat_str = tokens_to_string(pattern.to_token_stream(), &mut fragments);
	let expr_str = expression_to_string(expression.to_token_stream(), &mut fragments);
	let custom_msg = match format_args {
		Some(x) => quote!(Some(format_args!(#x))),
		None => quote!(None),
	};

	quote! {
		let (#placeholders) = {
			let value = #expression;
			if let #pattern #eq_token value {
				(#placeholders)
			} else {
				#[allow(unused)]
				use ::assert2::maybe_debug::{IsDebug, IsMaybeNotDebug};
				let value = (&&::assert2::maybe_debug::Wrap(&value)).__assert2_maybe_debug().wrap(&value);
				::assert2::print::FailedCheck {
					macro_name: #macro_name,
					file: file!(),
					line: line!(),
					column: column!(),
					custom_msg: #custom_msg,
					expression: ::assert2::print::MatchExpr {
						print_let: false,
						value: &value,
						pattern: #pat_str,
						expression: #expr_str,
					},
					fragments: #fragments,
				}.print();
				panic!("assertion failed");
			}
		};
	}
}

fn collect_placeholders(pat: &syn::Pat) -> Punctuated<syn::Ident, syn::token::Comma> {
	#[derive(Default)]
	struct CollectPlaceholders {
		placeholders: Vec<syn::Ident>,
	}

	impl<'a> syn::visit::Visit<'a> for CollectPlaceholders {
		fn visit_pat_ident(&mut self, pat_ident: &'a syn::PatIdent) {
			self.placeholders.push(pat_ident.ident.clone());
		}
	}

	use syn::visit::Visit;
	let mut collector = CollectPlaceholders::default();
	collector.visit_pat(pat);
	collector.placeholders.into_iter().collect()
}

impl syn::parse::Parse for Args {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let macro_name = input.parse()?;
		let _comma = input.parse::<syn::token::Comma>()?;
		let pattern = input.parse()?;
		let eq_token = input.parse()?;
		let expression = input.parse()?;

		let format_args = if input.is_empty() {
			FormatArgs::new()
		} else {
			input.parse::<syn::token::Comma>()?;
			FormatArgs::parse_terminated(input)?
		};
		let format_args = Some(format_args).filter(|x| !x.is_empty());

		Ok(Self {
			macro_name,
			pattern,
			eq_token,
			expression,
			format_args,
		})
	}
}
