pub mod proto {
    tonic::include_proto!("tailcall");
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("tailcall_descriptor");
}
mod actions;
pub mod services;
mod utils;

#[derive(thiserror::Error, Debug)]
enum AppError {
    #[error("Error: {0}")]
    Simple(String),
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Remote Error: {0}")]
    RemoteRequestError(#[from] reqwest::Error),
}

type AppResult<T> = Result<T, AppError>;

// TODO: add logging to make debugging easier
impl From<AppError> for tonic::Status {
    fn from(value: AppError) -> Self {
        match value {
            AppError::Simple(error) => tonic::Status::aborted(error),
            AppError::IoError(_error) => tonic::Status::internal("IO Error"),
            AppError::RemoteRequestError(_error) => tonic::Status::internal("Remote Request Error"),
        }
    }
}
