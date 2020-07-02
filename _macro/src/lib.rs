extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream as TokenStream1;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Item, ItemFn, LitStr};

#[proc_macro_attribute]
pub fn webhook_report_error(attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
	error_reporter_impl(attr.into(), item.into()).into()
}

const DEFAULT_ENVAR_NAME: &str = "REPORT_ERRORS_AT";

fn error_reporter_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
	let mut item: ItemFn = match syn::parse2(item) {
		Ok(item) => item,
		Err(err) => panic!("{}", err)
	};

	let webhook_env = if attr.is_empty() {
		DEFAULT_ENVAR_NAME.to_string()
	} else {
		match syn::parse2::<LitStr>(attr) {
			Ok(lit) => lit.value(),
			Err(err) => panic!("{}", err)
		}
	};

	let method_code = item.block;

	let handler: TokenStream = (quote! {{
		return match (async move || {#method_code} )().await {
			Ok(x) => Ok(x),
			Err(error) => {
				if let Ok(hook) = std::env::var(#webhook_env) {
				    use ::webhook_error_reporter::core::report;

					Err(report(error, &hook).await)
				} else {
					panic!("failed to find the report url from envar: {}", #webhook_env);
				}
			}
		};
	}}).into();

	item.block = syn::parse2(handler).expect("handler code gen failed");

	let item: Item = item.into();
	item.into_token_stream()
}