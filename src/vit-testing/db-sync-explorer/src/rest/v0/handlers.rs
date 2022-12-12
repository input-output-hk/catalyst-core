use bb8::{Pool};
use crate::db::{Meta};
use axum::{

    extract::{ State},
    http::{ StatusCode},


};
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;
type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;
use diesel::RunQueryDsl;


pub async fn get_meta_information(
    State(pool): State<ConnectionPool>,
) -> Result<String, (StatusCode, String)> {
    let mut conn = pool.get().await.map_err(internal_error)?;

  //  use crate::db::schema::meta;

  //  let meta = meta::table.load::<Meta>(&mut *conn).map_err(internal_error)?;

    let row = conn
        .query_one("select 1 + 1", &[])
        .await
        .map_err(internal_error)?;

    let two: i32 = row.try_get(0).map_err(internal_error)?;

    Ok(two.to_string())
  /*  if meta.is_empty() || meta.len() > 2 {
        Err((StatusCode::INTERNAL_SERVER_ERROR,"data inconsistency.. expected only one meta record".to_string()))
    } else {
        Ok(meta[0].clone())
    }*/
}

fn internal_error<E>(err: E) -> (StatusCode, String)
    where
        E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())

}
/*
let results: Vec<BehindDuration> = db.exec(move |conn| behind().load(conn)).unwrap();
let time : DateTime<Utc> = results[0].behind_by.into();
let diff: chrono::Duration = chrono::offset::Utc::now() - time;
println!("{}",diff);

let results: Vec<TransactionConfirmationRow> = db.exec(move |conn| hash("7b27ea78e32fdb4522ad63495a0f89289663435f3904ff5d12c529a47c8a37b8").load::<TransactionConfirmationRow>(conn)).unwrap();
let results: Vec<TransactionConfirmation> = results.into_iter().map(TransactionConfirmation::from).collect();
println!("{results:?}");

Ok(())*/