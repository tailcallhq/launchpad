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
}

type AppResult<T> = Result<T, AppError>;
