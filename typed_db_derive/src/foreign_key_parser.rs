use std::fmt::Display;

use syn::{
    Ident, LitStr, Result, Token, Type,
    parse::{Parse, ParseStream},
};

use crate::structs::TableColonField;

#[derive(Debug, Clone, Copy)]
pub enum FKAction {
    NoAction,
    Restrict,
    SetNull,
    SetDefault,
    Cascade,
}

mod fk_actions {
    syn::custom_keyword!(NO);
    syn::custom_keyword!(ACTION);
    syn::custom_keyword!(RESTRICT);
    syn::custom_keyword!(SET);
    syn::custom_keyword!(NULL);
    syn::custom_keyword!(DEFAULT);
    syn::custom_keyword!(CASCADE);
}

impl Parse for FKAction {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(fk_actions::NO) {
            input.parse::<fk_actions::NO>()?;
            input.parse::<fk_actions::ACTION>()?;
            Ok(FKAction::NoAction)
        } else if lookahead.peek(fk_actions::RESTRICT) {
            input.parse::<fk_actions::RESTRICT>()?;
            Ok(FKAction::Restrict)
        } else if lookahead.peek(fk_actions::SET) {
            input.parse::<fk_actions::SET>()?;
            let lookahead = input.lookahead1();
            if lookahead.peek(fk_actions::NULL) {
                input.parse::<fk_actions::NULL>()?;
                Ok(FKAction::SetNull)
            } else if lookahead.peek(fk_actions::DEFAULT) {
                input.parse::<fk_actions::DEFAULT>()?;
                Ok(FKAction::SetDefault)
            } else {
                Err(lookahead.error())
            }
        } else if lookahead.peek(fk_actions::CASCADE) {
            input.parse::<fk_actions::CASCADE>()?;
            Ok(FKAction::Cascade)
        } else if lookahead.peek(LitStr) {
            // Parse the inside of quotes
            let value = input.parse::<LitStr>()?;

            // Parse the inside of quotes.
            let str_value = value.value().to_lowercase();

            match str_value.as_str() {
                "no action" => Ok(FKAction::NoAction),
                "restrict" => Ok(FKAction::Restrict),
                "set null" => Ok(FKAction::SetNull),
                "set default" => Ok(FKAction::SetDefault),
                "cascade" => Ok(FKAction::Cascade),
                _ => Err(syn::Error::new(value.span(), "Unknown Action")),
            }
        } else {
            Err(lookahead.error())
        }
    }
}

impl Display for FKAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            FKAction::NoAction => "NO ACTION",
            FKAction::Restrict => "RESTRICT",
            FKAction::SetNull => "SET NULL",
            FKAction::SetDefault => "SET DEFAULT",
            FKAction::Cascade => "CASCADE",
        };
        write!(f, "{s}",)
    }
}

#[derive(Debug, Clone)]
pub struct ForeignKeyAttr {
    pub table: Type,
    pub foreign_field: Ident,
    pub on_delete: FKAction,
    pub on_update: FKAction,
}

impl Parse for ForeignKeyAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let TableColonField {
            table,
            field: foreign_field,
        } = input.parse()?;

        // Parse optional actions.
        let mut on_delete = FKAction::NoAction;
        let mut on_update = FKAction::NoAction;
        while !input.is_empty() {
            // Expect a comma before each optional action.
            let _comma: Token![,] = input.parse()?;
            let key: Key = input.parse()?;
            let _eq: Token![=] = input.parse()?;

            match key {
                Key::OnDelete => on_delete = input.parse()?,
                Key::OnUpdate => on_update = input.parse()?,
            }
        }
        Ok(Self {
            table: table.into_owned(),
            foreign_field: foreign_field.into_owned(),
            on_delete,
            on_update,
        })
    }
}

#[derive(Debug)]
enum Key {
    OnDelete,
    OnUpdate,
}

impl Parse for Key {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::on_delete) {
            input.parse::<kw::on_delete>().unwrap();
            Ok(Key::OnDelete)
        } else if lookahead.peek(kw::on_update) {
            input.parse::<kw::on_update>()?;
            Ok(Key::OnUpdate)
        } else {
            Err(lookahead.error())
        }
    }
}

mod kw {
    syn::custom_keyword!(on_delete);
    syn::custom_keyword!(on_update);
}
