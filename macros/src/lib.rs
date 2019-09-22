extern crate proc_macro;

use proc_macro::*;

use quote::quote;
use syn::{parse_macro_input, parse_quote, ItemFn};

#[proc_macro_attribute]
pub fn with_mock_server(args: TokenStream, function: TokenStream) -> TokenStream {
    assert!(args.is_empty());

    let mut function = parse_macro_input!(function as ItemFn);
    let block = function.block;

    function.block = Box::new(parse_quote!({
        mocha::SERVER_GUARD.with(|_| {});
        #block
    }));

    TokenStream::from(quote!(#function))
}