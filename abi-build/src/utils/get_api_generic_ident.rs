use proc_macro2::TokenStream;
use quote::{format_ident, ToTokens};

pub(crate) fn get_api_generic_ident() -> TokenStream {
    let ident = format_ident!("{}", "StaticApi");
    ident.into_token_stream()
}