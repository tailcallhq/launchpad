use regex::Regex;
use tokio::process::Command;
use tonic::{Response, Status};

use crate::{
    proto::{GithubRequest, GithubResponse, GithubStatusEnum},
    utils::create_directory_path,
};

pub async fn handle(request: &GithubRequest) -> Result<Response<GithubResponse>, Status> {
    let directory_path = create_directory_path("deployments")?;

    let pulumi_info = Command::new("pulumi")
        .arg("stack")
        .arg("-s")
        .arg(request.get_identifier())
        .arg("-C")
        .arg(directory_path)
        .output()
        .await?;

    if let Some(code) = pulumi_info.status.code() {
        let err_msg = String::from_utf8_lossy(&pulumi_info.stderr);
        if code != 0 && err_msg.contains("no stack named") {
            return Err(Status::not_found(
                "No information have been found for this (repository, branch) combination.",
            ));
        } else if code != 0
            && err_msg.contains("Conflict: Another update is currently in progress.")
        {
            return Err(Status::cancelled("Another update is currently in progress"));
        } else if code != 0 {
            tracing::debug!("{}", err_msg);
            return Err(Status::internal(
                "Could not fetch metadata, please try again later.",
            ));
        }
    }
    let stdout: &str = &String::from_utf8_lossy(&pulumi_info.stdout);

    let endpoint = extract_output_url(stdout).unwrap_or_default();
    let num_of_resources = extract_num_of_resources(stdout).unwrap_or("-1".to_string());
    let resources: i32 = num_of_resources.parse().unwrap();
    let status = if resources == 14 {
        GithubStatusEnum::Deployed
    } else if resources == 0 {
        GithubStatusEnum::Down
    } else {
        GithubStatusEnum::Error
    };
    let updated_at = extract_updated_at(stdout).unwrap_or_default();

    let res = GithubResponse {
        username: request.username.clone(),
        repository: request.repository.clone(),
        branch: request.branch.clone(),
        status: status.into(),
        updated_at,
        endpoint,
    };

    Ok(Response::new(res))
}

fn extract_output_url(input: &str) -> Option<String> {
    let regex_pattern = r#"OUTPUT\s+VALUE\s+url\s+(https?:\/\/[^\s"\\]+)"#;

    let re = Regex::new(regex_pattern).unwrap();

    re.captures(input).map(|captures| captures[1].to_string())
}

fn extract_num_of_resources(input: &str) -> Option<String> {
    let regex_pattern = r#"Current\sstack\sresources\s\((\d+)\)"#;

    let re = Regex::new(regex_pattern).unwrap();

    re.captures(input).map(|captures| captures[1].to_string())
}

fn extract_updated_at(input: &str) -> Option<String> {
    let regex_pattern = r#"\((\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d+\s\+\d+\s[A-Z]+)\)"#;

    let re = Regex::new(regex_pattern).unwrap();

    re.captures(input).map(|captures| captures[1].to_string())
}
