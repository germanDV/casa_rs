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
