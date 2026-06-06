use chrono::{Local, TimeZone};
use sd_journal::{CursorMovement, FileFlags, Journal, UserFlags};

use crate::model::{Event, EventKind};

pub fn collect_events() -> Vec<Event> {
    let journal = match Journal::open(
        FileFlags::AllFiles,
        UserFlags::AllUsers,
    ) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let _ = journal.seek_tail();

    let mut events = Vec::new();

    loop {
        match journal.previous() {
            Ok(CursorMovement::Done)
            | Ok(CursorMovement::Limited(_)) => {
                let message =
                    journal.get_data("MESSAGE").unwrap_or_default();

                let lower = message.to_lowercase();

                let kind = if lower.contains("segfault")
                    || lower.contains("core dumped")
                    || lower.contains("panic")
                    || lower.contains("assertion")
                {
                    EventKind::ApplicationFailure
                } else if lower.contains("oom")
                    || lower.contains("out of memory")
                    || lower.contains("kernel panic")
                    || lower.contains("watchdog")
                {
                    EventKind::SystemFailure
                } else if lower.contains("warning")
                {
                    EventKind::Warning
                } else {
                    continue;
                };

                let naive = match journal.get_realtime() {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                let timestamp = Local.from_utc_datetime(&naive);

                let application = journal
                    .get_data("_COMM")
                    .or_else(|_| journal.get_data("_EXE"))
                    .unwrap_or_else(|_| "unknown".to_string());

                events.push(Event {
                    id: format!("journal-{}", events.len()),
                    timestamp,
                    application,
                    reason: message,
                    kind,
                });

                if events.len() >= 500 {
                    break;
                }
            }
            Ok(CursorMovement::EoF) => break,
            Err(_) => break,
        }
    }

    events.reverse();
    events
}