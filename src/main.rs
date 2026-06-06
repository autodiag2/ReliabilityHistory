use fltk::{app, button::Button, frame::Frame, prelude::*, text::TextBuffer, text::TextDisplay, window::Window};
use serde::{Deserialize, Serialize};
use sd_journal::{CursorMovement, FileFlags, Journal, UserFlags};
use std::thread;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Event {
    id: String,
    timestamp: String,
    application: String,
    reason: String,
}

enum Message {
    Loading,
    Loaded(Vec<Event>),
    Clear,
}

fn main() {
    let app = app::App::default();
    let mut wind = Window::new(100, 100, 900, 620, "Reliability History");
    let mut load = Button::new(20, 20, 120, 40, "Load events");
    let mut clear = Button::new(160, 20, 120, 40, "Clear");
    let mut status = Frame::new(300, 20, 560, 40, "Press Load events to query journald.");
    let mut display = TextDisplay::new(20, 80, 860, 520, "");
    let mut buffer = TextBuffer::default();
    display.set_buffer(buffer.clone());
    wind.end();
    wind.show();

    let (s, r) = app::channel::<Message>();
    let s_load = s.clone();
    load.set_callback(move |_| {
        s_load.send(Message::Loading);
        let tx = s_load.clone();
        thread::spawn(move || {
            let events = collect_events_from_journal_api().unwrap_or_default();
            tx.send(Message::Loaded(events));
        });
    });

    clear.set_callback(move |_| {
        s.send(Message::Clear);
    });

    while app.wait() {
        if let Some(msg) = r.recv() {
            match msg {
                Message::Loading => {
                    status.set_label("Loading events from journald...");
                }
                Message::Loaded(events) => {
                    status.set_label(&format!("Loaded {} events.", events.len()));
                    buffer.set_text(&format_events(&events));
                }
                Message::Clear => {
                    status.set_label("Cleared event list.");
                    buffer.set_text("");
                }
            }
        }
    }
}

fn format_events(events: &[Event]) -> String {
    let mut text = String::new();
    for event in events {
        text.push_str(&format!("{} | {} | {}\n", event.timestamp, event.application, event.reason));
    }
    if text.is_empty() {
        text.push_str("No events loaded.");
    }
    text
}

/// Collects up to 24 recent crash-like journal entries and returns them without storing details.
fn collect_events_from_journal_api() -> Result<Vec<Event>, String> {
    let journal = Journal::open(FileFlags::AllFiles, UserFlags::AllUsers)
        .map_err(|e| format!("failed to open journal: {:?}", e))?;

    journal.seek_tail().map_err(|e| format!("seek failed: {:?}", e))?;
    let mut events = Vec::new();

    loop {
        match journal.previous() {
            Ok(CursorMovement::Done) | Ok(CursorMovement::Limited(_)) => {
                let message = journal.get_data("MESSAGE").unwrap_or_default();
                let exe = journal.get_data("_EXE").unwrap_or_default();
                let comm = journal.get_data("_COMM").unwrap_or_default();
                let timestamp = journal.get_data("__REALTIME_TIMESTAMP").unwrap_or_default();
                let id = journal.get_data("_UID").unwrap_or_else(|_| format!("journal-{}", events.len()));
                let is_crash = message.to_lowercase().contains("segfault")
                    || message.to_lowercase().contains("panic")
                    || message.to_lowercase().contains("traceback")
                    || journal.get_data("_PID").is_ok();

                if is_crash {
                    let application = if !exe.is_empty() { exe } else { comm };
                    let reason = message.clone();
                    events.push(Event {
                        id,
                        timestamp,
                        application,
                        reason,
                    });
                }

                if events.len() >= 24 {
                    break;
                }
            }
            Ok(CursorMovement::EoF) => break,
            Err(e) => return Err(format!("journal iteration error: {:?}", e)),
        }
    }

    Ok(events)
}
