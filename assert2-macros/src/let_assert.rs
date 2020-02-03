use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;

use crate::spanned_to_string;
use crate::FormatArgs;

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

	let mut use_placeholders = TokenStream::new();
	for placeholder in &placeholders {
		use_placeholders.extend(quote! {
			&#placeholder,
		});
	}
	let use_placeholders = quote! {
		let _ = (#use_placeholders);
	};

	let placeholders = puncuate_idents(placeholders);

	let pat_str = spanned_to_string(&pattern);
	let expr_str = spanned_to_string(&expression);
	let custom_msg = match format_args {
		Some(x) => quote!(Some(format_args!(#x))),
		None => quote!(None),
	};

	quote! {
		let (#placeholders) = {
			let value = #expression;
			if let #pattern #eq_token &value {
				#use_placeholders
				if let #pattern #eq_token value {
					(#placeholders)
				} else {
					panic!("{}: second pattern match failed, please report this at https://github.com/de-vri-es/assert2-rs/issues/new/");
				}
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
						value: &value,
						pattern: #pat_str,
						expression: #expr_str,
					}
				}.print();
				panic!("assertion failed");
			}
		};
	}
}

struct CollectPlaceholders<'a> {
	placeholders: &'a mut Vec<syn::Ident>,
}

impl<'a> CollectPlaceholders<'a> {
	fn new(placeholders: &'a mut Vec<syn::Ident>) -> Self {
		Self { placeholders }
	}
}

impl<'a> syn::visit::Visit<'a> for CollectPlaceholders<'_> {
	fn visit_pat_ident(&mut self, pat_ident: &'a syn::PatIdent) {
		self.placeholders.push(pat_ident.ident.clone());
	}
}

fn collect_placeholders(pat: &syn::Pat) -> Vec<syn::Ident> {
	use syn::visit::Visit;
	let mut placeholders = Vec::new();
	let mut collector = CollectPlaceholders::new(&mut placeholders);
	collector.visit_pat(pat);
	placeholders
}

fn puncuate_idents(input: impl IntoIterator<Item = syn::Ident>) -> Punctuated<syn::Ident, syn::token::Comma> {
	let mut punctuated: Punctuated<syn::Ident, syn::token::Comma> = input.into_iter().collect();
	if !punctuated.is_empty() {
		punctuated.push_punct(syn::token::Comma::default());
	}
	punctuated
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