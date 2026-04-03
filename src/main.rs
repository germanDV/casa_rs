mod auth;
mod config;
mod db;
mod error;
mod forms;
mod models;
mod templates;

use axum::{
    Form, Router,
    extract::{Path, State},
    http::{StatusCode, header::SET_COOKIE},
    response::{IntoResponse, Redirect},
    routing::{delete, get, patch, post},
};
use cookie::{Cookie, time};
use sqlx::SqlitePool;

#[derive(Clone)]
struct AppState {
    pool: SqlitePool,
    credentials: auth::Credentials,
}

fn create_app(app_state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/", get(list_cosas))
        .route("/login", get(login_page))
        .route("/login", post(login))
        .route("/cosas", post(create_cosa))
        .route("/cosas/{cosa_id}", get(get_cosa))
        .route("/cosas/{cosa_id}", delete(delete_cosa))
        .route("/cosas/{cosa_id}/notes", post(create_note))
        .route("/cosas/{cosa_id}/notes/{note_id}", delete(delete_note))
        .route("/cosas/{cosa_id}/reminders", post(create_reminder))
        .route(
            "/cosas/{cosa_id}/reminders/{reminder_id}",
            delete(delete_reminder),
        )
        .route(
            "/cosas/{cosa_id}/reminders/{reminder_id}/done",
            patch(toggle_reminder_done),
        )
        .route("/cosas/{cosa_id}/contacts", post(create_contact))
        .route(
            "/cosas/{cosa_id}/contacts/{contact_id}",
            delete(delete_contact),
        )
        .layer(axum::middleware::from_fn(auth::auth_middleware))
        .with_state(app_state)
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

async fn create_cosa(
    State(AppState { pool, .. }): State<AppState>,
    Form(input): Form<forms::CreateCosa>,
) -> Result<Redirect, error::AppError> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(error::AppError::BadRequest);
    }

    let description = input.description.trim().to_string();
    if description.is_empty() {
        return Err(error::AppError::BadRequest);
    }

    sqlx::query("INSERT INTO cosas (name, description) VALUES (?, ?)")
        .bind(&name)
        .bind(&description)
        .execute(&pool)
        .await?;

    Ok(Redirect::to("/"))
}

async fn create_note(
    State(AppState { pool, .. }): State<AppState>,
    Path(cosa_id): Path<i64>,
    Form(input): Form<forms::CreateNote>,
) -> Result<Redirect, error::AppError> {
    let title = input.title.trim().to_string();
    if title.is_empty() {
        return Err(error::AppError::BadRequest);
    }

    let body = input.body.trim().to_string();

    sqlx::query("INSERT INTO notes (cosa_id, title, body) VALUES (?, ?, ?)")
        .bind(&cosa_id)
        .bind(&title)
        .bind(&body)
        .execute(&pool)
        .await?;

    Ok(Redirect::to(&format!("/cosas/{cosa_id}")))
}

async fn create_reminder(
    State(AppState { pool, .. }): State<AppState>,
    Path(cosa_id): Path<i64>,
    Form(input): Form<forms::CreateReminder>,
) -> Result<Redirect, error::AppError> {
    let title = input.title.trim().to_string();
    if title.is_empty() {
        return Err(error::AppError::BadRequest);
    }

    let body = input.body.trim().to_string();
    if body.is_empty() {
        return Err(error::AppError::BadRequest);
    }

    let due_at = chrono::NaiveDate::parse_from_str(&input.due_at, "%Y-%m-%d")
        .map_err(|_| error::AppError::BadRequest)?
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();

    sqlx::query("INSERT INTO reminders (cosa_id, title, body, due_at) VALUES (?, ?, ?, ?)")
        .bind(&cosa_id)
        .bind(&title)
        .bind(&body)
        .bind(due_at)
        .execute(&pool)
        .await?;

    Ok(Redirect::to(&format!("/cosas/{cosa_id}")))
}

async fn create_contact(
    State(AppState { pool, .. }): State<AppState>,
    Path(cosa_id): Path<i64>,
    Form(input): Form<forms::CreateContact>,
) -> Result<Redirect, error::AppError> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(error::AppError::BadRequest);
    }

    let contact_details = input.contact_details.trim().to_string();
    if contact_details.is_empty() {
        return Err(error::AppError::BadRequest);
    }

    sqlx::query("INSERT INTO contacts (cosa_id, name, contact_details) VALUES (?, ?, ?)")
        .bind(&cosa_id)
        .bind(&name)
        .bind(&contact_details)
        .execute(&pool)
        .await?;

    Ok(Redirect::to(&format!("/cosas/{cosa_id}")))
}

async fn login_page() -> templates::LoginTemplate {
    templates::LoginTemplate {}
}

async fn login(
    State(AppState { credentials, .. }): State<AppState>,
    Form(input): Form<forms::Login>,
) -> Result<impl IntoResponse, error::AppError> {
    let email = input.email.trim().to_string();
    if email.is_empty() {
        return Err(error::AppError::BadRequest);
    }

    let password = input.password.trim().to_string();
    if password.is_empty() {
        return Err(error::AppError::BadRequest);
    }

    let auth_token =
        auth::verify_credentials(credentials.clone(), auth::Credentials::new(email, password))?;

    let cookie = Cookie::build(("auth_token", auth_token))
        .http_only(true)
        .secure(true)
        .expires(time::OffsetDateTime::now_utc() + time::Duration::days(30))
        .path("/");

    let mut response = Redirect::to("/").into_response();

    response
        .headers_mut()
        .insert(SET_COOKIE, cookie.to_string().parse().unwrap());

    Ok(response)
}

async fn list_cosas(State(AppState { pool, .. }): State<AppState>) -> templates::IndexTemplate {
    let cosas = sqlx::query_as::<_, models::Cosa>("SELECT id, name, description FROM cosas")
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    templates::IndexTemplate { cosas }
}

async fn get_cosa(
    State(AppState { pool, .. }): State<AppState>,
    Path(cosa_id): Path<i64>,
) -> Result<templates::CosaTemplate, error::AppError> {
    let cosa =
        sqlx::query_as::<_, models::Cosa>("SELECT id, name, description FROM cosas WHERE id = ?")
            .bind(&cosa_id)
            .fetch_optional(&pool)
            .await?
            .ok_or(error::AppError::NotFound)?;

    let notes =
        sqlx::query_as::<_, models::Note>("SELECT id, title, body FROM notes where cosa_id = ?")
            .bind(&cosa_id)
            .fetch_all(&pool)
            .await
            .unwrap_or_default();

    let yesterday = chrono::Utc::now()
        .date_naive()
        .pred_opt()
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();

    let reminders = sqlx::query_as::<_, models::Reminder>(
        "SELECT id, title, body, due_at, done FROM reminders WHERE cosa_id = ? AND (done = FALSE OR due_at >= ?)",
    )
    .bind(&cosa_id)
    .bind(yesterday)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let contacts = sqlx::query_as::<_, models::Contact>(
        "SELECT id, name, contact_details FROM contacts where cosa_id = ?",
    )
    .bind(&cosa_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    Ok(templates::CosaTemplate {
        cosa,
        notes,
        reminders,
        contacts,
    })
}

async fn delete_note(
    State(AppState { pool, .. }): State<AppState>,
    Path((cosa_id, note_id)): Path<(i64, i64)>,
) -> Result<StatusCode, error::AppError> {
    sqlx::query("DELETE FROM notes WHERE id = ? AND cosa_id = ?")
        .bind(&note_id)
        .bind(&cosa_id)
        .execute(&pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_reminder(
    State(AppState { pool, .. }): State<AppState>,
    Path((cosa_id, reminder_id)): Path<(i64, i64)>,
) -> Result<StatusCode, error::AppError> {
    sqlx::query("DELETE FROM reminders WHERE id = ? AND cosa_id = ?")
        .bind(&reminder_id)
        .bind(&cosa_id)
        .execute(&pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_contact(
    State(AppState { pool, .. }): State<AppState>,
    Path((cosa_id, contact_id)): Path<(i64, i64)>,
) -> Result<StatusCode, error::AppError> {
    sqlx::query("DELETE FROM contacts WHERE id = ? AND cosa_id = ?")
        .bind(&contact_id)
        .bind(&cosa_id)
        .execute(&pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_cosa(
    State(AppState { pool, .. }): State<AppState>,
    Path(cosa_id): Path<i64>,
) -> Result<StatusCode, error::AppError> {
    sqlx::query("DELETE FROM notes WHERE cosa_id = ?")
        .bind(&cosa_id)
        .execute(&pool)
        .await?;

    sqlx::query("DELETE FROM reminders WHERE cosa_id = ?")
        .bind(&cosa_id)
        .execute(&pool)
        .await?;

    sqlx::query("DELETE FROM contacts WHERE cosa_id = ?")
        .bind(&cosa_id)
        .execute(&pool)
        .await?;

    sqlx::query("DELETE FROM cosas WHERE id = ?")
        .bind(&cosa_id)
        .execute(&pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn toggle_reminder_done(
    State(AppState { pool, .. }): State<AppState>,
    Path((cosa_id, reminder_id)): Path<(i64, i64)>,
) -> Result<StatusCode, error::AppError> {
    sqlx::query("UPDATE reminders SET done = 1-done WHERE id = ? AND cosa_id = ?")
        .bind(&reminder_id)
        .bind(&cosa_id)
        .execute(&pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    config::init();
    let cfg = config::get();

    let app = create_app(AppState {
        pool: db::create_pool("sqlite://casa.db?mode=rwc").await?,
        credentials: auth::Credentials::new(cfg.login_email.clone(), cfg.password.clone()),
    });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Listening on http://localhost:3000");
    axum::serve(listener, app).await?;

    Ok(())
}
