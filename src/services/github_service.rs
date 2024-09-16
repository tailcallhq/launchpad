use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use crate::{
    actions,
    proto::{github_service_server::GithubService, GithubRequest, GithubResponse, StreamMessage},
    utils::MessageChannel,
    AppError,
};

#[derive(Debug, Default)]
pub struct GithubDeploymentService {}

#[tonic::async_trait]
impl GithubService for GithubDeploymentService {
    type DeployStream = ReceiverStream<Result<StreamMessage, Status>>;

    async fn deploy(
        &self,
        request: Request<GithubRequest>,
    ) -> Result<Response<Self::DeployStream>, Status> {
        let request = request.get_ref().clone();

        let (tx, rx) = mpsc::channel(4);
        let message_channel = MessageChannel::new(tx);

        tokio::spawn(async move {
            match actions::github_service_deploy::handle(&message_channel, &request).await {
                Ok(_) => message_channel.debug("Closing connection gracefully."),
                Err(err) => send_error(&message_channel, err).await,
            };
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type TeardownStream = ReceiverStream<Result<StreamMessage, Status>>;

    async fn teardown(
        &self,
        request: Request<GithubRequest>,
    ) -> Result<Response<Self::TeardownStream>, Status> {
        let request = request.get_ref().clone();

        let (tx, rx) = mpsc::channel(4);
        let message_channel = MessageChannel::new(tx);

        tokio::spawn(async move {
            match actions::github_service_teardown::handle(&message_channel, &request).await {
                Ok(_) => message_channel.debug("Closing connection gracefully."),
                Err(err) => send_error(&message_channel, err).await,
            };
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn status(
        &self,
        request: Request<GithubRequest>,
    ) -> Result<Response<GithubResponse>, Status> {
        actions::github_service_status::handle(request.get_ref()).await
    }
}

async fn send_error(message_channel: &MessageChannel, err: AppError) {
    match err {
        AppError::Simple(msg) => message_channel.send_status(Status::cancelled(msg)).await,
        AppError::IoError(_) => {
            message_channel
                .send_status(Status::cancelled("IO Error"))
                .await
        }
    };
}
