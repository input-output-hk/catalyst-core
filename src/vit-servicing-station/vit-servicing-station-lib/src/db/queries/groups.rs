use crate::{
    db::{models::groups::Group, schema::groups, DbConnection},
    execute_q,
};
use diesel::{Insertable, QueryResult};

pub fn batch_insert(
    batch_groups: &[<Group as Insertable<groups::table>>::Values],
    db_conn: &DbConnection,
) -> QueryResult<usize> {
    execute_q!(
        db_conn,
        diesel::insert_into(groups::table).values(batch_groups)
    )
}
