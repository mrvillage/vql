use std::fmt::Debug;

use syn::{
    braced,
    parse::{Parse, ParseStream},
    token::Brace,
    Ident, LitInt, Result, Token,
};

use crate::structs::{
    BoolOp, BoolWhere, Column, ColumnCondition, Conditional, Expr, ForLock, Join, JoinType,
    Ordering, Query, Where, WhereOp,
};

mod kw {
    use syn::custom_keyword;

    custom_keyword!(SELECT);
    custom_keyword!(INSERT);
    custom_keyword!(UPDATE);
    custom_keyword!(DELETE);
    custom_keyword!(FROM);
    custom_keyword!(WHERE);
    custom_keyword!(AND);
    custom_keyword!(OR);
    custom_keyword!(IN);
    custom_keyword!(LIKE);
    custom_keyword!(NOT);
    custom_keyword!(GROUP);
    custom_keyword!(BY);
    custom_keyword!(ORDER);
    custom_keyword!(ASC);
    custom_keyword!(DESC);
    custom_keyword!(LIMIT);
    custom_keyword!(OFFSET);
    custom_keyword!(JOIN);
    custom_keyword!(ON);
    custom_keyword!(INNER);
    custom_keyword!(OUTER);
    custom_keyword!(LEFT);
    custom_keyword!(RIGHT);
    custom_keyword!(FULL);
    custom_keyword!(FOR);
    custom_keyword!(SHARE);
    custom_keyword!(SET);
    custom_keyword!(RETURNING);
    custom_keyword!(AS);
    custom_keyword!(INTO);
}

fn parse_where(input: ParseStream) -> Result<Option<Where>> {
    Ok(if input.peek(kw::WHERE) {
        input.parse::<kw::WHERE>()?;
        Some(input.parse()?)
    } else {
        None
    })
}

fn parse_returning(input: ParseStream) -> Result<Vec<Column>> {
    Ok(if input.peek(kw::RETURNING) {
        input.parse::<kw::RETURNING>()?;
        let content;
        braced!(content in input);
        content
            .parse_terminated(Column::parse, Token![,])?
            .into_iter()
            .collect()
    } else {
        vec![]
    })
}

fn parse_semicolon(input: ParseStream) -> Result<()> {
    if input.peek(Token![;]) {
        input.parse::<Token![;]>()?;
    }

    Ok(())
}
impl Parse for Query {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::SELECT) {
            input.parse::<kw::SELECT>()?;

            let content;
            braced!(content in input);
            let columns = content
                .parse_terminated(Column::parse, Token![,])?
                .into_iter()
                .collect();

            input.parse::<kw::FROM>()?;

            let table = input.parse::<Ident>()?.to_string();

            let joins = if input.peek(Brace) {
                let content;
                braced!(content in input);
                content
                    .parse_terminated(Join::parse, Token![,])?
                    .into_iter()
                    .collect()
            } else {
                vec![]
            };

            let where_clause = parse_where(input)?;

            let group_by = if input.peek(kw::GROUP) && input.peek2(kw::BY) {
                input.parse::<kw::GROUP>()?;
                input.parse::<kw::BY>()?;
                let content;
                braced!(content in input);
                content
                    .parse_terminated(|input| Ok(input.parse::<Ident>()?.to_string()), Token![,])?
                    .into_iter()
                    .collect()
            } else {
                vec![]
            };

            let order_by = if input.peek(kw::ORDER) && input.peek2(kw::BY) {
                input.parse::<kw::ORDER>()?;
                input.parse::<kw::BY>()?;
                let content;
                braced!(content in input);
                content
                    .parse_terminated(
                        |input| {
                            let column = input.parse::<Ident>()?.to_string();
                            let ordering = input.parse::<Ordering>()?;
                            Ok((column, ordering))
                        },
                        Token![,],
                    )?
                    .into_iter()
                    .collect()
            } else {
                vec![]
            };

            let limit = if input.peek(kw::LIMIT) {
                input.parse::<kw::LIMIT>()?;
                Some(input.parse::<LitInt>()?.base10_parse()?)
            } else {
                None
            };

            let offset = if input.peek(kw::OFFSET) {
                input.parse::<kw::OFFSET>()?;
                Some(input.parse::<LitInt>()?.base10_parse()?)
            } else {
                None
            };

            let lock = if input.peek(kw::FOR) {
                input.parse::<kw::FOR>()?;
                Some(input.parse::<ForLock>()?)
            } else {
                None
            };

            parse_semicolon(input)?;

            Ok(Self::Select {
                columns,
                table,
                where_clause,
                group_by,
                order_by,
                limit,
                offset,
                joins,
                lock,
            })
        } else if lookahead.peek(kw::INSERT) {
            input.parse::<kw::INSERT>()?;

            let content;
            braced!(content in input);
            let columns = content
                .parse_terminated(
                    |input| {
                        let column = input.parse::<Ident>()?.to_string();
                        input.parse::<Token![=]>()?;
                        let expr = input.parse()?;
                        Ok((column, expr))
                    },
                    Token![,],
                )?
                .into_iter()
                .collect();

            input.parse::<kw::INTO>()?;

            let table = input.parse::<Ident>()?.to_string();

            let returning = parse_returning(input)?;

            parse_semicolon(input)?;

            Ok(Self::Insert {
                columns,
                table,
                returning,
            })
        } else if lookahead.peek(kw::UPDATE) {
            input.parse::<kw::UPDATE>()?;

            let table = input.parse::<Ident>()?.to_string();

            input.parse::<kw::SET>()?;

            let content;
            braced!(content in input);
            let columns = content
                .parse_terminated(
                    |input| {
                        let column = input.parse::<Ident>()?.to_string();
                        input.parse::<Token![=]>()?;
                        let expr = input.parse()?;
                        Ok((column, expr))
                    },
                    Token![,],
                )?
                .into_iter()
                .collect();

            let where_clause = if input.peek(kw::WHERE) {
                input.parse::<kw::WHERE>()?;
                Some(input.parse()?)
            } else {
                None
            };

            let returning = parse_returning(input)?;

            parse_semicolon(input)?;

            Ok(Self::Update {
                columns,
                table,
                where_clause,
                returning,
            })
        } else if lookahead.peek(kw::DELETE) {
            input.parse::<kw::DELETE>()?;

            input.parse::<kw::FROM>()?;

            let table = input.parse::<Ident>()?.to_string();

            let where_clause = parse_where(input)?;

            let returning = parse_returning(input)?;

            parse_semicolon(input)?;

            Ok(Self::Delete {
                table,
                where_clause,
                returning,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl<T> Parse for Conditional<T>
where
    T: Parse + Debug,
{
    fn parse(input: ParseStream) -> Result<Self> {
        let value = input.parse()?;
        let condition = if input.peek(Token![if]) {
            input.parse::<Token![if]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        Ok(Self { value, condition })
    }
}

impl Parse for BoolWhere {
    fn parse(input: ParseStream) -> Result<Self> {
        let op = input.parse()?;
        input.parse::<Token![:]>()?;
        let content;
        braced!(content in input);
        let conditions = content
            .parse_terminated(Conditional::<Where>::parse, Token![,])?
            .into_iter()
            .collect();
        Ok(Self { op, conditions })
    }
}

impl Parse for BoolOp {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::AND) {
            input.parse::<kw::AND>()?;
            Ok(BoolOp::And)
        } else if lookahead.peek(kw::OR) {
            input.parse::<kw::OR>()?;
            Ok(BoolOp::Or)
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for Where {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if (lookahead.peek(kw::AND) || lookahead.peek(kw::OR)) && input.peek2(Token![:]) {
            Ok(Where::BoolWhere(input.parse()?))
        } else if lookahead.peek(Ident) {
            Ok(Where::Column(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for ColumnCondition {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            column: input.parse::<Ident>()?.to_string(),
            op: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl Parse for WhereOp {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![==]) {
            input.parse::<Token![==]>()?;
            Ok(WhereOp::Eq)
        } else if lookahead.peek(Token![!=]) {
            input.parse::<Token![!=]>()?;
            Ok(WhereOp::Ne)
        } else if lookahead.peek(Token![>=]) {
            input.parse::<Token![>=]>()?;
            Ok(WhereOp::Ge)
        } else if lookahead.peek(Token![>]) {
            input.parse::<Token![>]>()?;
            Ok(WhereOp::Gt)
        } else if lookahead.peek(Token![<=]) {
            input.parse::<Token![<=]>()?;
            Ok(WhereOp::Le)
        } else if lookahead.peek(Token![<]) {
            input.parse::<Token![<]>()?;
            Ok(WhereOp::Lt)
        } else if lookahead.peek(kw::LIKE) {
            input.parse::<kw::LIKE>()?;
            Ok(WhereOp::Like)
        } else if lookahead.peek(kw::NOT) && input.peek2(kw::LIKE) {
            input.parse::<kw::NOT>()?;
            input.parse::<kw::LIKE>()?;
            Ok(WhereOp::NotLike)
        } else if lookahead.peek(kw::IN) {
            input.parse::<kw::IN>()?;
            Ok(WhereOp::In)
        } else if lookahead.peek(kw::NOT) && input.peek2(kw::IN) {
            input.parse::<kw::NOT>()?;
            input.parse::<kw::IN>()?;
            Ok(WhereOp::NotIn)
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for Ordering {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::ASC) {
            input.parse::<kw::ASC>()?;
            Ok(Ordering::Asc)
        } else if lookahead.peek(kw::DESC) {
            input.parse::<kw::DESC>()?;
            Ok(Ordering::Desc)
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for Join {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        let join_type = if lookahead.peek(kw::INNER) {
            input.parse::<kw::INNER>()?;
            JoinType::Inner
        } else if lookahead.peek(kw::LEFT) {
            input.parse::<kw::LEFT>()?;
            JoinType::Left
        } else if lookahead.peek(kw::RIGHT) {
            input.parse::<kw::RIGHT>()?;
            JoinType::Right
        } else if lookahead.peek(kw::FULL) {
            input.parse::<kw::FULL>()?;
            JoinType::Full
        } else {
            Err(lookahead.error())?
        };
        let outer = if input.peek(kw::OUTER) {
            input.parse::<kw::OUTER>()?;
            true
        } else {
            false
        };
        input.parse::<kw::JOIN>()?;
        let table = input.parse::<Ident>()?.to_string();
        input.parse::<kw::ON>()?;
        let on = input.parse()?;
        Ok(Self {
            table,
            on,
            join_type,
            outer,
        })
    }
}

impl Parse for Column {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Ident) {
            let name = input.parse::<Ident>()?.to_string();
            let alias = if input.peek(kw::AS) {
                input.parse::<kw::AS>()?;
                Some(input.parse::<Ident>()?.to_string())
            } else {
                None
            };
            Ok(Column::Named(name, alias))
        } else if lookahead.peek(Token![*]) {
            input.parse::<Token![*]>()?;
            Ok(Column::All)
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for Expr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Expr(input.parse()?))
    }
}

impl Parse for ForLock {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if input.peek(kw::UPDATE) {
            input.parse::<kw::UPDATE>()?;
            Ok(ForLock::Update)
        } else if lookahead.peek(kw::SHARE) {
            input.parse::<kw::SHARE>()?;
            Ok(ForLock::Share)
        } else {
            Err(lookahead.error())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_select() {
        let query = syn::parse_str::<Query>("SELECT {a, b, c} FROM table").unwrap();
        if let Query::Select {
            columns,
            table,
            where_clause,
            ..
        } = &query
        {
            println!("{:?}", &query);
            assert_eq!(columns.len(), 3);
            assert_eq!(columns[0], "a");
            assert_eq!(columns[1], "b");
            assert_eq!(columns[2], "c");
            assert_eq!(table, "table");
            assert!(where_clause.is_none());
        } else {
            panic!("expected select query");
        }
    }

    #[test]
    fn test_select_with_where() {
        let query = syn::parse_str::<Query>(
            "SELECT {a, b, c} FROM table WHERE AND: {a == 1, OR: {b NOT LIKE 2, c LIKE 3 if d == 4}}",
        )
        .unwrap();
        if let Query::Select {
            columns,
            table,
            where_clause,
            ..
        } = &query
        {
            println!("{:?}", &query);
            assert_eq!(columns.len(), 3);
            assert_eq!(columns[0], "a");
            assert_eq!(columns[1], "b");
            assert_eq!(columns[2], "c");
            assert_eq!(table, "table");
            assert!(where_clause.is_some());
        } else {
            panic!("expected select query");
        }
    }

    #[test]
    fn test_full_select() {
        let query = syn::parse_str::<Query>(
            "SELECT {a, b, c} FROM table WHERE AND: {a == 1, OR: {b NOT LIKE 2, c LIKE 3 if d == 4}} GROUP BY {a, b} ORDER BY {a ASC, b DESC} LIMIT 10 OFFSET 20",
        )
        .unwrap();
        if let Query::Select {
            columns,
            table,
            where_clause,
            group_by,
            order_by,
            limit,
            offset,
            ..
        } = &query
        {
            println!("{:?}", &query);
            assert_eq!(columns.len(), 3);
            assert_eq!(columns[0], "a");
            assert_eq!(columns[1], "b");
            assert_eq!(columns[2], "c");
            assert_eq!(table, "table");
            assert!(where_clause.is_some());
            assert_eq!(group_by.len(), 2);
            assert_eq!(group_by[0], "a");
            assert_eq!(group_by[1], "b");
            assert_eq!(order_by.len(), 2);
            assert_eq!(order_by[0].0, "a");
            assert_eq!(order_by[0].1, Ordering::Asc);
            assert_eq!(order_by[1].0, "b");
            assert_eq!(order_by[1].1, Ordering::Desc);
            assert_eq!(*limit, Some(10));
            assert_eq!(*offset, Some(20));
        } else {
            panic!("expected select query");
        }
    }

    #[test]
    fn test_joins() {
        let query = syn::parse_str::<Query>(
            "SELECT {a, b, c} FROM table {INNER JOIN table2 ON a == b, FULL OUTER JOIN table2 ON c == d}",
        )
        .unwrap();
        if let Query::Select { .. } = query {
            println!("{:?}", &query);
        } else {
            panic!("expected select query");
        }
    }

    #[test]
    fn test_update() {
        let query = syn::parse_str::<Query>(
            "UPDATE table SET {a = b, c = 22} WHERE c == d RETURNING {a, b, *}",
        )
        .unwrap();
        if let Query::Update { .. } = query {
            println!("{:?}", &query);
        } else {
            panic!("expected update query");
        }
    }

    #[test]
    fn test_insert() {
        let query =
            syn::parse_str::<Query>("INSERT {a = b, c = 22} INTO table RETURNING {a, b, *}")
                .unwrap();
        if let Query::Insert { .. } = query {
            println!("{:?}", &query);
        } else {
            panic!("expected insert query");
        }
    }

    #[test]
    fn test_delete() {
        let query =
            syn::parse_str::<Query>("DELETE FROM table WHERE a == b RETURNING {a, b, *}").unwrap();
        if let Query::Delete { .. } = query {
            println!("{:?}", &query);
        } else {
            panic!("expected delete query");
        }
    }
}
