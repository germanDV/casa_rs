use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Cosa {
    pub id: i64,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct Note {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
