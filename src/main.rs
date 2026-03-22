use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Form, Router,
    extract::State,
    http::StatusCode,
    response::Redirect,
    routing::{get, post},
};
use serde::Deserialize;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

// ── DB model
#[derive(sqlx::FromRow)]
struct Note {
    id: i64,
    body: String,
}

// ── Templates
#[derive(Template, WebTemplate)]
#[template(path = "index.html")]
struct IndexTemplate {
    notes: Vec<Note>,
}

// ── Form input
#[derive(Deserialize)]
struct CreateNote {
    body: String,
}

/// GET /  — render all notes
async fn list_notes(State(pool): State<SqlitePool>) -> IndexTemplate {
    let notes = sqlx::query_as::<_, Note>("SELECT id, body FROM notes ORDER BY id DESC")
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    IndexTemplate { notes }
}

/// POST /notes  — create a note, then redirect back to /
async fn create_note(
    State(pool): State<SqlitePool>,
    Form(input): Form<CreateNote>,
) -> Result<Redirect, StatusCode> {
    let body = input.body.trim().to_string();
    if body.is_empty() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    sqlx::query("INSERT INTO notes (body) VALUES (?)")
        .bind(&body)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::to("/"))
}

// ── Main
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Open (or create) notes.db in the current directory
    let pool = SqlitePoolOptions::new()
        .connect("sqlite://notes.db?mode=rwc")
        .await?;

    // Create table if it doesn't exist
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS notes (
            id   INTEGER PRIMARY KEY AUTOINCREMENT,
            body TEXT    NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    let app = Router::new()
        .route("/", get(list_notes))
        .route("/notes", post(create_note))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Listening on http://localhost:3000");
    axum::serve(listener, app).await?;

    Ok(())
}
