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
            let mut server_guard = match mocha::SERVER_MUTEX.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };

            mocha::TEST_INITIALIZED.with(|is_init| {
                *is_init.borrow_mut() = true
            });

            let test_result = std::panic::catch_unwind(move || {
                #block
            });

            mocha::TEST_INITIALIZED.with(|is_init| {
                *is_init.borrow_mut() = false
            });

            test_result.unwrap();
        }));

    TokenStream::from(quote!(#function))
}
