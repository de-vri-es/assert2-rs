use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;

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

	// Collect all placeholders and make a non-mut and mut version.
	//
	// The mut version is used for the outer pattern match,
	// while the non-mut version is used as final expression value.
	let placeholders = collect_placeholders(&pattern);
	let non_mut_placeholders = non_mut_placeholders(&placeholders);
	let mut_placeholders = mut_placeholders(&placeholders);

	let mut fragments = Fragments::new();
	let pat_str = tokens_to_string(pattern.to_token_stream(), &mut fragments);

	// Also strip the `mut` from the original pattern.
	//The `mut` would cause warnings that we can't selectively disable.
	let pattern = remove_mut(pattern);

	let expr_str = expression_to_string(&crate_name, expression.to_token_stream(), &mut fragments);
	let custom_msg = match format_args {
		Some(x) => quote!(Some(format_args!(#x))),
		None => quote!(None),
	};

	quote! {
		let (#mut_placeholders) = match #expression {
			#pattern => (#non_mut_placeholders),
			value => {
				#[allow(unused)]
				use #crate_name::maybe_debug::{IsDebug, IsMaybeNotDebug};
				let value = (&&::assert2::maybe_debug::Wrap(&value)).__assert2_maybe_debug().wrap(&value);
				#crate_name::print::FailedCheck {
					macro_name: #macro_name,
					file: file!(),
					line: line!(),
					column: column!(),
					custom_msg: #custom_msg,
					expression: #crate_name::print::MatchExpr {
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

fn collect_placeholders(pat: &syn::Pat) -> Vec<syn::PatIdent> {
	#[derive(Default)]
	struct CollectPlaceholders {
		placeholders: Vec<syn::PatIdent>,
	}

	impl<'a> syn::visit::Visit<'a> for CollectPlaceholders {
		fn visit_pat_ident(&mut self, pat_ident: &'a syn::PatIdent) {
			self.placeholders.push(pat_ident.clone());
		}
	}

	use syn::visit::Visit;
	let mut collector = CollectPlaceholders::default();
	collector.visit_pat(pat);
	collector.placeholders.into_iter().collect()
}

/// Remove `mut` from all identifiers in a pattern.
///
/// Consumes the input and returns a modified pattern.
fn remove_mut(mut pat: syn::Pat) -> syn::Pat {
	use syn::visit_mut::VisitMut;

	struct RemoveMutInplace;

	impl VisitMut for RemoveMutInplace {
		fn visit_pat_ident_mut(&mut self, pat_ident: &mut syn::PatIdent) {
			pat_ident.mutability = None;
		}
	}

	RemoveMutInplace.visit_pat_mut(&mut pat);
	pat
}

/// Get all placeholders as tuple without `mut` removed.
fn non_mut_placeholders(placeholders: &[syn::PatIdent]) -> Punctuated<syn::Ident, syn::token::Comma> {
	placeholders.iter().map(|x| x.ident.clone()).collect()
}

/// Get all placeholders as tuple with the original `mut` intact.
fn mut_placeholders(placeholders: &[syn::PatIdent]) -> Punctuated<syn::PatIdent, syn::token::Comma> {
	placeholders.iter().map(|x| syn::PatIdent {
		attrs: vec![],
		by_ref: None, // ref was already added by the original match
		mutability: x.mutability,
		ident: x.ident.clone(),
		subpat: None, // sub-pattern was already applied by the original match
	}).collect()
}

impl syn::parse::Parse for Args {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let crate_name = input.parse()?;
		let _comma = input.parse::<syn::token::Comma>()?;
		let macro_name = input.parse()?;
		let _comma = input.parse::<syn::token::Comma>()?;
		let pattern = input.parse()?;
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
