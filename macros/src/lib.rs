use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod getter;
mod utils;

use getter::derive_getter_impl;

/// ゲッター導出マクロ
#[proc_macro_derive(Getter, attributes(getter, ret_type, ret_ref))]
pub fn derive_getter(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match derive_getter_impl(input) {
        Ok(token_stream) => TokenStream::from(token_stream),
        Err(err) => TokenStream::from(err.into_compile_error()),
    }
}
