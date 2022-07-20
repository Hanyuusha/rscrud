use mockall::predicate;
use tonic::Request;
use uuid::Uuid;

use crate::datastore::models::Post;
use crate::datastore::store::MockDataStoreService;
use crate::grpc::server::PostsServiceImp;
use crate::posts::posts_service_server::PostsService;
use crate::posts::*;

macro_rules! aw {
    ($e:expr) => {
        tokio_test::block_on($e)
    };
}

#[test]
fn get_post_list() {
    let mut store_mock = MockDataStoreService::new();
    let mut posts: Vec<Post> = Vec::new();
    let mock_post = Post {
        id: Uuid::new_v4(),
        title: "title".to_string(),
        body: "body".to_string(),
        published: true,
    };
    let id = mock_post.id.to_string();
    posts.push(mock_post);
    store_mock
        .expect_get_posts()
        .with(predicate::eq(&true))
        .return_once(|_| Ok(posts));

    let server = PostsServiceImp::new(Box::new(store_mock));
    let req = Request::new(PostListRequest { published: true });
    let response = aw!(server.get_post_list(req));

    match response {
        Ok(posts) => {
            let posts = posts.into_inner().posts;
            let post = posts.get(0).unwrap();
            assert_eq!(post.id, id)
        }
        Err(err) => panic!("{}", err),
    }
}
