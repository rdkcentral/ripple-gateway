
//extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;
use proc_macro::{self, TokenStream};
//use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Nothing, Result};
use syn::{parse_quote, FnArg, ItemFn, PatType, ReturnType,DeriveInput,parse_macro_input};

#[proc_macro_derive(RippleClientTMT)]
pub fn ripple_extension_client_send(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let gen = quote! {
        
        use ripple_sdk::extn::extn_client_message::ExtnMessage;
        use ripple_sdk::utils::error::RippleError;
       
        #[ripple_sdk::async_trait::async_trait]
        impl crate::service::extn::ripple_client::RippleClientTMT for #ident {
            async fn send_extn_request(&self,  payload: impl ripple_sdk::extn::extn_client_message::ExtnPayloadProvider) -> Result<ripple_sdk::extn::extn_client_message::ExtnMessage, ripple_sdk::utils::error::RippleError> {
                self.state.get_client().send_extn_request(payload).await
            }            
        }
    };

    gen.into()
}


#[proc_macro_derive(Observable)]
pub fn ripple_extension_client_observable(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let gen = quote! {
  
        #[ripple_sdk::async_trait::async_trait]
        impl crate::service::extn::ripple_client::Observable for #ident {
            async fn send_timer(&self, platform_state: &crate::state::platform_state::PlatformState, timer: ripple_sdk::api::firebolt::fb_metrics::Timer ) ->  () {

            }
            async fn send_counter(&self, platform_state: &crate::state::platform_state::PlatformState, counter: ripple_sdk::api::firebolt::fb_metrics::Counter ) -> () {

            }
        }
    };

    gen.into()
}

#[proc_macro_derive(TestTrait)]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
        impl crate::service::extn::ripple_client::TestTrait for #ident {
         
        }
    };
    print!("{}",output);
    output.into()
}
