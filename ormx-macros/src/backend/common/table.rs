use proc_macro2::TokenStream;
use quote::{TokenStreamExt, quote};

use crate::backend::Backend;
use crate::table::Table;

pub fn impl_table<D: sqlx::Database, B: Backend<D>>(table: &Table<D, B>) -> TokenStream {
    let table_ident = &table.ident;
    let id_ident = &table.id.field;
    let id_ty = &table.id.ty;
    let column_list = table.select_column_list();

    let get = get::<D, B>(table, &column_list);
    let stream_all = stream_all(table, &column_list);
    let stream_all_paginated = stream_all_paginated::<D, B>(table, &column_list);
    let update = update::<D, B>(table);
    let delete = delete::<D, B>(table);
    let sync_safe = sync_safe::<D, B>(table);

    quote! {
        impl ormx::Table for #table_ident {
            type Id = #id_ty;

            fn id(&self) -> Self::Id { self.#id_ident }

            #get
            #stream_all
            #stream_all_paginated
            #update
            #delete
            #sync_safe
        }
    }
}

fn sync_safe<D: sqlx::Database, B: Backend<D>>(table: &Table<D, B>) -> TokenStream {
    if !table.syncable {
        return TokenStream::default();
    }

    let box_future = crate::utils::box_future();
    let sync_sql = format!(
        "CREATE TABLE IF NOT EXISTS {quote}{}{quote} (\n{}\n){}{}{}",
        table.table,
        table.create_column_list(),
        table.engine.as_ref().map(|e| format!(" ENGINE={}", e)).unwrap_or_default(),
        table.charset.as_ref().map(|e| format!(" DEFAULT CHARSET={}", e)).unwrap_or_default(),
        table.collation.as_ref().map(|e| format!(" DEFAULT COLLATE={}", e)).unwrap_or_default(),
        quote = B::QUOTE,
    );
    println!("{}", sync_sql);
    quote! {
        fn sync_safe<'a, 'c: 'a>(
            db: impl sqlx::Executor<'c, Database = ormx::Db> + 'a,
        ) -> #box_future<'a, sqlx::Result<()>> {
            Box::pin(async move {
                sqlx::query!(#sync_sql)
                    .execute(db)
                    .await?;

                Ok(())
            })
        }
    }
}

fn get<D: sqlx::Database, B: Backend<D>>(table: &Table<D, B>, column_list: &str) -> TokenStream {
    let box_future = crate::utils::box_future();
    let get_sql = format!(
        "SELECT {} FROM {quote}{}{quote} WHERE {} = {}",
        column_list,
        table.table,
        table.id.column(),
        B::Bindings::default().next().unwrap(),
        quote = B::QUOTE,
    );

    quote! {
        fn get<'a, 'c: 'a>(
            db: impl sqlx::Executor<'c, Database = ormx::Db> + 'a,
            id: Self::Id,
        ) -> #box_future<'a, sqlx::Result<Self>> {
            Box::pin(async move {
                sqlx::query_as!(Self, #get_sql, id)
                    .fetch_one(db)
                    .await
            })
        }
    }
}

fn update<D: sqlx::Database, B: Backend<D>>(table: &Table<D, B>) -> TokenStream {
    let box_future = crate::utils::box_future();
    let mut bindings = B::Bindings::default();
    let mut assignments = vec![];
    for field in table.fields_except_id() {
        let fragment = format!("{} = {}", field.column(), bindings.next().unwrap());
        assignments.push(fragment);
    }
    let assignments = assignments.join(", ");

    let update_sql = format!(
        "UPDATE {} SET {} WHERE {} = {}",
        table.table,
        assignments,
        table.id.column(),
        bindings.next().unwrap()
    );
    let id_argument = &table.id.field;
    let other_arguments = table.fields_except_id().map(|field| {
        let ident = &field.field;
        let mut out = quote!(self.#ident);

        if field.custom_type {
            let ty = &field.ty;
            out.append_all(quote!(as #ty))
        }

        out
    });

    quote! {
        fn update<'a, 'c: 'a>(
            &'a self,
            db: impl sqlx::Executor<'c, Database = ormx::Db> + 'a,
        ) -> #box_future<'a, sqlx::Result<()>> {
            Box::pin(async move {
                sqlx::query!(#update_sql, #( #other_arguments, )* self.#id_argument)
                    .execute(db)
                    .await?;
                Ok(())
            })
        }
    }
}

fn stream_all<D: sqlx::Database, B: Backend<D>>(table: &Table<D, B>, column_list: &str) -> TokenStream {
    let box_stream = crate::utils::box_stream();
    let all_sql = format!("SELECT {} FROM {}", column_list, table.table);

    quote! {
        fn stream_all<'a, 'c: 'a>(
            db: impl sqlx::Executor<'c, Database = ormx::Db> + 'a,
        ) -> #box_stream<'a, sqlx::Result<Self>> {
            sqlx::query_as!(Self, #all_sql)
                .fetch(db)
        }
    }
}

fn stream_all_paginated<D: sqlx::Database, B: Backend<D>>(table: &Table<D, B>, column_list: &str) -> TokenStream {
    let box_stream = crate::utils::box_stream();
    let mut bindings = B::Bindings::default();
    let all_sql = format!(
        "SELECT {} FROM {} LIMIT {} OFFSET {}",
        column_list,
        table.table,
        bindings.next().unwrap(),
        bindings.next().unwrap()
    );

    quote! {
        fn stream_all_paginated<'a, 'c: 'a>(
            db: impl sqlx::Executor<'c, Database = ormx::Db> + 'a,
            offset: i64,
            limit: i64,
        ) -> #box_stream<'a, sqlx::Result<Self>> {
            sqlx::query_as!(Self, #all_sql, limit, offset)
                .fetch(db)
        }
    }
}

fn delete<D: sqlx::Database, B: Backend<D>>(table: &Table<D, B>) -> TokenStream {
    let box_future = crate::utils::box_future();
    let id_ty = &table.id.ty;
    let delete_sql = format!(
        "DELETE FROM {} WHERE {} = {}",
        table.table,
        table.id.column(),
        B::Bindings::default().next().unwrap()
    );
    #[cfg(feature = "mysql")]
    let result_import = quote!(sqlx::mysql::MySqlQueryResult);
    #[cfg(feature = "postgres")]
    let result_import = quote!(sqlx::postgres::PgQueryResult);
    #[cfg(feature = "sqlite")]
    let result_import = quote!(sqlx::sqlite::SqliteQueryResult);

    quote! {
        fn delete_row<'a, 'c: 'a>(
            db: impl sqlx::Executor<'c, Database = ormx::Db> + 'a,
            id: #id_ty
        ) -> #box_future<'a, sqlx::Result<()>> {
            use #result_import;

            Box::pin(async move {
                let result = sqlx::query!(#delete_sql, id)
                    .execute(db)
                    .await?;
                if result.rows_affected() == 0 {
                    Err(sqlx::Error::RowNotFound)
                } else {
                    Ok(())
                }
            })
        }
    }
}
