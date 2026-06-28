use chrono::{DateTime, Local};
use serde::Serialize;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
pub enum EventKind {
    ApplicationFailure,
    SystemFailure,
    ApplicationWarning,
    SystemWarning,
}

#[derive(Clone, Debug, Serialize)]
pub struct Event {
    pub id: String,
    pub timestamp: DateTime<Local>,
    pub application: String,
    pub exec_path: String,
    pub reason: String,
    pub kind: EventKind,
    pub user: String,
    pub user_id: i64,
    pub classifier_reason: String,
}

#[derive(Clone, Serialize)]
pub struct DaySummary {
    pub day: String,
    pub score: f32,
    pub events: Vec<Event>,
}