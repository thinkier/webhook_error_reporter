extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream as TokenStream1;
use std::backtrace::Backtrace;
use std::error::Error;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Item, ItemFn, LitStr, ReturnType};

#[proc_macro_attribute]
pub fn webhook_report_error(attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
    error_reporter_impl(attr.into(), item.into()).into()
}

const DEFAULT_ENVAR_NAME: &str = "ERROR_WEBHOOK";

pub enum ReportedError<E> {
    Reported(E),
    Unreported(ReporterError, E),
}

impl<E: Error> Error for ReportedError<E> {
    #[cfg(feature = "backtrace")]
    fn backtrace(&self) -> Option<&Backtrace> {
        match self {
            Self::Reported(e) | Self::Unreported(_, e) => {
                e.backtrace()
            }
        }

        None
    }

    fn cause(&self) -> Option<&dyn Error> {}
}

pub struct ReporterError {}

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
		match {#method_code} {
			Err(error) => {
				let webhook = if let Ok(hook) = std::env::var(#webhook_env) {
					hook
				} else {
					panic!("failed to find the report url from envar: {}", #webhook_env);
				};
				Err(error)
			}
			ok => ok
		}
	}}).into();

    item.block = syn::parse2(handler).unwrap();
    if let ReturnType::Type(rarr, ty) = &mut item.sig.output {}

    let item: Item = item.into();
    item.into_token_stream()
}