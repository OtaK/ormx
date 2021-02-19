#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(rename_all = "UPPERCASE")]
pub enum IndexHint {
    Use,
    Force,
    Ignore,
}

#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(rename_all = "UPPERCASE")]
pub enum TableHints {
    NoLock,
    ReadUncommited,
    UpdLock,
    RepeatableRead,
    Serializable,
    ReadCommited,
    TabLock,
    TabLockX,
    PagLock,
    RowLock,
    NoWait,
    ReadPast,
    XLock,
    Snapshot,
    NoExpand,
}

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(rename_all = "UPPERCASE")]
pub enum QueryType {
    Select,
    Insert,
    Update,
    BulkUpdate,
    BulkDelete,
    Delete,
    Upsert,
    Version,
    ShowTables,
    ShowIndexes,
    Describe,
    Raw,
    ForeignKeys,
    ShowConstraints,
}
