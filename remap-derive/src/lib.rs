use proc_macro::TokenStream;

use crate::extend::derive;
use crate::table::impl_table;

pub(crate) mod table;
pub(crate) mod extend;

#[proc_macro_derive(Table)]
pub fn derive_table(input: TokenStream) -> TokenStream {
    impl_table(input)
}

#[proc_macro_derive(Remap, attributes(remap))]
pub fn derive_remap(input: TokenStream) -> TokenStream {
    derive(syn::parse_macro_input!(input))
}