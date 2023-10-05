use crate::db::{models::goals::InsertGoal, schema::goals, DbConnection};
use diesel::{QueryResult, RunQueryDsl};

pub fn batch_insert(goals: Vec<InsertGoal>, db_conn: &DbConnection) -> QueryResult<usize> {
    diesel::insert_into(goals::table)
        .values(goals)
        .execute(db_conn)
}
