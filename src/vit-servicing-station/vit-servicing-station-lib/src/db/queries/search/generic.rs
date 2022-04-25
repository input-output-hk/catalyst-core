use diesel::{
    dsl::Like,
    expression::bound::Bound,
    query_dsl::{
        methods::{FilterDsl, OrderDsl},
        LoadQuery,
    },
    r2d2::{ConnectionManager, PooledConnection},
    sql_types::Text,
    Expression, TextExpressionMethods,
};

use crate::{db::DbConnection, v0::errors::HandleError};

type SqlStr = Text;

diesel::no_arg_sql_function!(RANDOM, (), "Represents the sql RANDOM() function");

// Temp1 and Temp2 are intermediates set by the implementation
//
// They are needed to mitigate a compiler bug to do with overly restrictive bounds on the types of
// intermediate values
//
// They "bubble up" the constraint to the caller, while giving the compiler flexibility to reject
// if it can't find values for Temp1 and Temp2 that match
pub fn search<Table, Column, Order, Model, Temp1, Temp2>(
    table: Table,
    col: Column,
    order: Order,
    query: &str,
    conn: &PooledConnection<ConnectionManager<DbConnection>>,
) -> Result<Vec<Model>, HandleError>
where
    Table: FilterDsl<Like<Column, Bound<SqlStr, String>>, Output = Temp1>,
    Column: Expression<SqlType = SqlStr> + TextExpressionMethods,
    Order: Expression,
    Temp1: OrderDsl<Order, Output = Temp2>,
    Temp2: LoadQuery<PooledConnection<ConnectionManager<DbConnection>>, Model>,
{
    let query = format!("%{query}%");
    table
        .filter(col.like(query))
        .order(order)
        .load(conn)
        .map_err(|_| HandleError::InternalError("Error searching".to_string()))
}
