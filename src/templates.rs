use crate::models::{Contact, Cosa, Note, Reminder};
use askama::Template;
use axum::response::{Html, IntoResponse, Response};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub cosas: Vec<Cosa>,
}

impl IntoResponse for IndexTemplate {
    fn into_response(self) -> Response {
        Html(self.render().unwrap()).into_response()
    }
}

#[derive(Template)]
#[template(path = "cosa.html")]
pub struct CosaTemplate {
    pub cosa: Cosa,
    pub notes: Vec<Note>,
    pub reminders: Vec<Reminder>,
    pub contacts: Vec<Contact>,
}

impl IntoResponse for CosaTemplate {
    fn into_response(self) -> Response {
        Html(self.render().unwrap()).into_response()
    }
}
