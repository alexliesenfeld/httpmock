extern crate proc_macro;

use proc_macro::*;

use quote::quote;
use syn::{parse_macro_input, parse_quote, ItemFn};

#[proc_macro_attribute]
pub fn test_executors(args: TokenStream, function: TokenStream) -> TokenStream {
    assert!(args.is_empty());

    let mut function = parse_macro_input!(function as ItemFn);
    let block = function.block;

    function.block = Box::new(parse_quote!({
        // Blocking
        // println!("Running without an executor");
        #block

        // With Tokio Runtime
        // println!("Running with Tokio executor");
        let mut trt = tokio::runtime::Runtime::new().unwrap();
        trt.block_on(async {
            #block
        });

         // With actix Runtime
        // println!("Running with actix Runtime");
        let mut art = actix_rt::Runtime::new().unwrap();
        art.block_on(async {
            #block
        });

        // With async_std executor
        // println!("Running with std_async executor");
        async_std::task::block_on(async {
            #block
        })

    }));

    TokenStream::from(quote!(#function))
}
