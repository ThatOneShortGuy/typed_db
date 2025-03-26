use std::collections::HashMap;

use quote::{ToTokens, quote};
use syn::{Result, spanned::Spanned};

use crate::{default_value_parser::*, foreign_key_parser::ForeignKeyAttr};

pub struct FieldInfo {
    pub visibility: syn::Visibility,
    pub name: proc_macro2::Ident,
    pub attributes: Vec<syn::Attribute>,
    pub ty: syn::Type,
}

impl FieldInfo {
    pub fn as_txt(&self) -> Result<proc_macro2::TokenStream> {
        // SQLite format creation string
        let field_name = &self.name;
        let ty = &self.ty;

        let constraints = self.column_constraints()?;
        let out = quote! {
            format!(stringify!(#field_name {} {}), <#ty as DbType>::db_type(), #constraints)
        };
        Ok(out)
    }

    fn is_primary_key(&self) -> bool {
        self.attributes
            .iter()
            .any(|attr| attr.path().is_ident("primary_key"))
    }

    fn is_composite_key(&self) -> bool {
        self.attributes
            .iter()
            .any(|attr| attr.path().is_ident("composite_key"))
    }

    fn primary_key_text(&self) -> &str {
        // Check for an attribute to the field called `primary_key`
        if self.is_primary_key() {
            "PRIMARY KEY"
        } else {
            ""
        }
    }

    fn unique_text(&self) -> &str {
        // Check for an attribute to the field called `unique`
        if self
            .attributes
            .iter()
            .any(|attr| attr.path().is_ident("unique"))
        {
            "UNIQUE"
        } else {
            ""
        }
    }

    fn default_text(&self) -> Result<String> {
        // Check for an attribute to the field called `default("value")`
        let attrs = self
            .attributes
            .iter()
            .filter(|attr| attr.path().is_ident("default"))
            .collect::<Vec<_>>();
        if attrs.len() > 1 {
            return Err(syn::Error::new(
                attrs[1].span(),
                "Only one default attribute allowed per field",
            ));
        }
        let attr = attrs.into_iter().next();
        let ret = if let Some(attr) = attr {
            let val: DefaultValues = attr.parse_args()?;
            format!("DEFAULT {val}")
        } else {
            "".to_string()
        };
        Ok(ret)
    }

    fn is_optional(&self) -> bool {
        match &self.ty {
            syn::Type::Path(path) => path
                .path
                .segments
                .last()
                .map_or(false, |s| s.ident == "Option"),
            _ => false,
        }
    }

    fn foreign_key(&self) -> Result<Option<ForeignKeyAttr>> {
        let foreign_keys = self
            .attributes
            .iter()
            .filter(|attr| attr.path().is_ident("foreign_key"))
            .collect::<Vec<_>>();

        if foreign_keys.len() > 1 {
            return Err(syn::Error::new(
                foreign_keys[1].span(),
                "Field can only be one foreign key",
            ));
        }

        let fk = match foreign_keys.into_iter().next() {
            None => return Ok(None),
            Some(fk) => fk,
        };

        let fk_attr = Some(fk.parse_args()?);

        Ok(fk_attr)
    }

    fn column_constraints(&self) -> Result<String> {
        let optional_text = if self.is_optional() { "" } else { "NOT NULL" };
        let optional_text = [
            optional_text,
            self.primary_key_text(),
            self.unique_text(),
            self.default_text()?.as_str(),
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
        Ok(optional_text)
    }
}

pub struct TableInfo {
    pub name: syn::Ident,
    pub fields: Vec<FieldInfo>,
    #[allow(dead_code)]
    pub attributes: Vec<syn::Attribute>,
}

impl TableInfo {
    pub fn fields_str(&self) -> impl Iterator<Item = proc_macro2::TokenStream> {
        self.fields.iter().map(|f| {
            let fname = f.name.to_string();
            quote! { #fname }
        })
    }
    pub fn separated_fields(&self, sep: &str) -> String {
        self.fields
            .iter()
            .map(|f| f.name.to_string())
            .collect::<Vec<_>>()
            .join(sep)
    }
    pub fn builder_name(&self) -> syn::Ident {
        let name = &self.name;
        syn::Ident::new((name.to_string() + "Builder").as_str(), name.span())
    }

    pub fn foreign_keys(&self) -> Result<Vec<String>> {
        let mut foreign_tables = HashMap::<_, Vec<_>>::new();
        for f in self.fields.iter() {
            let fk = match f.foreign_key()? {
                None => continue,
                Some(fk) => fk,
            };

            foreign_tables
                .entry(fk.table.to_token_stream().to_string())
                .or_default()
                .push((f, fk));
        }

        let out = foreign_tables
            .into_iter()
            .map(|(k, v)| {
                let self_ids = v
                    .iter()
                    .map(|fk| fk.0.name.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                let foreign_ids = v
                    .iter()
                    .map(|fk| fk.1.foreign_field.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                let actions = v
                    .into_iter()
                    .map(|(_, fk)| {
                        format!(
                            "ON UPDATE {} ON DELETE {}",
                            fk.on_update.to_string(),
                            fk.on_delete.to_string()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                // let table_name = v[0].1.table;

                format!("FOREIGN KEY ({self_ids}) REFERENCES {k}({foreign_ids}) {actions}",)
            })
            .collect::<Vec<_>>();
        Ok(out)
    }

    pub fn creation_str(&self) -> proc_macro2::TokenStream {
        let data = self
            .fields
            .iter()
            .map(|f| f.as_txt().unwrap_or_else(|e| e.to_compile_error()));

        let composite_keys = self
            .fields
            .iter()
            .filter(|f| f.is_composite_key())
            .map(|f| &f.name)
            .collect::<Vec<_>>();

        let primary_keys = self
            .fields
            .iter()
            .filter(|f| f.is_primary_key())
            .collect::<Vec<_>>();

        let foreign_keys = match self.foreign_keys() {
            Ok(v) => v,
            Err(e) => return e.to_compile_error(),
        };

        if primary_keys.len() > 1 {
            return syn::Error::new(
                self.fields[1].name.span(),
                "Only one primary key allowed per table, use `#[composite_key]` instead",
            )
            .to_compile_error();
        }

        if primary_keys.len() > 0 && composite_keys.len() > 0 {
            return syn::Error::new(
                primary_keys[0].name.span(),
                "Cannot have both a primary key and composite keys, use `#[composite_key]` instead",
            )
            .to_compile_error();
        }

        let composite_keys = if composite_keys.len() > 0 {
            quote! { stringify!(PRIMARY KEY (#(#composite_keys),*) )}
        } else {
            quote! {""}
        };

        quote! {
            let mut lines = vec![#(#data),*];
            let foreign_keys = vec![#(#foreign_keys.to_string()),*];
            if !#composite_keys.is_empty() {
                lines.push(#composite_keys.to_string());
            }
            lines.extend(foreign_keys);
            format!(
                "CREATE TABLE IF NOT EXISTS {} (
    {}
)",
                Self::TABLE_NAME,
                lines.join(",\n    "),
            )
        }
    }

    pub fn impl_dtable_str(&self) -> proc_macro2::TokenStream {
        let name = &self.name;
        let creation_str = self.creation_str();
        let column_names = self.fields_str();
        let select_where = self.impl_select_where();
        quote! {
            #[automatically_derived]
            impl DbTable for #name {
                const TABLE_NAME: &'static str = stringify!(#name);
                fn create_table_str() -> String {
                    #creation_str
                }
                fn column_names() -> Box<[&'static str]> {
                    Box::new([#(#column_names),*])
                }
                #select_where
            }
        }
    }

    pub fn impl_builder_str(&self) -> proc_macro2::TokenStream {
        let original_name = &self.name;
        let name = self.builder_name();
        let full_types = self.fields.iter().map(|f| {
            let field_name = &f.name;
            let ty = &f.ty;
            let vis = &f.visibility;
            quote! {#vis #field_name: ::std::option::Option<#ty>,}
        });
        let with_fns = self.fields.iter().map(|f| {
            let field_name = &f.name;
            let with_name = syn::Ident::new(&format!("with_{field_name}"), field_name.span());
            let ty = &f.ty;
            quote! {#[automatically_derived] pub fn #with_name(mut self, #field_name: impl Into<#ty>) -> Self {self.#field_name = Some(#field_name.into()); self}}
        });
        let build_str = self.fields.iter().map(|f| {
            let fname = &f.name;
            let fname_str = f.name.to_string();
            quote! {
                if let Some(#fname) = self.#fname {
                    fnames.push(#fname_str);
                    values.push(Box::new(#fname));
                }
            }
        });
        let row_getters = self.fields.iter().enumerate().map(|(i, f)| {
            let name = &f.name;
            quote! { #name: row.get(#i)?, }
        });

        quote! {
            #[automatically_derived]
            #[derive(Debug, Clone)]
            pub struct #name {
                #(#full_types)*
            }

            #[automatically_derived]
            impl #name {
                #(#with_fns)*

                #[automatically_derived]
                /// Inserts the item into the db without returning the row id. Returns the default `rusqlite` instead
                pub fn build_raw(self, conn: &::rusqlite::Connection) -> ::rusqlite::Result<usize> {
                    let mut fnames = vec![];
                    let mut values: Vec<Box<dyn rusqlite::ToSql>> = vec![];

                    #(#build_str)*

                    let value_params: Vec<_> = (1..values.len() + 1).map(|i| format!("?{i}")).collect();
                    let insert_str = format!(
                        "INSERT INTO {} ({}) VALUES ({})",
                        #original_name::TABLE_NAME,
                        fnames.join(","),
                        value_params.join(",")
                    );
                    let values_refs: Vec<&dyn rusqlite::ToSql> =
                        values.iter().map(|v| v as &dyn rusqlite::ToSql).collect();
                    conn.execute(&insert_str, values_refs.as_slice())
                }

                #[automatically_derived]
                /// Inserts the row into the database and returns the [ROWID](https://www.sqlite.org/lang_createtable.html#rowid)
                pub fn build(self, conn: &::rusqlite::Connection) -> ::rusqlite::Result<i64> {
                    self.build_raw(&conn)?;
                    Ok(conn.last_insert_rowid())
                }

                #[automatically_derived]
                /// Inserts and returns the new object with all data from the db
                pub fn build_val(self, conn: &::rusqlite::Connection) -> ::rusqlite::Result<#original_name> {
                    let rowid = self.build(&conn)?;
                    let sql = format!("SELECT * FROM {} WHERE ROWID = {rowid}", #original_name::TABLE_NAME);
                    let mut stmt = conn.prepare(&sql)?;
                    stmt.query_map([], |row| {
                            Ok(#original_name {
                                #(#row_getters)*
                            })
                        })?
                        .next()
                        .unwrap()
                }
            }

        }
    }

    fn impl_select_where(&self) -> proc_macro2::TokenStream {
        let comma_separated_cols = self.separated_fields(",");
        let row_getters = self.fields.iter().enumerate().map(|(i, f)| {
            let name = &f.name;
            quote! { #name: row.get(#i)?, }
        });

        quote! {
            #[automatically_derived]
            fn select(conn: &rusqlite::Connection, where_clause: &str, params: impl rusqlite::Params) -> rusqlite::Result<Box<[Self]>> {
                let sql = format!("SELECT {} FROM {} {}", #comma_separated_cols, Self::TABLE_NAME, where_clause);
                let mut stmt = conn.prepare(&sql)?;
                let iter = stmt.query_map(params, |row| {
                    Ok(Self {
                        #(#row_getters)*
                    })
                })?
                .collect::<rusqlite::Result<_>>()?;
                Ok(iter)
            }
        }
    }

    fn impl_table_info_str(&self) -> proc_macro2::TokenStream {
        let name = &self.name;
        let builder_name = self.builder_name();
        let fields = self.fields.iter().map(|f| {
            let field_name = &f.name;
            quote! {#field_name: None,}
        });

        quote! {
            #[automatically_derived]
            impl #name {
                pub fn new() -> #builder_name {
                    #builder_name {
                        #(#fields)*
                    }
                }
            }
        }
    }

    fn impl_table_tests(&self) -> proc_macro2::TokenStream {
        let name = &self.name;
        let test_name = syn::Ident::new(&(name.to_string() + "_gen_tests"), name.span());

        let foreign_field_type_inits = self.fields.iter().map(|f| match f.foreign_key() {
            Ok(fk) => match fk {
                Some(fk) => {
                    let ty = fk.table;
                    quote! { #ty::create_table(&conn)?; }
                }
                None => quote! {},
            },
            Err(e) => e.into_compile_error(),
        });

        let build_fields = self.fields.iter().map(|f| {
            let field_name = &f.name;
            let with_name = syn::Ident::new(&format!("with_{field_name}"), field_name.span());
            let ty = &f.ty;

            quote! {.#with_name(<#ty as ::std::default::Default>::default())}
        });

        quote! {
            #[cfg(test)]
            #[automatically_derived]
            #[allow(non_snake_case)]
            mod #test_name {
                use super::*;

                #[test]
                fn create() -> ::core::result::Result<(), Box<dyn std::error::Error>> {
                    let conn = ::rusqlite::Connection::open(":memory:")?;
                    conn.execute("PRAGMA foreign_keys = ON;", [])?;
                    #(#foreign_field_type_inits)*
                    #name::create_table(&conn)?;
                    conn.execute("PRAGMA foreign_keys = OFF;", [])?;

                    #name::new()#(#build_fields)*.build_val(&conn)?;
                    Ok(())
                }
            }
        }
    }

    pub fn impls(&self) -> proc_macro2::TokenStream {
        let dtable_str = self.impl_dtable_str();
        let table_info_str = self.impl_table_info_str();
        let builder_str = self.impl_builder_str();
        let tests_str = self.impl_table_tests();
        quote! {
            #dtable_str
            #table_info_str
            #builder_str
            #tests_str
        }
    }
}
