use diesel::{
    dsl::Like, expression::bound::Bound, query_dsl::methods::FilterDsl, sql_types::Text,
    Expression, TextExpressionMethods,
};

type SqlStr = Text;

/// A macro is needed instead of a regular function due to cyclic trait bounds
///
/// It's possible some extra trait bounds would resolve this, but it seems non-trivial
/// 
/// Example usage:
/// ```ignore
/// fn search_funds_by_name() {
///   let conn = pool.get().unwrap();
///   db_search!(&conn => search in funds.fund_name)
/// }
/// ```
#[macro_export]
macro_rules! db_search {
    ($conn:expr => $query:ident in $table:ident.$col:ident) => {{
       let query = search_query($table, $col, $query);
       query.load($conn)
    }};
}

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
