use proc_macro::TokenStream;

use crate::extend::derive;

pub(crate) mod extend;

#[proc_macro_derive(Remap, attributes(remap))]
pub fn derive_remap(input: TokenStream) -> TokenStream {
    derive(syn::parse_macro_input!(input))
}