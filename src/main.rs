use tailcall_launchpad::{
    proto::github_service_server::GithubServiceServer,
    services::github_service::GithubDeploymentService,
};
use tonic::transport::Server;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
    // setup debugging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    // read .env file
    match dotenvy::dotenv() {
        Ok(_) => {
            println!(".env file loaded")
        }
        Err(_) => {
            println!("environment variables loaded")
        }
    }

    // initialize services
    let github_deployment_service = GithubServiceServer::new(GithubDeploymentService::default());

    // reflection service
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(tailcall_launchpad::proto::FILE_DESCRIPTOR_SET)
        .build_v1()
        .unwrap();

    // start server
    let grpc_service = Server::builder()
        .add_service(reflection_service)
        .add_service(github_deployment_service)
        .into_service()
        .into_axum_router();

    run(grpc_service).await;
}

async fn run(router: axum::Router) {
    // extract important config variables
    use std::env;
    let host = env::var("SERVER_HOST").expect("SERVER_HOST is not set in .env file");
    let port = env::var("SERVER_PORT").expect("SERVER_PORT is not set in .env file");
    let addr = format!("{host}:{port}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
