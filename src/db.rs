use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

pub async fn create_pool(database_url: &str) -> anyhow::Result<SqlitePool> {
    let pool = SqlitePoolOptions::new().connect(database_url).await?;

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
            body       TEXT    NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS reminders (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            cosa_id    INTEGER NOT NULL,
            title      TEXT    NOT NULL,
            body       TEXT    NOT NULL,
            due_at     TEXT    NOT NULL,
            done       BOOLEAN NOT NULL DEFAULT FALSE
        )",
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}
