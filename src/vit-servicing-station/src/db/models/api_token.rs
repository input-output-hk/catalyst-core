use diesel::Queryable;

#[allow(dead_code)]
#[derive(Queryable)]
#[diesel(table = "api_tokens")]
pub struct APIToken {
    token: Vec<u8>,
    creation_time: String,
    expire_time: String,
}
