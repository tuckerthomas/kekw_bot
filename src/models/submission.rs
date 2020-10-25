use crate::schema::submissions;
use crate::models::period::Period;

#[derive(Identifiable, Queryable, Associations, AsChangeset, PartialEq, Clone, Debug)]
#[belongs_to(Period)]
pub struct Submission {
    pub id: i32,
    pub dis_user_id: String,
    pub title: String,
    pub link: String,
    pub period_id: i32
}

#[derive(Insertable, Associations, PartialEq, Debug)]
#[belongs_to(Period)]
#[table_name="submissions"]
pub struct NewSubmission<'a> {
    pub dis_user_id: &'a str,
    pub title: &'a str,
    pub link: &'a str,
    pub period_id: i32
}