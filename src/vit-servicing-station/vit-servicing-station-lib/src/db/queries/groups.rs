use crate::db::{models::groups::Group, schema::groups, DbConnection};
use diesel::{Insertable, QueryResult, RunQueryDsl};

pub fn batch_insert(
    batch_groups: &[<Group as Insertable<groups::table>>::Values],
    db_conn: &DbConnection,
) -> QueryResult<usize> {
    diesel::insert_into(groups::table)
        .values(batch_groups)
        .execute(db_conn)
}
