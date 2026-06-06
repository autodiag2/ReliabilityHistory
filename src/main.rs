use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use sd_journal::{Journal, JournalFiles, JournalSeek};

#[derive(Debug, Serialize, Deserialize)]
struct Event {
    id: String,
    timestamp: String,
    application: String,
    reason: String,
    details: String,
}

fn main() {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("collect") => {
            if let Err(err) = collect_events() {
                eprintln!("Collection failed: {}", err);
                std::process::exit(1);
            }
        }
        Some("report") => {
            let application = args.next().unwrap_or_else(|| "unknown".into());
            let reason = args.next().unwrap_or_else(|| "crash".into());
            if let Err(err) = write_manual_report(&application, &reason) {
                eprintln!("Report failed: {}", err);
                std::process::exit(1);
            }
        }
        _ => {
            print_help();
        }
    }
}

fn print_help() {
    println!("Reliability History collector\n");
    println!("Usage:");
    println!("  reliability_history collect          # scan crash reports and update events.json");
    println!("  reliability_history report APP REASON # append a manual crash event");
}

fn collect_events() -> io::Result<()> {
    // Try collecting events from systemd journal first
    match collect_events_from_journal() {
        Ok(events) if !events.is_empty() => {
            let data_dir = Path::new("data");
            fs::create_dir_all(&data_dir)?;
            let output = File::create(data_dir.join("events.json"))?;
            serde_json::to_writer_pretty(output, &events)?;
            println!("Collected {} events from journal into {}.", events.len(), data_dir.join("events.json").display());
            return Ok(());
        }
        _ => {
            // fallback to file-based collection
        }
    }

    // Existing file-based fallback
    let data_dir = Path::new("data");
    let crash_dir = data_dir.join("crash_reports");
    fs::create_dir_all(&crash_dir)?;
    fs::create_dir_all(&data_dir)?;

    let mut events = Vec::new();
    let mut entries = fs::read_dir(&crash_dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.metadata().and_then(|meta| meta.modified()).ok());
    for entry in entries.into_iter().rev() {
        let path = entry.path();
        if path.is_file() {
            if let Ok(event) = parse_crash_file(&path) {
                events.push(event);
            }
        }
        if events.len() >= 24 {
            break;
        }
    }

    if events.is_empty() {
        println!("No crash report files found in {}.", crash_dir.display());
    }

    let output = File::create(data_dir.join("events.json"))?;
    serde_json::to_writer_pretty(output, &events)?;
    println!("Collected {} events into {}.", events.len(), data_dir.join("events.json").display());
    Ok(())
}

fn collect_events_from_journal() -> Result<Vec<Event>, String> {
    let mut journal = Journal::open(JournalFiles::All, false, false)
        .map_err(|e| format!("failed to open journal: {}", e))?;

    // Seek to the end and walk backwards collecting most recent entries
    journal.seek(JournalSeek::Tail).map_err(|e| format!("seek failed: {}", e))?;

    let mut events = Vec::new();
    // Iterate backwards and collect entries that look like crashes
    // We stop when we have 24 events or when we hit the beginning
    loop {
        match journal.previous_entry() {
            Ok(true) => {
                let mut map = std::collections::HashMap::new();
                if let Ok(entry) = journal.get_entry() {
                    for (k, v) in entry {
                        if let Ok(val) = String::from_utf8(v.clone()) {
                            map.insert(k.to_uppercase(), val);
                        }
                    }

                    // Heuristic: look for fields indicating a crash: _EXE, _COMM, MESSAGE containing 'segfault' or 'panic'
                    let message = map.get("MESSAGE").cloned().unwrap_or_default();
                    let exe = map.get("_EXE").cloned().unwrap_or_default();
                    let comm = map.get("_COMM").cloned().unwrap_or_default();

                    let is_crash = message.to_lowercase().contains("segfault")
                        || message.to_lowercase().contains("panic")
                        || message.to_lowercase().contains("traceback")
                        || map.get("_PID").is_some();

                    if is_crash {
                        let id = map.get("_UID").cloned().unwrap_or_else(|| format!("journal-{}", events.len()));
                        let timestamp = map.get("__REALTIME_TIMESTAMP").cloned().unwrap_or_else(|| "".into());
                        let application = if !exe.is_empty() { exe } else { comm };
                        let reason = message.clone();
                        let details = format!("fields: {}", map.keys().cloned().collect::<Vec<_>>().join(", "));

                        events.push(Event {
                            id,
                            timestamp,
                            application,
                            reason,
                            details,
                        });
                    }
                }

                if events.len() >= 24 {
                    break;
                }
            }
            Ok(false) => break,
            Err(e) => return Err(format!("journal iteration error: {}", e)),
        }
    }

    Ok(events)
}

fn parse_crash_file(path: &Path) -> io::Result<Event> {
    let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    let filename = path.file_stem().and_then(|stem| stem.to_str()).unwrap_or("unknown");
    let metadata = path.metadata()?;
    let modified = metadata.modified().unwrap_or(SystemTime::now());
    let timestamp = format_timestamp(modified);

    if ext.eq_ignore_ascii_case("json") {
        let file = File::open(path)?;
        let event: Event = serde_json::from_reader(file).unwrap_or_else(|_| Event {
            id: filename.to_string(),
            timestamp: timestamp.clone(),
            application: "unknown".into(),
            reason: "invalid json".into(),
            details: fs::read_to_string(path).unwrap_or_default(),
        });
        return Ok(event);
    }

    let text = fs::read_to_string(path)?;
    Ok(Event {
        id: filename.to_string(),
        timestamp,
        application: extract_value(&text, "application").unwrap_or_else(|| filename.to_string()),
        reason: extract_value(&text, "reason").unwrap_or_else(|| "crash report".into()),
        details: text.trim().to_string(),
    })
}

fn extract_value(text: &str, key: &str) -> Option<String> {
    for line in text.lines() {
        let normalized = line.trim();
        if normalized.to_lowercase().starts_with(&format!("{}:", key)) {
            return Some(normalized.splitn(2, ':').nth(1)?.trim().to_string());
        }
    }
    None
}

fn format_timestamp(time: SystemTime) -> String {
    let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    let local = chrono_from_unix(secs);
    local
}

fn chrono_from_unix(seconds: u64) -> String {
    use std::time::Duration;
    let ts = chrono::NaiveDateTime::from_timestamp_opt(seconds as i64, 0)
        .unwrap_or_else(|| chrono::NaiveDateTime::from_timestamp(0, 0));
    let datetime: chrono::DateTime<chrono::Local> = chrono::DateTime::from_utc(ts, *chrono::Local::now().offset());
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn write_manual_report(application: &str, reason: &str) -> io::Result<()> {
    let data_dir = Path::new("data");
    let crash_dir = data_dir.join("crash_reports");
    fs::create_dir_all(&crash_dir)?;
    let id = timestamp_id();
    let path = crash_dir.join(format!("{}.json", id));

    let event = Event {
        id: id.clone(),
        timestamp: current_timestamp(),
        application: application.to_string(),
        reason: reason.to_string(),
        details: format!("Manual report created by the reliability history collector."),
    };

    let mut file = File::create(path)?;
    serde_json::to_writer_pretty(&mut file, &event)?;
    file.write_all(b"\n")?;
    println!("Saved manual report {}.", id);
    Ok(())
}

fn timestamp_id() -> String {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("event-{}", now.as_secs())
}

fn current_timestamp() -> String {
    chrono_from_unix(SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs())
}
