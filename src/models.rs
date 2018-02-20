use super::schema::listings;

#[derive(Insertable)]
#[table_name = "listings"]
pub struct NewAdServer<'a> {
    pub domain: &'a str,
}

#[derive(Queryable)]
pub struct AdServer {
    pub id: i32,
    pub domain: String,
}
