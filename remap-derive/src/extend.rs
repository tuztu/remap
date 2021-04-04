use proc_macro::TokenStream;
use std::str::FromStr;

use quote::quote;

pub fn impl_table(input: TokenStream) -> TokenStream {
    let input = syn::parse::<syn::DeriveInput>(input).unwrap();
    let ident = &input.ident;

    let fields = match input.data {
        syn::Data::Struct(ref s) => &s.fields,
        _ => panic!("Entity can only be derived for struct"),
    };

    let mut fields_name = String::new();
    for f in fields {
        if let Some(id) = f.ident.as_ref() {
            if !fields_name.is_empty() { fields_name.push_str(",") }
            fields_name.push_str(id.to_string().as_str());
        }
    }

    let fields_vec = fields.iter().filter_map(|field| {
        field.ident.as_ref().map(|a| { a.to_string() })
    }).collect::<Vec<String>>();

    // Build bind args statement
    let mut build_args = String::new();
    for x in &fields_vec {
        build_args.push_str(format!(r#".bind(&self.{})"#, x).as_str());
    }

    // Build from row statements.
    let mut from_row_value = String::new();
    for x in &fields_vec {
        from_row_value.push_str(format!(r#"{0}: row.try_get("{0}")?,"#, x).as_str());
    }

    let expanded = quote! {
        impl Table for #ident {
            fn struct_name() -> String {
                // "name".to_string()
                stringify!(#ident).to_string()
                // #struct_name.to_string()

            }

            fn fields_name() -> Vec<String> {
                #fields_name.split(",").map(|a| a.to_string()).collect::<Vec<String>>()
            }

            fn bind_args(&self, mut args: Args) -> Args {
                args[build_args_holder]
            }

            fn from_mysql_row(row: MySqlRow) -> Result<#ident, Error> {
                let t = #ident {
                    [from_row_holder]
                };
                Ok(t)
            }
        }
    };

    let expanded = expanded.to_string()
        .replace("[build_args_holder]", build_args.as_str())
        .replace("[from_row_holder]", from_row_value.as_str());
    TokenStream::from_str(expanded.as_str()).unwrap()

    // TokenStream::from(expanded)
}