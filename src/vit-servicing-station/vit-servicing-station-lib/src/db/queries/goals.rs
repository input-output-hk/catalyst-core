use crate::{
    db::{models::goals::InsertGoal, schema::goals, DbConnection},
    execute_q,
};
use diesel::QueryResult;

pub fn batch_insert(goals: Vec<InsertGoal>, db_conn: &DbConnection) -> QueryResult<usize> {
    execute_q!(db_conn, diesel::insert_into(goals::table).values(goals))
}
