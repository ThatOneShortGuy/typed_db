mod default_value_parser;
mod foreign_key_parser;
mod structs;

use proc_macro::TokenStream;
use syn::{DataStruct, Ident, spanned::Spanned};

use structs::*;

#[proc_macro_derive(
    DbTable,
    attributes(default, primary_key, unique, composite_key, foreign_key)
)]
pub fn marshal_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_dbtable_macro(&ast)
}

fn impl_dbtable_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let data = match &ast.data {
        syn::Data::Struct(data_struct) => dbtable_struct(data_struct, name).impls(),
        syn::Data::Enum(data_enum) => {
            syn::Error::new(data_enum.enum_token.span(), "Enums are not valid DB Tables")
                .into_compile_error()
                .into()
        }
        syn::Data::Union(data_union) => syn::Error::new(
            data_union.union_token.span(),
            "Unions are not valid DB Tables",
        )
        .into_compile_error()
        .into(),
    };

    data.into()
}

fn dbtable_struct(data_struct: &DataStruct, name: &Ident) -> TableInfo {
    let fields = data_struct
        .fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().expect("no field name");
            let attributes = field.attrs.clone();
            let ty = field.ty.clone();
            let vis = field.vis.clone();
            FieldInfo {
                name: field_name.clone(),
                attributes,
                ty,
                visibility: vis,
            }
        })
        .collect();
    TableInfo {
        name: name.clone(),
        fields,
        attributes: vec![],
    }
}
