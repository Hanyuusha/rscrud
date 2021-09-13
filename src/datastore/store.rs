use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use log::info;
use mockall::automock;
use uuid::Uuid;

use crate::datastore::models::Post;
use crate::datastore::schema::posts;
use crate::datastore::schema::posts::columns::*;

type PgPool = Pool<ConnectionManager<PgConnection>>;

type Conn = PooledConnection<ConnectionManager<PgConnection>>;

embed_migrations!();

#[automock]
pub trait DataStoreService: Send + Sync {
    fn get_posts(&self, is_published: &bool) -> QueryResult<Vec<Post>>;
    fn create_post(&self, post_title: &str, post_body: &str) -> QueryResult<Post>;
    fn update_post(&self, post: &Post) -> QueryResult<usize>;
    fn delete_post(&self, post_id: uuid::Uuid) -> QueryResult<usize>;
    fn get_post(&self, post_id: uuid::Uuid) -> QueryResult<Post>;
    fn run_migrations(&self);
}

pub struct Datastore {
    pool: PgPool,
}

impl Datastore {
    pub fn new(pg_dsn: String) -> impl DataStoreService {
        let manager = ConnectionManager::<PgConnection>::new(pg_dsn);
        let pool = Pool::builder()
            .build(manager)
            .expect("Failed to create pool");

        Self { pool }
    }

    fn get_connection(&self) -> Conn {
        self.pool.get().expect("Can't get DB connection")
    }
}

impl DataStoreService for Datastore {
    fn get_posts(&self, is_published: &bool) -> QueryResult<Vec<Post>> {
        let conn = self.get_connection();
        let posts_query = posts::table.filter(published.eq(is_published)).load(&conn);
        posts_query
    }

    fn create_post(&self, post_title: &str, post_body: &str) -> QueryResult<Post> {
        let conn = self.get_connection();
        let new_post = Post {
            id: Uuid::new_v4(),
            title: String::from(post_title),
            body: String::from(post_body),
            published: true,
        };

        diesel::insert_into(posts::table)
            .values(&new_post)
            .get_result(&conn)
    }

    fn update_post(&self, post: &Post) -> QueryResult<usize> {
        let conn = self.get_connection();
        let target = posts::table.filter(id.eq(post.id));

        diesel::update(target)
            .set((
                title.eq(&post.title),
                body.eq(&post.body),
                published.eq(&post.published),
            ))
            .execute(&conn)
    }

    fn delete_post(&self, post_id: uuid::Uuid) -> QueryResult<usize> {
        let conn = self.get_connection();
        let target = posts::table.filter(id.eq(post_id));
        diesel::delete(target).execute(&conn)
    }

    fn get_post(&self, post_id: uuid::Uuid) -> QueryResult<Post> {
        let conn = self.get_connection();
        let target = posts::table.filter(id.eq(post_id));
        target.first::<Post>(&conn)
    }

    fn run_migrations(&self) {
        info!("Running migrations");
        let conn = self.get_connection();
        embedded_migrations::run(&conn).expect("Failed to run database migrations");
    }
}
