use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Form, Router,
    extract::{Path, State},
    http::StatusCode,
    response::Redirect,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

#[derive(sqlx::FromRow)]
struct Cosa {
    id: i64,
    name: String,
    description: String,
}

#[derive(sqlx::FromRow)]
struct Note {
    id: i64,
    title: String,
    body: String,
    created_at: DateTime<Utc>,
}

#[derive(Template, WebTemplate)]
#[template(path = "index.html")]
struct IndexTemplate {
    cosas: Vec<Cosa>,
}

#[derive(Template, WebTemplate)]
#[template(path = "cosa.html")]
struct CosaTemplate {
    cosa: Cosa,
    notes: Vec<Note>,
}

#[derive(Deserialize)]
struct CreateCosa {
    name: String,
    description: String,
}

#[derive(Deserialize)]
struct CreateNote {
    title: String,
    body: String,
}

async fn create_cosa(
    State(pool): State<SqlitePool>,
    Form(input): Form<CreateCosa>,
) -> Result<Redirect, StatusCode> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let description = input.description.trim().to_string();
    if description.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    sqlx::query("INSERT INTO cosas (name, description) VALUES (?, ?)")
        .bind(&name)
        .bind(&description)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::to("/"))
}

/// POST /cosas/:cosa_id/notes  — create a note for a cosa, then redirect back to /
async fn create_note(
    State(pool): State<SqlitePool>,
    Path(cosa_id): Path<i64>,
    Form(input): Form<CreateNote>,
) -> Result<Redirect, StatusCode> {
    let title = input.title.trim().to_string();
    if title.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let body = input.body.trim().to_string();
    if body.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    sqlx::query("INSERT INTO notes (cosa_id, title, body, created_at) VALUES (?, ?, ?, ?)")
        .bind(&cosa_id)
        .bind(&title)
        .bind(&body)
        .bind(Utc::now())
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::to(&format!("/cosas/{}", cosa_id)))
}

/// GET /  — render all cosas
async fn list_cosas(State(pool): State<SqlitePool>) -> IndexTemplate {
    let cosas = sqlx::query_as::<_, Cosa>("SELECT id, name, description FROM cosas")
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    IndexTemplate { cosas }
}

/// GET /cosas/:cosa_id — render all notes for a cosa
async fn get_cosa(
    State(pool): State<SqlitePool>,
    Path(cosa_id): Path<i64>,
) -> Result<CosaTemplate, StatusCode> {
    let cosa = sqlx::query_as::<_, Cosa>("SELECT id, name, description FROM cosas WHERE id = ?")
        .bind(&cosa_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

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
    let pool = SqlitePoolOptions::new()
        .connect("sqlite://casa.db?mode=rwc")
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS cosas (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT    NOT NULL,
            description TEXT    NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS notes (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            cosa_id    INTEGER NOT NULL,
            title      TEXT    NOT NULL,
            body       TEXT    NOT NULL,
            created_at TEXT    NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    let app = Router::new()
        .route("/", get(list_cosas))
        .route("/cosas/{cosa_id}", get(get_cosa))
        .route("/cosas", post(create_cosa))
        .route("/cosas/{cosa_id}/notes", post(create_note))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Listening on http://localhost:3000");
    axum::serve(listener, app).await?;

    Ok(())
}
