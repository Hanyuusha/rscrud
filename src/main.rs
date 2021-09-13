#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use dotenv::dotenv;
use log::{error, info};
use std::error::Error;
use tonic::transport::Server;

use crate::posts::posts_service_server::PostsServiceServer;
use grpc::server::PostsServiceImp;

use crate::datastore::store::{DataStoreService, Datastore};

mod datastore;
mod grpc;

pub mod posts {
    tonic::include_proto!("posts.v1");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    env_logger::init();

    info!("Starting Simple CRUD App server...");

    let postgres_dsn = std::env::var("POSTGRES_DSN");

    let postgres_dsn = match postgres_dsn {
        Ok(postgres_dsn) => postgres_dsn,
        Err(_) => {
            error!("env 'POSTGRES_DSN' not set!");
            return Ok(());
        }
    };

    let addr = std::env::var("GRPC_SERVER_ADDRESS");

    let addr = match addr {
        Ok(addr) => addr,
        Err(_) => {
            error!("env 'GRPC_SERVER_ADDRESS' not set!");
            return Ok(());
        }
    };

    let socket_addr = match addr.parse() {
        Ok(addr) => addr,
        Err(_) => {
            error!("cannot parse {} to SocketAddr", addr);
            return Ok(());
        }
    };

    let store = Datastore::new(postgres_dsn);
    store.run_migrations();

    let posts = PostsServiceImp::new(Box::new(store));
    let svc = PostsServiceServer::new(posts);

    info!("Simple CRUD App server started on {}", addr);

    Server::builder()
        .add_service(svc)
        .serve(socket_addr)
        .await?;
    Ok(())
}
