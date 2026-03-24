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
}

#[derive(Debug, Clone, FromRow)]
pub struct Reminder {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub due_at: chrono::DateTime<chrono::Utc>,
    pub done: bool,
}

impl Reminder {
    pub fn get_formatted_due_date(&self) -> String {
        self.due_at.format("%d-%m-%Y").to_string()
    }
}
