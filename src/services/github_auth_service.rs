use regex::Regex;
use reqwest::StatusCode;
use serde::Deserialize;
use tonic::{Request, Response, Status};

use crate::{
    proto::{
        github_auth_service_server, Empty, GetAccessTokenRequest, GetAccessTokenResponse,
        LoginLinkResponse, LoginRequest, UserInfoResponse,
    },
    AppError,
};

#[derive(Debug)]
pub struct GithubAuthService {
    client_id: String,
    client_secret: String,
}

impl GithubAuthService {
    pub fn new(client_id: &str, client_secret: &str) -> Self {
        Self {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
        }
    }
}

#[tonic::async_trait]
impl github_auth_service_server::GithubAuthService for GithubAuthService {
    async fn start(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginLinkResponse>, Status> {
        Ok(Response::new(LoginLinkResponse {
            url: format!(
                "https://github.com/login/oauth/authorize?client_id={}&state={}",
                self.client_id,
                request.into_inner().state
            ),
        }))
    }

    async fn get_access_token(
        &self,
        request: Request<GetAccessTokenRequest>,
    ) -> Result<Response<GetAccessTokenResponse>, Status> {
        let access_token = get_access_token(
            &self.client_id,
            &self.client_secret,
            &request.into_inner().access_code,
        )
        .await?;
        Ok(Response::new(GetAccessTokenResponse { access_token }))
    }

    async fn user_info(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<UserInfoResponse>, Status> {
        let extension = request.extensions().get::<AuthExtension>().unwrap();

        match &extension.bearer {
            Some(access_token) => {
                let user = get_user(&self.client_id, &self.client_secret, access_token).await?;
                Ok(Response::new(user))
            }
            None => Err(Status::unauthenticated("The request is not authorized")),
        }
    }
}

async fn get_access_token(
    client_id: &str,
    client_secret: &str,
    access_code: &str,
) -> Result<String, AppError> {
    let url = format!(
        "https://github.com/login/oauth/access_token?client_id={}&client_secret={}&code={}",
        client_id, client_secret, access_code
    );
    let res = reqwest::get(url).await?;
    let body = res.text().await?;
    let re = Regex::new("access_token=([a-z_A-Z0-9]+)").unwrap();
    let captures = re.captures(&body).unwrap();
    Ok(captures[1].to_string())
}

pub fn auth_interceptor(mut req: Request<()>) -> Result<Request<()>, Status> {
    let bearer = match req.metadata().get("authorization") {
        Some(bearer) => bearer
            .to_str()
            .map_err(|_| Status::invalid_argument("`authorization` header is bad formatted"))?
            .split(" ")
            .last()
            .map(|bearer| bearer.to_string()),
        None => None,
    };

    req.extensions_mut().insert(AuthExtension { bearer });

    Ok(req)
}

#[derive(Clone)]
struct AuthExtension {
    pub bearer: Option<String>,
}

async fn get_user(
    client_id: &str,
    client_secret: &str,
    access_token: &str,
) -> Result<UserInfoResponse, AppError> {
    let url = format!("https://api.github.com/applications/{}/token", client_id);

    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .body(format!("{{\"access_token\":\"{}\"}}", access_token))
        .basic_auth(client_id, Some(client_secret))
        .header("Content-Type", "application/json")
        .header("User-Agent", "Tailcall Launchpad")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?;
    if res.status() == StatusCode::OK {
        let json: AuthInfoJson = res.json().await.unwrap();

        Ok(UserInfoResponse {
            id: json.user.id,
            username: json.user.login,
        })
    } else {
        Err(AppError::Simple("Could not fetch user data.".to_string()))
    }
}

#[derive(Deserialize)]
pub struct AuthInfoJson {
    pub id: i64,
    pub user: UserInfoJson,
}

#[derive(Deserialize)]
pub struct UserInfoJson {
    pub id: i64,
    pub login: String,
}
