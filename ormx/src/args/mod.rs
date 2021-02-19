mod op;
use std::hash::Hash;

pub use self::op::*;

mod hints;
pub use self::hints::*;

#[derive(Debug)]
pub enum WhereValue {
    // Switch to sqlx::Encode
    Literal(Box<dyn sqlx::Encode>),
    Alternatives(Vec<Box<dyn sqlx::Encode>>),
}

#[derive(Debug, Clone)]
pub struct AttributesValue {
    pub list: Vec<String>,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

#[derive(Debug)]
pub struct AssociationArg {}

#[derive(Debug, sqlx::Type)]
#[sqlx(rename_all = "UPPERCASE")]
pub enum OrderType {
    Desc,
    Asc,
}

#[derive(Debug, Clone)]
pub struct QueryArgs {
    where_opts: std::collections::HashMap<String, (Operator, WhereValue)>,
    attributes: AttributesValue,
    include: Vec<AssociationArg>,
    // Maybe btreemap? We need to keep order of insertion
    order: Vec<(String, OrderType)>,
}
