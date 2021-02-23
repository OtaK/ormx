#![cfg(any(feature = "mysql", feature = "postgres", feature = "sqlite"))]
//! Lightweight derive macros for bringing orm-like features to sqlx.
//!
//! # Example: Table
//! ```rust,ignore
//! #[derive(ormx::Table)]
//! #[ormx(table = "users", id = user_id, insertable)]
//! struct User {
//!     #[ormx(column = "id")]
//!     user_id: u32,
//!     first_name: String,
//!     last_name: String,
//!     #[ormx(get_optional(&str))]
//!     email: String,
//!     #[ormx(default, set)]
//!     last_login: Option<NaiveDateTime>,
//! }
//! ```
//!
//! # Example: Patch
//! ```rust,ignore
//! #[derive(ormx::Patch)]
//! #[ormx(table_name = "users", table = User, id = "id")]
//! struct UpdateName {
//!     first_name: String,
//!     last_name: String,
//! }
//! ```
//!
//! # Documentation
//! See the docs of [derive(Table)](derive.Table.html) and [Patch](trait.Patch.html).

use futures::future::BoxFuture;
use futures::stream::BoxStream;
use sqlx::{Database, Executor, Result};

pub use ormx_macros::*;

#[doc(hidden)]
pub mod exports {
    pub use crate::query2::map::*;
    pub use futures;
}

pub mod args;

#[cfg(any(feature = "mysql", feature = "postgres"))]
mod query2;

#[cfg(feature = "mysql")]
pub type Db = sqlx::MySql;
#[cfg(feature = "postgres")]
pub type Db = sqlx::Postgres;
#[cfg(feature = "sqlite")]
pub type Db = sqlx::Sqlite;

/// A database table in which each row is identified by a unique ID.
#[async_trait::async_trait]
pub trait Table
where
    Self: Sized + Send + Sync + 'static,
{
    /// Type of the ID column of this table.
    type Id: 'static + Copy + Send;

    /// Returns the id of this row.
    fn id(&self) -> Self::Id;

    /// Insert a row into the database.
    async fn insert(
        db: &mut <Db as Database>::Connection,
        row: impl Insert<Table = Self>,
    ) -> Result<Self> {
        row.insert(db).await?
    }

    /// Queries the row of the given id.
    async fn get<'a, 'c: 'a>(
        db: impl Executor<'c, Database = Db> + 'a,
        id: Self::Id,
    ) -> Result<Self>;

    /// Stream all rows from this table.
    async fn stream_all<'a, 'c: 'a>(
        db: impl Executor<'c, Database = Db> + 'a,
    ) -> Result<Self>;

    async fn stream_all_paginated<'a, 'c: 'a>(
        db: impl Executor<'c, Database = Db> + 'a,
        offset: i64,
        limit: i64,
    ) -> Result<Self>;

    /// Load all rows from this table.
    async fn all<'a, 'c: 'a>(
        db: impl Executor<'c, Database = Db> + 'a,
    ) -> Result<Vec<Self>> {
        Self::stream_all(db).await?.collect()
    }

    async fn all_paginated<'a, 'c: 'a>(
        db: impl Executor<'c, Database = Db> + 'a,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Self>> {
        Self::stream_all_paginated(db, offset, limit).await?.collect()
    }
    /// Applies a patch to this row.
    async fn patch<'a, 'c: 'a, P>(
        &'a mut self,
        db: impl Executor<'c, Database = Db> + 'a,
        patch: P,
    ) -> Result<()>
    where
        P: Patch<Table = Self>,
    {
        let patch: P = patch;
        patch.patch_row(db, self.id()).await?;
        patch.apply_to(self);
        Ok(())
    }

    /// Updates all fields of this row, regardless if they have been changed or not.
    async fn update<'a, 'c: 'a>(
        &'a self,
        db: impl Executor<'c, Database = Db> + 'a,
    ) -> Result<()>;

    // Refresh this row, querying all columns from the database.
    async fn reload<'a, 'c: 'a>(
        &'a mut self,
        db: impl Executor<'c, Database = Db> + 'a,
    ) -> Result<()> {
        *self = Self::get(db, self.id()).await?;
        Ok(())
    }

    /// Delete a row from the database
    async fn delete_row<'a, 'c: 'a>(
        db: impl Executor<'c, Database = Db> + 'a,
        id: Self::Id,
    ) -> Result<()>;

    /// Deletes this row from the database.
    async fn delete<'a, 'c: 'a>(
        self,
        db: impl Executor<'c, Database = Db> + 'a,
    ) -> Result<()> {
        Self::delete_row(db, self.id()).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
pub trait FindableTable: Table {
    async fn find_all(args: QueryArgs) -> Vec<Self> {
        todo!()
    }
}

/// A type which can be used to "patch" a row, updating multiple fields at once.
pub trait Patch
where
    Self: Sized + Send + Sync + 'static,
{
    type Table: Table;

    /// Applies the data of this patch to the given entity.
    /// This does not persist the change in the database.
    fn apply_to(self, entity: &mut Self::Table);

    /// Applies this patch to a row in the database.
    fn patch_row<'a, 'c: 'a>(
        &'a self,
        db: impl Executor<'c, Database = Db> + 'a,
        id: <Self::Table as Table>::Id,
    ) -> BoxFuture<'a, Result<()>>;
}

/// A type which can be inserted as a row into the database.
pub trait Insert
where
    Self: Sized + Send + Sync + 'static,
{
    type Table: Table;

    /// Insert a row into the database, returning the inserted row.
    fn insert(self, db: &mut <Db as Database>::Connection) -> BoxFuture<Result<Self::Table>>;
}
