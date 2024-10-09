use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn auth(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = parse_macro_input!(item as ItemFn);
    let stmts = &block.stmts;
    quote! {
        #(#attrs)* #vis #sig {
            if !session.lock().await.authorized {
                err!(message, 401, "UNAUTHORIZED");
            }
            #(#stmts)*
        }
    }
    .into()
}
