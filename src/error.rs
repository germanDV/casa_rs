use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub enum AppError {
    BadRequest,
    NotFound,
    InternalError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::BadRequest => StatusCode::BAD_REQUEST,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, status.to_string()).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(_: sqlx::Error) -> Self {
        AppError::InternalError
    }
}
