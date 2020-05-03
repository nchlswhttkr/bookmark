use super::schema::*;
use chrono::NaiveDateTime;

#[derive(Identifiable, Queryable)]
#[table_name = "bookmark"]
pub struct Bookmark {
    pub id: i32,
    pub url: String,
    pub name: String,
    pub created: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "bookmark"]
pub struct BookmarkToInsert<'a> {
    pub url: &'a str,
    pub name: Option<&'a str>,
}

#[derive(Identifiable, Queryable, Associations)]
#[belongs_to(Bookmark)]
#[table_name = "tag"]
pub struct Tag {
    pub id: i32,
    pub bookmark_id: i32,
    pub value: String,
}

#[derive(Insertable)]
#[table_name = "tag"]
pub struct TagToInsert {
    pub value: String,
    pub bookmark_id: i32,
}
