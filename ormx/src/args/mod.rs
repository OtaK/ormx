mod op;
use std::hash::Hash;

pub use self::op::*;

mod hints;
pub use self::hints::*;

#[derive(Debug, Clone)]
pub enum WhereValue<'a> {
    Literal(Box<dyn sqlx::Encode<'a, sqlx::Any>>),
    Alternatives(Vec<Box<dyn sqlx::Encode<'a, sqlx::Any>>>),
}

#[derive(Debug, Clone)]
pub struct AttributesValue {
    pub list: Vec<String>,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AssociationArg {}

#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(rename_all = "UPPERCASE")]
pub enum OrderType {
    Desc,
    Asc,
}

pub type WhereOpts<'a> = std::collections::HashMap<String, (Operator, WhereValue<'a>)>;

#[derive(Debug, Default, Clone)]
pub struct QueryArgs<'a> {
    where_opts: WhereOpts<'a>,
    attributes: AttributesValue,
    include: Vec<AssociationArg>,
    order: Vec<(String, OrderType)>,
    transaction: Option<sqlx::Transaction<'a, sqlx::Any>>,
    having: WhereOpts<'a>,
}
