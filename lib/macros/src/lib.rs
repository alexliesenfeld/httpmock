extern crate proc_macro;

use proc_macro::*;

use quote::quote;
use syn::{parse_macro_input, parse_quote, ItemFn};

/// This attribute macro must be applied to test functions that require a running mock server.
/// This macro will wrap the actual test function and perform some initialization tasks at the
/// mock server before the test starts (such as removing all existing mocks from the server so the
/// test finds a clean environment when it starts).
#[proc_macro_attribute]
pub fn with_mock_server(args: TokenStream, function: TokenStream) -> TokenStream {
    assert!(args.is_empty());

    let mut function = parse_macro_input!(function as ItemFn);
    let block = function.block;

    function.block = Box::new(parse_quote!({
        let mut server = httpmock::internal_server_management_lock();
        let keep_mocks = option_env!("HTTPMOCK_KEEP_MOCKS");

        httpmock::util::with_retry(10, 1000, || {
            if !keep_mocks.is_some() {
                server.delete_all_mocks()
            } else {
                Ok(())
            }
        }).expect("Cannot initialize mock server");

        httpmock::internal_thread_local_test_init_status(true);

        let test_result = std::panic::catch_unwind(move || {
            #block
        });

        httpmock::internal_thread_local_test_init_status(false);

        match std::panic::catch_unwind(move || {
            if !keep_mocks.is_some() {
                match server.delete_all_mocks() {
                    _ => {}
                }
            }
        }) {
            _ => {}
        }

        test_result.unwrap();
    }));

    TokenStream::from(quote!(#function))
}
