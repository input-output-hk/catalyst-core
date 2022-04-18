use diesel::{
    dsl::{Filter, Like},
    expression::{bound::Bound, AsExpression},
    query_dsl::{methods::FilterDsl, LoadQuery},
    r2d2::{ConnectionManager, PooledConnection},
    sql_types::VarChar,
    Column, Expression, RunQueryDsl, SqliteConnection, Table, TextExpressionMethods,
};

type SqlStr = VarChar;

pub fn search_query<T, C>(
    table: T,
    col: C,
    query: String,
) -> <T as FilterDsl<Like<C, Bound<SqlStr, String>>>>::Output
where
    T: FilterDsl<Like<C, Bound<SqlStr, String>>>,
    C: Expression<SqlType = SqlStr> + TextExpressionMethods,
{
    let p = predicate(col, query);
    table.filter(p)
}

fn predicate<C>(col: C, query: String) -> Like<C, Bound<SqlStr, String>>
where
    C: Expression<SqlType = SqlStr> + TextExpressionMethods,
{
    let query = format!("%{query}%");
    col.like(query)
}
