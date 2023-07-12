use std::fmt::Debug;

use quote::ToTokens;
use syn::parse::Parse;

#[derive(Debug)]
pub enum Query {
    Select {
        columns: Vec<Column>,
        table: String,
        where_clause: Option<Where>,
        group_by: Vec<String>,
        order_by: Vec<(String, Ordering)>,
        limit: Option<u128>,
        offset: Option<u128>,
        joins: Vec<Join>,
        lock: Option<ForLock>,
    },
    Update {
        columns: Vec<(String, Expr)>,
        table: String,
        where_clause: Option<Where>,
        returning: Vec<Column>,
    },
    Insert {
        columns: Vec<(String, Expr)>,
        table: String,
        returning: Vec<Column>,
    },
    Delete {
        table: String,
        where_clause: Option<Where>,
        returning: Vec<Column>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum Column {
    All,
    Named(String, Option<String>),
}

impl PartialEq<&str> for Column {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Column::All => false,
            Column::Named(name, _) => name == other,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ForLock {
    Update,
    Share,
}

pub struct Expr(pub syn::Expr);

impl Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Expr")
            .field(&self.0.to_token_stream())
            .finish()
    }
}

#[derive(Debug)]
pub struct Conditional<T>
where
    T: Parse + Debug,
{
    pub value: T,
    pub condition: Option<Expr>,
}

#[derive(Debug)]
pub struct BoolWhere {
    pub op: BoolOp,
    pub conditions: Vec<Conditional<Where>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BoolOp {
    And,
    Or,
}

#[derive(Debug)]
pub enum Where {
    Column(Conditional<ColumnCondition>),
    BoolWhere(BoolWhere),
}

#[derive(Debug)]
pub struct ColumnCondition {
    pub column: String,
    pub op: WhereOp,
    pub value: Expr,
}

#[derive(Debug, PartialEq, Eq)]
pub enum WhereOp {
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
    Like,
    NotLike,
    In,
    NotIn,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Ordering {
    Asc,
    Desc,
}

#[derive(Debug)]
pub struct Join {
    pub table: String,
    pub on: Expr,
    pub join_type: JoinType,
    pub outer: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}
