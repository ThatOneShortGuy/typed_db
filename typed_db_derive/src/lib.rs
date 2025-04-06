mod cte_info;
mod cte_params;
mod default_value_parser;
mod foreign_key_parser;
mod structs;

use cte_info::{CteFieldInfo, CteInfo};
use proc_macro::TokenStream;
use syn::{Attribute, DataStruct, Ident, spanned::Spanned};

use structs::*;

#[proc_macro_derive(
    DbTable,
    attributes(default, primary_key, unique, composite_key, foreign_key)
)]
pub fn dbtable_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_dbtable_macro(&ast)
}

fn impl_dbtable_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let attrs = &ast.attrs;

    let data = match &ast.data {
        syn::Data::Struct(data_struct) => dbtable_struct(data_struct, name, attrs).impls(),
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

fn dbtable_struct(data_struct: &DataStruct, name: &Ident, attrs: &[Attribute]) -> TableInfo {
    let fields = data_struct
        .fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().expect("no field name");
            let attributes = field.attrs.clone();
            let ty = field.ty.clone();
            let vis = field.vis.clone();
            TableFieldInfo {
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
        attributes: attrs.into(),
    }
}

#[proc_macro_derive(CommonTableExpression, attributes(param, cte_params))]
pub fn dbview_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_dbview_macro(&ast)
}

fn impl_dbview_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let attrs = &ast.attrs;

    let data = match &ast.data {
        syn::Data::Struct(data_struct) => cte_struct(data_struct, name, attrs).impls(),
        syn::Data::Enum(data_enum) => {
            syn::Error::new(data_enum.enum_token.span(), "Enums are not valid DB Views")
                .into_compile_error()
                .into()
        }
        syn::Data::Union(data_union) => syn::Error::new(
            data_union.union_token.span(),
            "Unions are not valid DB Views",
        )
        .into_compile_error()
        .into(),
    };

    data.into()
}

fn cte_struct(data_struct: &DataStruct, name: &Ident, attrs: &[Attribute]) -> CteInfo {
    let fields = data_struct
        .fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().expect("no field name");
            let attributes = field.attrs.clone();
            let ty = field.ty.clone();
            let vis = field.vis.clone();
            CteFieldInfo {
                name: field_name.clone(),
                attributes,
                ty,
                visibility: vis,
            }
        })
        .collect();
    CteInfo {
        name: name.clone(),
        fields,
        attributes: attrs.into(),
    }
}
