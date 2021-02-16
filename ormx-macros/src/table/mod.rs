use std::{collections::HashMap, convert::{TryFrom}};

use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Result, Type, Visibility};

use crate::attrs::{Getter, Insertable};
use crate::backend::{Backend, Implementation};
use std::borrow::Cow;
use std::marker::PhantomData;

mod parse;

pub struct Table<B: Backend> {
    pub ident: Ident,
    pub vis: Visibility,
    pub table: String,
    pub id: TableField<B>,
    pub fields: Vec<TableField<B>>,
    pub insertable: Option<Insertable>,
    pub engine: Option<String>,
    pub charset: Option<String>,
    pub collation: Option<String>,
    pub syncable: bool,
}

#[derive(Clone)]
pub struct TableField<B: Backend> {
    pub field: Ident,
    pub ty: Type,
    pub column_name: String,
    pub column_type: Option<String>,
    pub primary: bool,
    pub auto_increment: bool,
    pub allow_null: bool,
    pub unique: Option<Option<String>>,
    pub custom_type: bool,
    pub reserved_ident: bool,
    pub default: Option<String>,
    pub get_one: Option<Getter>,
    pub get_optional: Option<Getter>,
    pub get_many: Option<Getter>,
    pub set: Option<Ident>,
    pub _phantom: PhantomData<*const B>,
}

impl<B: Backend> Table<B> {
    pub fn fields_except_id(&self) -> impl Iterator<Item = &TableField<B>> + Clone {
        let id = self.id.field.clone();
        self.fields.iter().filter(move |field| field.field != id)
    }

    pub fn insertable_fields(&self) -> impl Iterator<Item = &TableField<B>> + Clone {
        self.fields_except_id().filter(|field| field.default.is_none())
    }

    pub fn default_fields(&self) -> impl Iterator<Item = &TableField<B>> + Clone {
        self.fields.iter().filter(|field| field.default.is_some())
    }

    pub fn select_column_list(&self) -> String {
        self.fields
            .iter()
            .map(TableField::fmt_for_select)
            .join(", ")
    }

    pub fn create_column_list(&self) -> String {
        let unique_clauses = self.fields.iter()
            .filter(|field| field.unique.is_some())
            .fold(HashMap::<String, Vec<&str>>::new(), |mut acc, field| {
                let unique = field.unique.as_ref().unwrap();
                let index_name = if let Some(index_name) = unique {
                    index_name.clone()
                } else {
                    format!("{}_uniq", field.field.to_string())
                };

                acc
                    .entry(index_name)
                    .or_default()
                    .push(&field.column_name);
                acc
            })
            .into_iter()
            .map(|(index_name, fields)| {
                format!(
                    "UNIQUE {quote}{}{quote} ({})",
                    index_name,
                    fields.into_iter().map(|field_name|
                        format!("{quote}{}{quote}", field_name, quote = B::QUOTE)
                    ).join(", "),
                    quote = B::QUOTE,
                )
            });

        self.fields
            .iter()
            .map(TableField::fmt_for_create)
            .chain(unique_clauses)
            .join(",\n")
    }
}

impl<B: Backend> TableField<B> {
    pub fn fmt_for_select(&self) -> String {
        if self.custom_type {
            format!(
                "{} AS {quote}{}: _{quote}",
                self.column(),
                self.field,
                quote = B::QUOTE
            )
        } else if self.field == self.column_name {
            self.column().into()
        } else {
            format!("{} AS {quote}{}{quote}", self.column(), self.field, quote = B::QUOTE)
        }
    }

    pub fn fmt_for_create(&self) -> String {
        if self.column_type.is_none() {
            panic!("column_type is not set for {}! Cannot sync", self.field);
        }

        format!(
            "{quote}{}{quote} {}{}{}{}",
            self.column_name,
            self.column_type.as_ref().unwrap(),
            if !self.allow_null {
                " NOT NULL"
            } else {
                ""
            },
            if self.primary {
                format!(" PRIMARY KEY NOT NULL{}", if self.auto_increment {
                    " AUTO_INCREMENT"
                } else {
                    ""
                })
            } else {
                "".into()
            },
            self.default.as_ref().map(|d| format!(" DEFAULT {}", d)).unwrap_or_default(),
            quote = B::QUOTE,
        )
    }

    pub fn column<'a>(&'a self) -> Cow<'a, str> {
        if self.reserved_ident {
            format!("{quote}{}{quote}", self.column_name, quote = B::QUOTE).into()
        } else {
            Cow::Borrowed(&self.column_name)
        }
    }
}

impl Getter {
    pub fn or_fallback<B: Backend>(&self, field: &TableField<B>) -> (Ident, Type) {
        let ident = self
            .func
            .clone()
            .unwrap_or_else(|| Ident::new(&format!("by_{}", field.field), Span::call_site()));
        let arg = self.arg_ty.clone().unwrap_or_else(|| {
            let ty = &field.ty;
            syn::parse2(quote!(&#ty)).unwrap()
        });
        (ident, arg)
    }
}

pub fn derive(input: DeriveInput) -> Result<TokenStream> {
    let parsed = Table::try_from(&input)?;

    let impl_table = Implementation::impl_table(&parsed);
    let insert_struct = Implementation::insert_struct(&parsed);
    let impl_insert = Implementation::impl_insert(&parsed);
    let getters = Implementation::impl_getters(&parsed);
    let setters = Implementation::impl_setters(&parsed);

    Ok(quote! {
        #impl_table
        #insert_struct
        #impl_insert
        #getters
        #setters
    })
}
