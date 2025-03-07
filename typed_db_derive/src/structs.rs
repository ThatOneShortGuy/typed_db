use quote::quote;
use syn::{Lit, Result, spanned::Spanned};

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
            let mut value = String::new();
            attr.parse_nested_meta(|meta| {
                // #[default(CURRENT_TIMESTAMP)]
                if meta.path.is_ident("CURRENT_TIMESTAMP") {
                    value = "CURRENT_TIMESTAMP".to_string();
                    return Ok(());
                }
                // #[default(CURRENT_DATE)]
                if meta.path.is_ident("CURRENT_DATE") {
                    value = "CURRENT_DATE".to_string();
                    return Ok(());
                }
                // #[default(CURRENT_TIME)]
                if meta.path.is_ident("CURRENT_TIME") {
                    value = "CURRENT_TIME".to_string();
                    return Ok(());
                }
                // #[default(TRUE)]
                if meta.path.is_ident("TRUE") {
                    value = "TRUE".to_string();
                    return Ok(());
                }
                // #[default(FALSE)]
                if meta.path.is_ident("FALSE") {
                    value = "FALSE".to_string();
                    return Ok(());
                }

                if let Ok(lit) = meta.value()?.parse::<Lit>() {
                    match lit {
                        // #[default(numeric-literal)]
                        Lit::Int(lit_int) => {
                            value = lit_int.base10_digits().to_string();
                        }
                        // #[default("string-literal")]
                        Lit::Str(lit_str) => {
                            value = lit_str.value();
                        }
                        _ => return Err(meta.error("expected a numeric or string literal")),
                    };
                    return Ok(());
                }

                Err(meta.error("unrecognized default attribute"))
            })?;

            format!("DEFAULT {value}")
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
                "Cannot have both a primary key and composite keys",
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
            if !#composite_keys.is_empty() {
                lines.push(#composite_keys.to_string());
            }
            format!(
                "CREATE TABLE IF NOT EXISTS {} (
    {}
)",
                Self::TABLE_NAME,
                lines.join(",\n    ")
            )
        }
    }

    pub fn impl_dtable_str(&self) -> proc_macro2::TokenStream {
        let name = &self.name;
        let creation_str = self.creation_str();
        let column_names = self.fields_str();
        quote! {
            impl DbTable for #name {
                const TABLE_NAME: &'static str = stringify!(#name);
                fn create_table_str() -> String {
                    #creation_str
                }
                fn column_names() -> Box<[&'static str]> {
                    Box::new([#(#column_names),*])
                }
            }
        }
    }

    pub fn impl_builder_str(&self) -> proc_macro2::TokenStream {
        let original_name = &self.name;
        let name = syn::Ident::new(
            (original_name.to_string() + "Builder").as_str(),
            original_name.span(),
        );
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
            quote! {pub fn #with_name(mut self, #field_name: #ty) -> Self {self.#field_name = Some(#field_name); self}}
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

        quote! {
            #[derive(Debug, Clone)]
            pub struct #name {
                #(#full_types)*
            }

            impl #name {
                #(#with_fns)*
                pub fn build(self, conn: &::rusqlite::Connection) -> ::rusqlite::Result<usize> {
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
            pub fn select(conn: &rusqlite::Connection, where_clause: &str, params: impl rusqlite::Params) -> rusqlite::Result<Box<[Self]>> {
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
        let builder_name = syn::Ident::new((name.to_string() + "Builder").as_str(), name.span());
        let fields = self.fields.iter().map(|f| {
            let field_name = &f.name;
            quote! {#field_name: None,}
        });
        let select_where = self.impl_select_where();

        quote! {
            impl #name {
                pub fn new() -> #builder_name {
                    #builder_name {
                        #(#fields)*
                    }
                }
                #select_where
            }
        }
    }

    pub fn impls(&self) -> proc_macro2::TokenStream {
        let dtable_str = self.impl_dtable_str();
        let table_info_str = self.impl_table_info_str();
        let builder_str = self.impl_builder_str();
        quote! {
            #dtable_str
            #table_info_str
            #builder_str
        }
    }
}
