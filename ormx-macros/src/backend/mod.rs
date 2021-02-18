use std::borrow::Cow;

use proc_macro2::TokenStream;

use crate::patch::Patch;
use crate::table::Table;

mod common;
#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "postgres")]
mod postgres;

#[cfg(feature = "mysql")]
pub type Implementation = mysql::MySqlBackend;
#[cfg(feature = "postgres")]
pub type Implementation = postgres::PgBackend;
#[cfg(feature = "sqlite")]
compile_error!("sqlite is currently not supported");

pub trait Backend<D: sqlx::Database>: Sized + Clone {
    const QUOTE: char;
    /// TODO: benchmark HashSet vs linear search
    const RESERVED_IDENTS: &'static [&'static str];

    type TypeInfo: sqlx::TypeInfo;

    type Bindings: Iterator<Item = Cow<'static, str>> + Default;

    /// Generate an `impl <Table>` block, containing getter methods
    fn impl_getters(table: &Table<D, Self>) -> TokenStream {
        common::getters::<D, Self>(table)
    }

    /// Generate an `impl <Table>` block, containing setter methods
    fn impl_setters(table: &Table<D, Self>) -> TokenStream {
        common::setters::<D, Self>(table)
    }

    /// Generate an `impl Table for <Table>` block
    fn impl_table(table: &Table<D, Self>) -> TokenStream {
        common::impl_table::<D, Self>(table)
    }

    /// Implement [Insert] for the helper struct for inserting
    fn impl_insert(table: &Table<D, Self>) -> TokenStream;

    /// Generate a helper struct for inserting
    fn insert_struct(table: &Table<D, Self>) -> TokenStream {
        common::insert_struct(table)
    }

    /// Implement [Patch]
    fn impl_patch(patch: &Patch) -> TokenStream {
        common::impl_patch::<D, Self>(patch)
    }
}
