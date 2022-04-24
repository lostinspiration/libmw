//! Provides macro support for `libmw` crate
#![deny(missing_docs, unsafe_code)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Implement `PipelineContext` trait
#[proc_macro_derive(PipelineContext)]
pub fn pipeline_context_impl(tokens: TokenStream) -> TokenStream {
	let input = parse_macro_input!(tokens as DeriveInput);
	let name = input.ident;

	let modified = quote! {
		impl PipelineContext for #name {
			fn as_any_mut(&mut self) -> &mut (dyn std::any::Any + 'static) {
				self
			}
	
			fn as_any(&self) -> &(dyn std::any::Any + 'static) {
				self
			}
		}
	};

	TokenStream::from(modified)
}
