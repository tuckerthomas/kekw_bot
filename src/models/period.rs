use crate::schema::periods;

#[derive(Identifiable, Queryable, AsChangeset, Clone, Copy)]
#[changeset_options(treat_none_as_null="true")]
pub struct Period {
    pub id: i32,
    pub start_day: i64,
    pub end_day: Option<i64>
}

#[derive(Insertable)]
#[table_name="periods"]
pub struct NewPeriod {
    pub start_day: i64
}