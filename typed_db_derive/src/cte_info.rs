use quote::{ToTokens, quote};
use syn::{Result, spanned::Spanned};

use crate::cte_params::{CteFieldParam, CteTableParams};

pub struct CteFieldInfo {
    #[allow(unused)]
    pub visibility: syn::Visibility,
    pub name: proc_macro2::Ident,
    pub attributes: Vec<syn::Attribute>,
    #[allow(unused)]
    pub ty: syn::Type,
}

impl CteFieldInfo {
    fn param(&self) -> Result<Option<CteFieldParam>> {
        let param_attrs = self
            .attributes
            .iter()
            .filter(|attr| attr.path().is_ident("param"))
            .collect::<Vec<_>>();

        if param_attrs.len() > 1 {
            return Err(syn::Error::new(
                self.name.span(),
                "Only one param attribute is allowed per field",
            ));
        }
        let attr = param_attrs.into_iter().next();

        let attr = match attr {
            Some(attr) => attr,
            None => return Ok(None),
        };

        Ok(Some(attr.parse_args()?))
    }
    pub fn select_stmt(&self) -> Result<(proc_macro2::TokenStream, String)> {
        let name = &self.name;
        let cte = match self.param()? {
            Some(p) => p,
            None => {
                return Err(syn::Error::new(
                    name.span(),
                    "`#[param(...)] attribute needed",
                ));
            }
        };
        let check = cte.validity_check(&self.ty);
        let CteFieldParam {
            table: table_name,
            field_name,
            val: where_clause,
            table_shorthand,
        } = cte;
        let table_shorthand = table_shorthand.value();
        let s = format!(
            "(SELECT {table_shorthand}.{field_name} FROM {} AS {table_shorthand}, params WHERE {}) AS {name}",
            table_name.into_token_stream(),
            where_clause.value()
        );

        Ok((check, s))
    }
}

pub struct CteInfo {
    pub name: syn::Ident,
    pub fields: Vec<CteFieldInfo>,
    pub attributes: Vec<syn::Attribute>,
}

impl CteInfo {
    fn cte_str_params(&self) -> Result<String> {
        let cte_attrs = self
            .attributes
            .iter()
            .filter(|attr| attr.path().is_ident("cte_params"))
            .collect::<Vec<_>>();

        if cte_attrs.len() > 1 {
            return Err(syn::Error::new(
                cte_attrs[1].path().span(),
                "Only one cte_params attribute is allowed for the CTE",
            ));
        }

        let attr = cte_attrs.into_iter().next();
        let attr = match attr {
            Some(a) => a,
            None => return Ok(String::new()),
        };

        let CteTableParams { param_list } = attr.parse_args()?;
        let param_list = param_list
            .into_iter()
            .map(|p| format!("? AS {}", p.value()))
            .collect::<Box<[String]>>()
            .join(", ");
        let s = format!(
            "WITH params AS (
	SELECT {param_list}
)",
        );

        Ok(s)
    }
    pub fn impl_cte_str_fn(&self) -> proc_macro2::TokenStream {
        let param_str = match self.cte_str_params() {
            Ok(s) => s,
            Err(err) => return err.to_compile_error(),
        };

        let mut checks = Vec::new();

        let fields = self
            .fields
            .iter()
            .map(|f| {
                let (a, b) = f.select_stmt()?;
                checks.push(a);
                Ok(b)
            })
            .collect::<Result<Vec<_>>>();

        let fields = match fields {
            Ok(v) => v.join(",\n"),
            Err(err) => return err.to_compile_error(),
        };

        let s = format!("{param_str}SELECT {fields};");

        quote! { #(#checks)* #s }
    }
    pub fn impl_cte(&self) -> proc_macro2::TokenStream {
        let name = &self.name;
        let cte_str_fn = self.impl_cte_str_fn();
        let field_getters = self.fields.iter().map(|f| {
            let f_name = &f.name;
            quote! { #f_name: row.get(stringify!(#f_name))?,}
        });
        quote! {
            impl CommonTableExpression for #name {
                fn cte_str() -> &'static str {
                    #cte_str_fn
                }
                fn select(
                    conn: &rusqlite::Connection,
                    params: impl rusqlite::Params,
                ) -> rusqlite::Result<Box<[Self]>> {
                    let mut stmt = conn.prepare(&Self::cte_str())?;
                    let rows = stmt
                        .query_map(params, |row| {
                            Ok(Self {
                                #(#field_getters)*
                            })
                        })?
                        .collect::<rusqlite::Result<_>>()?;
                    Ok(rows)
                }
            }
        }
    }

    pub fn impl_self(&self) -> proc_macro2::TokenStream {
        let name = &self.name;
        quote! {
            impl #name {
                #[automatically_derived]
                pub fn print_query_plan(conn: &Connection, params: impl Params) -> Result<(), rusqlite::Error> {
                    let query_plan_str = format!("EXPLAIN QUERY PLAN {}", Self::cte_str());
                    let mut stmt = conn.prepare(&query_plan_str)?;
                    let rows = stmt.query_map(params, |row| {
                        Ok((
                            row.get::<_, i32>(0)?,
                            row.get::<_, i32>(1)?,
                            row.get::<_, i32>(2)?,
                            row.get::<_, String>(3)?,
                        ))
                    })?;
                    println!("Query Plan:");
                    for row in rows {
                        let (selectid, order, from, detail) = row?;
                        println!(
                            "selectid: {}, order: {}, from: {}, detail: {}",
                            selectid, order, from, detail
                        );
                    }

                    Ok(())
                }
            }
        }
    }

    pub fn impls(&self) -> proc_macro2::TokenStream {
        let cte_impl = self.impl_cte();
        let self_impl = self.impl_self();

        quote! {
            #cte_impl
            #self_impl
        }
    }
}
