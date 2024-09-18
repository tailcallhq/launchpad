use std::env;
use tailcall_launchpad::{
    proto::{
        github_auth_service_server::GithubAuthServiceServer,
        github_service_server::GithubServiceServer,
    },
    services::{
        github_auth_service::{auth_interceptor, GithubAuthService},
        github_service::GithubDeploymentService,
    },
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
    let github_deployment_service =
        GithubServiceServer::with_interceptor(GithubDeploymentService::default(), auth_interceptor);

    let client_id = env::var("OAUTH_CLIENT_ID").expect("OAUTH_CLIENT_ID is not set in .env file");
    let client_secret =
        env::var("OAUTH_CLIENT_SECRET").expect("OAUTH_CLIENT_SECRET is not set in .env file");
    let github_auth_service = GithubAuthServiceServer::with_interceptor(
        GithubAuthService::new(&client_id, &client_secret),
        auth_interceptor,
    );

    // reflection service
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(tailcall_launchpad::proto::FILE_DESCRIPTOR_SET)
        .build_v1()
        .unwrap();

    // extract important config variables
    let host = env::var("SERVER_HOST").expect("SERVER_HOST is not set in .env file");
    let port = env::var("SERVER_PORT").expect("SERVER_PORT is not set in .env file");
    let addr = format!("{host}:{port}");

    println!("server running {}", addr);
    // start server
    Server::builder()
        .add_service(reflection_service)
        .add_service(github_deployment_service)
        .add_service(github_auth_service)
        .serve(addr.parse().unwrap())
        .await
        .unwrap();
}
