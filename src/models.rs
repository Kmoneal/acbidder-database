use super::schema::listings;
use super::schema::requests;
use super::schema::responses;

//listings
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

//requests
#[derive(Insertable)]
#[table_name = "requests"]
pub struct NewRequest<'a> {
    pub publisher: &'a str,
    pub userquality: i32,
}

#[derive(Queryable)]
pub struct Request {
    pub id: i32,
    pub publisher: String,
    pub userquality: i32,
}

//responses
#[derive(Insertable)]
#[table_name = "responses"]
pub struct NewResponse<'a> {
    pub publisher: &'a str,
}

#[derive(Queryable)]
pub struct Response {
    pub id: i32,
    pub publisher: String,
}