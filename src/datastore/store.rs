use crate::datastore::models::Post;
use crate::datastore::schema::posts;
use crate::datastore::schema::posts::columns::*;
use async_trait::async_trait;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use log::info;
use mockall::automock;
use tokio_diesel::{AsyncError, AsyncRunQueryDsl};
use uuid::Uuid;

type PgPool = Pool<ConnectionManager<PgConnection>>;

type Conn = PooledConnection<ConnectionManager<PgConnection>>;

embed_migrations!();

#[automock]
#[async_trait]
pub trait DataStoreService: Send + Sync {
    async fn get_posts(&self, is_published: bool) -> Result<Vec<Post>, AsyncError>;
    async fn create_post(&self, post_title: &str, post_body: &str) -> Result<Post, AsyncError>;
    async fn update_post(&self, post: &Post) -> Result<usize, AsyncError>;
    async fn delete_post(&self, post_id: uuid::Uuid) -> Result<usize, AsyncError>;
    async fn get_post(&self, post_id: uuid::Uuid) -> Result<Post, AsyncError>;
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

#[async_trait]
impl DataStoreService for Datastore {
    async fn get_posts(&self, is_published: bool) -> Result<Vec<Post>, AsyncError> {
        let posts_query = posts::table
            .filter(published.eq(is_published))
            .load_async(&self.pool);
        posts_query.await
    }

    async fn create_post(&self, post_title: &str, post_body: &str) -> Result<Post, AsyncError> {
        let new_post = Post {
            id: Uuid::new_v4(),
            title: String::from(post_title),
            body: String::from(post_body),
            published: true,
        };

        diesel::insert_into(posts::table)
            .values(&new_post)
            .get_result_async(&self.pool)
            .await
    }

    async fn update_post(&self, post: &Post) -> Result<usize, AsyncError> {
        let target = posts::table.filter(id.eq(post.id));

        diesel::update(target)
            .set((
                title.eq(&post.title),
                body.eq(&post.body),
                published.eq(&post.published),
            ))
            .execute_async(&self.pool)
            .await
    }

    async fn delete_post(&self, post_id: uuid::Uuid) -> Result<usize, AsyncError> {
        let target = posts::table.filter(id.eq(post_id));
        diesel::delete(target).execute_async(&self.pool).await
    }

    async fn get_post(&self, post_id: uuid::Uuid) -> Result<Post, AsyncError> {
        posts::table
            .filter(id.eq(post_id))
            .first_async(&self.pool)
            .await
    }

    fn run_migrations(&self) {
        info!("Running migrations");
        let conn = self.get_connection();
        embedded_migrations::run(&conn).expect("Failed to run database migrations");
    }
}
