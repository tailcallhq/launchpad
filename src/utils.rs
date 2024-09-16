use tokio::sync::mpsc;
use tonic::{Code, Status};

use crate::proto::{GithubRequest, StreamMessage};

impl GithubRequest {
    pub fn get_identifier(&self) -> String {
        format!("{}.-.{}.-.{}", self.username, self.repository, self.branch)
    }
}

pub struct MessageChannel {
    tx: mpsc::Sender<Result<StreamMessage, Status>>,
    span: tracing::Span,
}

impl MessageChannel {
    pub fn new(tx: mpsc::Sender<Result<StreamMessage, Status>>) -> Self {
        let id = uuid::Uuid::new_v4();
        let span = tracing::span!(
            tracing::Level::DEBUG,
            "message-channel",
            id = id.to_string()
        );

        Self { tx, span }
    }

    pub async fn send_message(&self, msg: &str) {
        self.debug(msg);
        let message = StreamMessage {
            message: msg.to_string(),
        };
        match self.tx.send(Ok(message)).await {
            Ok(_) => {}
            Err(err) => self.error(&format!("Error: {:?}", err)),
        };
    }

    pub async fn send_status(&self, status: Status) {
        if status.code() == Code::Ok {
            self.debug(status.message());
        } else {
            self.error(status.message());
        }

        match self.tx.send(Err(status)).await {
            Ok(_) => {}
            Err(err) => self.error(&format!("Error: {:?}", err)),
        };
    }

    pub fn trace(&self, msg: &str) {
        self.span.in_scope(|| tracing::trace!(msg))
    }

    pub fn debug(&self, msg: &str) {
        self.span.in_scope(|| tracing::debug!(msg))
    }

    pub fn error(&self, msg: &str) {
        self.span.in_scope(|| tracing::error!(msg))
    }
}

pub fn create_directory_path(relative_path: &str) -> Result<String, std::io::Error> {
    use std::path::PathBuf;

    let current_dir = std::env::current_dir().unwrap();
    let full_path = current_dir.join(PathBuf::from(relative_path));
    Ok(full_path.to_str().unwrap_or_default().to_string())
}
