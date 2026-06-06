use std::collections::BTreeMap;

use chrono::Datelike;

use crate::model::{DaySummary, Event, EventKind};

pub fn build_days(events: &[Event]) -> Vec<DaySummary> {
    let mut days = BTreeMap::<String, Vec<Event>>::new();

    for event in events {
        let key = format!(
            "{:04}-{:02}-{:02}",
            event.timestamp.year(),
            event.timestamp.month(),
            event.timestamp.day()
        );

        days.entry(key).or_default().push(event.clone());
    }

    let mut result = Vec::new();
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

        if score < 1.0 {
            score = 1.0;
        }

        result.push(DaySummary {
            day,
            score,
            events,
        });

        score += 0.1;

        if score > 10.0 {
            score = 10.0;
        }
    }

    result
}