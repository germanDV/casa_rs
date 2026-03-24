use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateCosa {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateNote {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateReminder {
    pub title: String,
    pub body: String,
    pub due_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateContact {
    pub name: String,
    pub contact_details: String,
}
