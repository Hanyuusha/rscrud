use uuid::Uuid;

use super::schema::posts;

#[derive(Debug, Queryable, Insertable)]
pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub published: bool,
}
