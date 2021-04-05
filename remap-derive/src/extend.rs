use proc_macro::TokenStream;
use std::str::FromStr;

use heck::SnakeCase;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{Ident, Lit, Meta, NestedMeta, punctuated::Punctuated};

pub fn derive(ast: syn::DeriveInput) -> TokenStream {
    let (db, table) = parse_attrs(&ast);
    let ident = ast.ident;

    let fields = match ast.data {
        syn::Data::Struct(ref s) => &s.fields,
        _ => panic!("Remap can only be derived for struct"),
    };

    let mut fields_str = String::new();
    let mut fields_vec = Vec::new();
    for field in fields {
        if let Some(ident) = field.ident.as_ref() {
            let name = ident.to_string();
            let item = format!(" \"{}\", ", name);
            fields_str.push_str(item.as_str());
            fields_vec.push(name)
        }
    }

    let mut add_args_str = String::new();
    for x in &fields_vec {
        add_args_str.push_str(format!(r#".add(&self.{}) "#, x).as_str());
    }

    let mut decode_rows_str = String::new();
    for x in &fields_vec {
        decode_rows_str.push_str(format!(r#"{0}: row.try_get("{0}")?, "#, x).as_str());
    }

    let token = quote!(
        impl Remap<_db_> for #ident {
            fn table_name() -> &'static str {
                stringify!(#table)
            }
            fn fields_name() -> Vec<&'static str> {
                vec![ _fields_str_ ]
            }
            fn fields_args(&self) -> remap::arguments::Args<_db_> {
                remap::arguments::Args::new()_add_args_str_
            }
            fn decode_row(row: <_db_ as sqlx::Database>::Row) -> Result<Self, anyhow::Error> {
                use sqlx::Row;
                let x = Self {
                    _decode_rows_str_
                };
                Ok(x)
            }
        }
    );

    let token = token.to_string()
        .replace("_db_", db.as_str())
        .replace("_fields_str_", fields_str.as_str())
        .replace("_add_args_str_", add_args_str.as_str())
        .replace("_decode_rows_str_", decode_rows_str.as_str());

    // panic!("{}", token);

    TokenStream::from_str(token.as_str()).expect("Parse token stream failed")
}

fn parse_attrs(ast: &syn::DeriveInput) -> (String, String) {
    ast.attrs.iter().find_map(|attr| {
        if let Meta::List(meta_list) = attr.parse_meta().unwrap() {
            if meta_list.path.get_ident() ==
                Some(&Ident::new("remap", Span::call_site())) {
                let (db_type, table) = parse_remap(&meta_list.nested);
                let db_type = db_type.expect("Must specify a database");
                let table = table.unwrap_or_else(||
                    (&ast.ident).to_string().to_snake_case());
                return Some((db_type, table));
            }
        }
        None
    }).expect("Must specify a database")
}

fn parse_remap<P>(props: &Punctuated<NestedMeta, P>) -> (Option<String>, Option<String>) {
    let db_type = props.iter().find_map(|item| {
        if let NestedMeta::Meta(Meta::Path(path)) = item {
            return Some(path.to_token_stream().to_string());
        }
        None
    });

    let table = props.iter().find_map(|item| {
        if let NestedMeta::Meta(Meta::NameValue(name_value)) = item {
            if name_value.path.get_ident() ==
                Some(&Ident::new("table", Span::call_site())){
                if let Lit::Str(s) = &name_value.lit {
                    return Some(s.value());
                }
            }
        }
        None
    });

    (db_type, table)
}