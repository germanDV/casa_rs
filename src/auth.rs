use crate::error::AppError;
use axum::{
    extract::Request,
    http::{StatusCode, header::LOCATION},
    middleware::Next,
    response::Response,
};
use cookie::Cookie;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Credentials {
    email: String,
    password: String,
}

impl Credentials {
    pub fn new(email: String, password: String) -> Self {
        Self { email, password }
    }

    pub fn matches(&self, other: &Credentials) -> bool {
        self.email == other.email && self.password == other.password
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

fn jwt_secret() -> String {
    crate::config::get().jwt_secret.clone()
}

pub fn create_jwt(email: &str) -> Result<String, AppError> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(31))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: email.to_owned(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
    .map_err(|_| AppError::Unauthorized)
}

fn validate_jwt(token: &str) -> bool {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &Validation::default(),
    )
    .is_ok()
}

fn redirect_to(location: &str) -> Response {
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(LOCATION, location)
        .body(axum::body::Body::empty())
        .unwrap()
}

fn redirect_to_login_clearing_cookie() -> Response {
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(LOCATION, "/login")
        .header(
            "Set-Cookie",
            "auth_token=; Max-Age=0; Path=/; HttpOnly; SameSite=Strict",
        )
        .body(axum::body::Body::empty())
        .unwrap()
}

pub async fn auth_middleware(request: Request, next: Next) -> Response {
    let path = request.uri().path();

    if path == "/health" {
        return next.run(request).await;
    }

    let auth_token: Option<String> = request
        .headers()
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|cookie_str| {
            Cookie::split_parse(cookie_str)
                .filter_map(Result::ok)
                .find(|c| c.name() == "auth_token")
                .map(|c| c.value().to_string())
        });

    if path == "/login" {
        return match auth_token {
            Some(token) if validate_jwt(&token) => redirect_to("/"),
            Some(_) => redirect_to_login_clearing_cookie(),
            None => next.run(request).await,
        };
    }

    match auth_token {
        Some(token) if validate_jwt(&token) => next.run(request).await,
        Some(_) => redirect_to_login_clearing_cookie(),
        None => redirect_to("/login"),
    }
}

pub fn verify_credentials(target: Credentials, candidate: Credentials) -> Result<String, AppError> {
    if target.matches(&candidate) {
        create_jwt(&target.email)
    } else {
        Err(AppError::Unauthorized)
    }
}
