use syn::{
    Ident, LitStr, Result, Token, Type,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::structs::TableColonField;

#[derive(Debug, Clone)]
pub struct CteFieldParam {
    pub table: Type,
    pub table_shorthand: LitStr,
    pub field_name: Ident,
    pub val: LitStr,
}

impl CteFieldParam {
    pub fn validity_check(&self, ty: &syn::Type) -> proc_macro2::TokenStream {
        let tcf = TableColonField {
            table: std::borrow::Cow::Borrowed(&self.table),
            field: std::borrow::Cow::Borrowed(&self.field_name),
        };
        tcf.validity_check(&ty)
    }
}

impl Parse for CteFieldParam {
    fn parse(input: ParseStream) -> Result<Self> {
        let TableColonField {
            table,
            field: field_name,
        } = input.parse()?;

        let _: Token![as] = input.parse()?;
        let table_shorthand: LitStr = input.parse()?;

        let _: Token![,] = input.parse()?;
        let val = input.parse::<LitStr>()?;

        Ok(Self {
            table: table.into_owned(),
            table_shorthand,
            field_name: field_name.into_owned(),
            val,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CteTableParams {
    pub param_list: Box<[LitStr]>,
}

impl Parse for CteTableParams {
    fn parse(input: ParseStream) -> Result<Self> {
        let param_list = Punctuated::<LitStr, Token![,]>::parse_terminated(input)?
            .into_iter()
            .collect();

        Ok(Self { param_list })
    }
}
