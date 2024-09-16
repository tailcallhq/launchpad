use tokio::process::Command;

use crate::{
    proto::GithubRequest,
    utils::{create_directory_path, MessageChannel},
    AppError, AppResult,
};

pub async fn handle(message_channel: &MessageChannel, request: &GithubRequest) -> AppResult<()> {
    teardown_deployment(message_channel, request).await?;

    remove_stack(message_channel, request).await?;

    Ok(())
}

async fn teardown_deployment(
    message_channel: &MessageChannel,
    request: &GithubRequest,
) -> AppResult<()> {
    message_channel
        .send_message("Tearing down deployment...")
        .await;

    let directory_path = create_directory_path("deployments")?;

    let pulumi_destroy = Command::new("pulumi")
        .arg("destroy")
        .arg("-s")
        .arg(request.get_identifier())
        .arg("-C")
        .arg(directory_path)
        .arg("-y")
        .output()
        .await?;

    message_channel.trace(&format!(
        "pulumi_destroy: {:?}",
        pulumi_destroy.status.code()
    ));
    message_channel.trace(&format!(
        "stdout: {:?}",
        String::from_utf8_lossy(&pulumi_destroy.stdout)
    ));

    if let Some(code) = pulumi_destroy.status.code() {
        let err_msg = String::from_utf8_lossy(&pulumi_destroy.stderr);
        if code != 0 && err_msg.contains("Conflict: Another update is currently in progress.") {
            return Err(AppError::Simple(
                "Another update is currently in progress".into(),
            ));
        } else if code != 0 && !err_msg.contains("no stack named") {
            message_channel.error(&format!("stderr: {:?}", err_msg));
            return Err(AppError::Simple("Tearing down failed.".into()));
        }
    }

    message_channel
        .send_message("Tearing down finished...")
        .await;

    Ok(())
}

async fn remove_stack(message_channel: &MessageChannel, request: &GithubRequest) -> AppResult<()> {
    message_channel
        .send_message("Removing deployment stack...")
        .await;

    let directory_path = create_directory_path("deployments")?;

    let pulumi_stack_rm = Command::new("pulumi")
        .arg("stack")
        .arg("rm")
        .arg(request.get_identifier())
        .arg("-C")
        .arg(directory_path)
        .arg("-y")
        .output()
        .await?;

    message_channel.trace(&format!(
        "pulumi_stack_rm: {:?}",
        pulumi_stack_rm.status.code()
    ));
    message_channel.trace(&format!(
        "stdout: {:?}",
        String::from_utf8_lossy(&pulumi_stack_rm.stdout)
    ));

    if let Some(code) = pulumi_stack_rm.status.code() {
        let err_msg = String::from_utf8_lossy(&pulumi_stack_rm.stderr);
        if code != 0 && err_msg.contains("Conflict: Another update is currently in progress.") {
            return Err(AppError::Simple(
                "Another update is currently in progress".into(),
            ));
        } else if code != 0 && !err_msg.contains("no stack named") {
            message_channel.error(&format!("stderr: {:?}", err_msg));
            return Err(AppError::Simple("Removing of stack failed.".into()));
        }
    }

    message_channel
        .send_message("Stack removed successfully...")
        .await;

    Ok(())
}
