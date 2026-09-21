mod dcc;
mod func;

use proc_macro::TokenStream;
use syn::{DeriveInput, ItemImpl, parse_macro_input};

#[proc_macro_derive(DccOptions, attributes(dcc))]
pub fn derive_dcc_options_data(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match dcc::derive_options_data(input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn derive_fn_data(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemImpl);
    match func::derive_data(input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
