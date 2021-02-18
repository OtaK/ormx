use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::backend::postgres::{PgBackend, PgBindings};
use crate::table::{Table, TableField};

fn insert_sql(table: &Table<sqlx::Postgres, PgBackend>, insert_fields: &[&TableField<sqlx::Postgres, PgBackend>]) -> String {
    format!(
        "INSERT INTO {} ({}) VALUES ({}) RETURNING {}",
        table.table,
        insert_fields.iter().map(|field| field.column()).join(", "),
        PgBindings::default().take(insert_fields.len()).join(", "),
        table.id.fmt_for_select()
    )
}

fn query_default_sql(
    table: &Table<sqlx::Postgres, PgBackend>,
    default_fields: &[&TableField<sqlx::Postgres, PgBackend>],
) -> String {
    format!(
        "SELECT {} FROM {} WHERE {} = {}",
        default_fields
            .iter()
            .map(|field| field.fmt_for_select())
            .join(", "),
        table.table,
        table.id.column(),
        PgBindings::default().next().unwrap()
    )
}

pub fn impl_insert(table: &Table<sqlx::Postgres, PgBackend>) -> TokenStream {
    let insert_ident = match &table.insertable {
        Some(i) => &i.ident,
        None => return quote!(),
    };

    let insert_fields: Vec<&TableField<sqlx::Postgres, PgBackend>> = table.insertable_fields().collect();
    let default_fields: Vec<&TableField<sqlx::Postgres, PgBackend>> = table.default_fields().collect();

    let id_ident = &table.id.field;
    let table_ident = &table.ident;
    let insert_field_idents = insert_fields
        .iter()
        .map(|field| &field.field)
        .collect::<Vec<&Ident>>();
    let default_field_idents = default_fields
        .iter()
        .map(|field| &field.field)
        .collect::<Vec<&Ident>>();

    let insert_sql = insert_sql(table, &insert_fields);

    let query_default_sql = query_default_sql(table, &default_fields);
    let query_default = if default_fields.is_empty() {
        quote!()
    } else {
        quote! {
            let _generated = sqlx::query!(#query_default_sql, _id)
                .fetch_one(db)
                .await?;
        }
    };

    let insert_field_exprs = insert_fields
        .iter()
        .map(|field| {
            let ident = &field.field;
            let ty = &field.ty;
            match field.custom_type {
                true => quote!(self.#ident as #ty),
                false => quote!(self.#ident),
            }
        })
        .collect::<Vec<TokenStream>>();

    let box_future = crate::utils::box_future();
    quote! {
        impl ormx::Insert for #insert_ident {
            type Table = #table_ident;

            fn insert(
                self,
                db: &mut sqlx::PgConnection,
            ) -> #box_future<sqlx::Result<Self::Table>> {
                Box::pin(async move {
                    let _id = sqlx::query!(#insert_sql, #( #insert_field_exprs, )*)
                        .fetch_one(db as &mut sqlx::PgConnection)
                        .await?
                        .#id_ident;

                    #query_default

                    Ok(Self::Table {
                        #id_ident: _id as _,
                        #( #insert_field_idents: self.#insert_field_idents, )*
                        #( #default_field_idents: _generated.#default_field_idents, )*
                    })
                })
            }
        }
    }
}
