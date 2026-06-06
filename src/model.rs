use chrono::{DateTime, Local};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EventKind {
    ApplicationFailure,
    SystemFailure,
    Warning,
    Information,
}

#[derive(Clone, Debug)]
pub struct Event {
    pub id: String,
    pub timestamp: DateTime<Local>,
    pub application: String,
    pub reason: String,
    pub kind: EventKind,
}

#[derive(Clone)]
pub struct DaySummary {
    pub day: String,
    pub score: f32,
    pub events: Vec<Event>,
}