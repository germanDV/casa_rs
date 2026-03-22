mod db;
mod error;
mod forms;
mod models;
mod templates;

use crate::db::create_pool;
use crate::error::AppError;
use crate::forms::{CreateCosa, CreateNote};
use crate::models::{Cosa, Note};
use crate::templates::{CosaTemplate, IndexTemplate};
use axum::{
    Form, Router,
    extract::{Path, State},
    response::Redirect,
    routing::{get, post},
};
use sqlx::SqlitePool;

pub fn create_app(pool: SqlitePool) -> Router {
    Router::new()
        .route("/", get(list_cosas))
        .route("/cosas/{cosa_id}", get(get_cosa))
        .route("/cosas", post(create_cosa))
        .route("/cosas/{cosa_id}/notes", post(create_note))
        .with_state(pool)
}

async fn create_cosa(
    State(pool): State<SqlitePool>,
    Form(input): Form<CreateCosa>,
) -> Result<Redirect, AppError> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::BadRequest);
    }

    let description = input.description.trim().to_string();
    if description.is_empty() {
        return Err(AppError::BadRequest);
    }

    sqlx::query("INSERT INTO cosas (name, description) VALUES (?, ?)")
        .bind(&name)
        .bind(&description)
        .execute(&pool)
        .await?;

    Ok(Redirect::to("/"))
}

async fn create_note(
    State(pool): State<SqlitePool>,
    Path(cosa_id): Path<i64>,
    Form(input): Form<CreateNote>,
) -> Result<Redirect, AppError> {
    let title = input.title.trim().to_string();
    if title.is_empty() {
        return Err(AppError::BadRequest);
    }

    let body = input.body.trim().to_string();
    if body.is_empty() {
        return Err(AppError::BadRequest);
    }

    sqlx::query("INSERT INTO notes (cosa_id, title, body, created_at) VALUES (?, ?, ?, ?)")
        .bind(&cosa_id)
        .bind(&title)
        .bind(&body)
        .bind(chrono::Utc::now())
        .execute(&pool)
        .await?;

    Ok(Redirect::to(&format!("/cosas/{cosa_id}")))
}

async fn list_cosas(State(pool): State<SqlitePool>) -> IndexTemplate {
    let cosas = sqlx::query_as::<_, Cosa>("SELECT id, name, description FROM cosas")
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    IndexTemplate { cosas }
}

async fn get_cosa(
    State(pool): State<SqlitePool>,
    Path(cosa_id): Path<i64>,
) -> Result<CosaTemplate, AppError> {
    let cosa = sqlx::query_as::<_, Cosa>("SELECT id, name, description FROM cosas WHERE id = ?")
        .bind(&cosa_id)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::NotFound)?;

    let notes = sqlx::query_as::<_, Note>(
        "SELECT id, title, body, created_at FROM notes where cosa_id = ?",
    )
    .bind(&cosa_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    Ok(CosaTemplate { cosa, notes })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = create_pool("sqlite://casa.db?mode=rwc").await?;
    let app = create_app(pool);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Listening on http://localhost:3000");
    axum::serve(listener, app).await?;
    Ok(())
}
