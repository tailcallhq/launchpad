use tokio::process::Command;

use crate::{
    proto::GithubRequest,
    utils::{create_directory_path, MessageChannel},
    AppError, AppResult,
};

pub async fn handle(message_channel: &MessageChannel, request: &GithubRequest) -> AppResult<()> {
    init_stack(message_channel, request).await?;

    deploy_stack(message_channel, request).await?;

    Ok(())
}

async fn init_stack(message_channel: &MessageChannel, request: &GithubRequest) -> AppResult<()> {
    message_channel.send_message("Initializing stack...").await;

    let directory_path = create_directory_path("deployments")?;

    let pulumi_init = Command::new("pulumi")
        .arg("stack")
        .arg("init")
        .arg(request.get_identifier())
        .arg("-C")
        .arg(directory_path)
        .output()
        .await?;

    message_channel.trace(&format!("pulumi_init: {:?}", pulumi_init.status.code()));
    message_channel.trace(&format!(
        "stdout: {:?}",
        String::from_utf8_lossy(&pulumi_init.stdout)
    ));

    if let Some(code) = pulumi_init.status.code() {
        let err_msg = String::from_utf8_lossy(&pulumi_init.stderr);
        if code != 0 && err_msg.contains("Conflict: Another update is currently in progress.") {
            return Err(AppError::Simple(
                "Another update is currently in progress".into(),
            ));
        } else if code != 0 && !err_msg.contains("already exists") {
            message_channel.error(&format!("stderr: {:?}", err_msg));
            return Err(AppError::Simple("Stack initialization failed.".into()));
        }
    }

    message_channel
        .send_message("Stack initialization finished...")
        .await;

    Ok(())
}

async fn deploy_stack(message_channel: &MessageChannel, request: &GithubRequest) -> AppResult<()> {
    message_channel.send_message("Deploying...").await;

    let directory_path = create_directory_path("deployments")?;

    let pulumi_up = Command::new("pulumi")
        .arg("up")
        .arg("-c")
        .arg(format!(
            "linked_config=https://raw.githubusercontent.com/{}/{}/{}/config.graphql",
            request.username, request.repository, request.branch
        ))
        .arg("-s")
        .arg(request.get_identifier())
        .arg("-C")
        .arg(directory_path)
        .arg("-y")
        .output()
        .await?;

    message_channel.trace(&format!("pulumi_up: {:?}", pulumi_up.status.code()));
    let stdout = String::from_utf8_lossy(&pulumi_up.stdout);
    message_channel.trace(&format!("stdout: {:?}", stdout));

    if let Some(code) = pulumi_up.status.code() {
        let err_msg = String::from_utf8_lossy(&pulumi_up.stderr);
        if code != 0 && err_msg.contains("Conflict: Another update is currently in progress.") {
            return Err(AppError::Simple(
                "Another update is currently in progress".into(),
            ));
        } else if code != 0 {
            message_channel.error(&format!("stderr: {:?}", err_msg));
            return Err(AppError::Simple("Deployment failed.".into()));
        }
    }

    message_channel.send_message("Deployment finished...").await;

    if let Some(url) = extract_output_url(&stdout) {
        message_channel
            .send_message(&format!("Deployed url: {}", url))
            .await;
    }

    Ok(())
}

fn extract_output_url(input: &str) -> Option<String> {
    use regex::Regex;

    let output_url_pattern = r#"Outputs:\s+url:\s+"(https?://[^\s"\\]+)""#;

    let re = Regex::new(output_url_pattern).unwrap();

    re.captures(input).map(|captures| captures[1].to_string())
}
