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

impl Note {
    pub fn get_formatted_date(&self) -> String {
        self.created_at.format("%d-%m-%Y").to_string()
    }
}
