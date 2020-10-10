#[derive(Queryable, Clone)]
pub struct MovieSub {
    pub id: i32,
    pub dis_user_id: i32,
    pub title: String,
    pub link: String
}

use crate::schema::moviesubs;

#[derive(Insertable)]
#[table_name="moviesubs"]
pub struct NewMovieSub<'a> {
    pub dis_user_id: i32,
    pub title: &'a str,
    pub link: &'a str
}