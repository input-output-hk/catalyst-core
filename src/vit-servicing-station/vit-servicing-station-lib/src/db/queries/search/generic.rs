use diesel::{
    dsl::Like,
    expression::bound::Bound,
    query_dsl::{methods::FilterDsl, LoadQuery},
    sql_types::Text,
    Expression, TextExpressionMethods,
};

use crate::{db::DbPoolConn, v0::errors::HandleError};

type SqlStr = Text;

pub fn execute_search<T, C, M, P>(
    table: T,
    col: C,
    query: &str,
    conn: &DbPoolConn,
) -> Result<Vec<M>, HandleError>
where
    T: FilterDsl<Like<C, Bound<SqlStr, String>>, Output = P>,
    C: Expression<SqlType = SqlStr> + TextExpressionMethods,
    P: LoadQuery<DbPoolConn, M>,
{
    search_query(table, col, query)
        .load(conn)
        .map_err(|_| HandleError::InternalError("Error searching".to_string()))
}

pub fn search_query<T, C>(
    table: T,
    col: C,
    query: &str,
) -> <T as FilterDsl<Like<C, Bound<SqlStr, String>>>>::Output
where
    T: FilterDsl<Like<C, Bound<SqlStr, String>>>,
    C: Expression<SqlType = SqlStr> + TextExpressionMethods,
{
    let p = predicate(col, query);
    table.filter(p)
}

fn predicate<C>(col: C, query: &str) -> Like<C, Bound<SqlStr, String>>
where
    C: Expression<SqlType = SqlStr> + TextExpressionMethods,
{
    let query = format!("%{query}%");
    col.like(query)
}
