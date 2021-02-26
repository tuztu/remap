use proc_macro::TokenStream;

use crate::table::impl_table;

pub(crate) mod table;

#[proc_macro_derive(Table)]
pub fn derive_table(input: TokenStream) -> TokenStream {
    impl_table(input)
}