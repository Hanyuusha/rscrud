use diesel::result::Error;
use log::{debug, error, info};
use tonic::{Request, Response, Status};
use uuid::Uuid;
use tokio_diesel::AsyncError;
use crate::datastore::models::Post as MPost;
use crate::datastore::store::DataStoreService;
use crate::posts::*;
use crate::posts::posts_service_server::PostsService;

pub struct PostsServiceImp {
    store: Box<dyn DataStoreService>,
}

impl PostsServiceImp {
    pub fn new(store: Box<impl DataStoreService + 'static>) -> Self {
        Self { store }
    }
}

#[tonic::async_trait]
impl PostsService for PostsServiceImp {
    async fn get_post_list(
        &self,
        request: Request<PostListRequest>,
    ) -> Result<Response<PostListResponse>, Status> {
        debug!("GetPostList got a request: {:?}", request);

        let published = request.into_inner().published;
        let posts = self.store
            .get_posts(published)
            .await;

        let response = match posts {
            Ok(posts) => {
                let posts: Vec<Post> = posts
                    .into_iter()
                    .map(|post| Post {
                        id: post.id.to_string(),
                        title: post.title,
                        body: post.body,
                        published: post.published,
                    })
                    .collect();
                info!("GetPostList find {} posts", posts.len());
                Ok(Response::new(PostListResponse { posts }))
            }

            Err(err) => {
                error!("GetPostList err: {}", err);
                Err(Status::internal(err.to_string()))
            }
        };

        response
    }

    async fn create_post(
        &self,
        request: Request<CreatePostRequest>,
    ) -> Result<Response<CreatePostResponse>, Status> {
        debug!("CreatePost got a request: {:?}", request);

        let post = request.into_inner();
        let post = self.store
            .create_post(&post.title, &post.body)
            .await;

        let response = match post {
            Ok(post) => {
                info!("CreatePost created '{:?}'", post);
                Ok(Response::new(CreatePostResponse {
                    id: post.id.to_string(),
                }))
            }

            Err(err) => {
                error!("CreatePost err: {}", err);
                Err(Status::internal(err.to_string()))
            }
        };

        response
    }

    async fn update_post(
        &self,
        request: Request<UpdatePostRequest>,
    ) -> Result<Response<SuccessResponse>, Status> {
        debug!("UpdatePost got a request: {:?}", request);

        let post = request.into_inner().post.expect("Failed to read post");

        let id = Uuid::parse_str(&post.id);

        let id = match id {
            Ok(id) => id,
            Err(_) => {
                error!("UpdatePost cannot parse post_id to UUID");
                return Err(Status::internal("Cannot parse post_id to UUID"));
            }
        };

        let result = self.store
            .update_post(&MPost {
                id,
                title: post.title,
                body: post.body,
                published: post.published,
            })
            .await;

        let response = match result {
            Ok(count) => {
                info!(
                    "UpdatePost updated post_id='{}', updated records {}",
                    id, count
                );
                Ok(Response::new(SuccessResponse { success: count > 0 }))
            }
            Err(err) => {
                error!("UpdatePost err: {}", err);
                Err(Status::internal(err.to_string()))
            }
        };

        response
    }

    async fn delete_post(
        &self,
        request: Request<DeletePostRequest>,
    ) -> Result<Response<SuccessResponse>, Status> {
        debug!("DeletePost got a request: {:?}", request);

        let post = request.into_inner();
        let id = Uuid::parse_str(&post.id);

        let id = match id {
            Ok(id) => id,
            Err(_) => {
                error!("DeletePost cannot parse post_id to UUID");
                return Err(Status::internal("Cannot parse post_id to UUID"));
            }
        };

        let result = self.store
            .delete_post(id)
            .await;

        let response = match result {
            Ok(count) => {
                info!(
                    "DeletePost deleted post_id='{}', deleted records {}",
                    id, count
                );
                Ok(Response::new(SuccessResponse { success: count > 0 }))
            }

            Err(err) => {
                error!("DeletePost err: {}", err);
                Err(Status::internal(err.to_string()))
            }
        };

        response
    }

    async fn get_post(
        &self,
        request: Request<GetPostRequest>,
    ) -> Result<Response<GetPostResponse>, Status> {
        debug!("GetPost got a request: {:?}", request);

        let post = request.into_inner();
        let id = Uuid::parse_str(&post.id);

        let id = match id {
            Ok(id) => id,
            Err(_) => {
                error!("GetPost cannot parse post_id to UUID");
                return Err(Status::internal("Cannot parse post_id to UUID"));
            }
        };

        let result = self.store
            .get_post(id)
            .await;

        let response = match result {
            Ok(post) => {
                info!("GetPost retrieve post_id='{}'", id);

                Ok(Response::new(GetPostResponse {
                    post: Some(Post {
                        id: post.id.to_string(),
                        title: post.title,
                        body: post.body,
                        published: post.published,
                    }),
                }))
            }

            Err(err) => {
                error!("GetPost err: {}", err);
                match err {
                    AsyncError::Error(err) => {
                        match err {
                            Error::NotFound => Err(Status::not_found("Post not found")),

                            _ => Err(Status::internal(err.to_string())),
                        }
                    }

                    _ => Err(Status::internal(err.to_string())),
                }
            }
        };

        response
    }
}
