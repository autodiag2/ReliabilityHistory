use std::collections::BTreeMap;

use chrono::{Datelike, Duration, Local, NaiveDate};

use crate::model::{DaySummary, Event, EventKind};

pub fn build_days(events: &[Event]) -> Vec<DaySummary> {
    let mut days = BTreeMap::<NaiveDate, Vec<Event>>::new();

    // Pre-populate the last 30 days with empty event lists.
    let today = Local::now().date_naive();

    for offset in (0..30).rev() {
        let day = today - Duration::days(offset);
        days.insert(day, Vec::new());
    }

    // Add events that fall within the last 30 days.
    for event in events {
        let date = event.timestamp.date_naive();

        if days.contains_key(&date) {
            days.entry(date).or_default().push(event.clone());
        }
    }

    let mut result = Vec::with_capacity(30);
    let mut score = 10.0f32;

    for (day, events) in days {
        for ev in &events {
            score -= match ev.kind {
                EventKind::ApplicationFailure => 1.0,
                EventKind::SystemFailure => 1.5,
                EventKind::Warning => 0.0,
                EventKind::Information => 0.0,
            };
        }

        score = score.max(1.0);

        result.push(DaySummary {
            day: format!(
                "{:04}-{:02}-{:02}",
                day.year(),
                day.month(),
                day.day()
            ),
            score,
            events,
        });

        // Recovery for the next day.
        score = (score + 0.1).min(10.0);
    }

    result
}