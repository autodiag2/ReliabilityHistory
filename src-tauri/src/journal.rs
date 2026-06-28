use chrono::{Local, TimeZone};
use sd_journal::{CursorMovement, FileFlags, Journal, UserFlags};

use crate::model::{Event, EventKind};
use users::{get_user_by_uid};

struct Score {
    total: i32,
    details: Vec<String>,
}

impl Score {
    fn add(&mut self, points: i32, reason: impl Into<String>) {
        self.total += points;
        self.details
            .push(format!("{:+4} {}", points, reason.into()));
    }
}

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

                let transport =
                    journal.get_data("_TRANSPORT").unwrap_or_default();

                let comm =
                    journal.get_data("_COMM").unwrap_or_default();

                let exe =
                    journal.get_data("_EXE").unwrap_or_default();

                let unit =
                    journal.get_data("_SYSTEMD_UNIT").unwrap_or_default();

                let priority = journal
                    .get_data("PRIORITY")
                    .ok()
                    .and_then(|s| s.parse::<u8>().ok())
                    .unwrap_or(5);

                let has_coredump =
                    journal.get_data("COREDUMP_PID").is_ok()
                        || journal.get_data("COREDUMP_EXE").is_ok();

                let mut app_failure = Score {
                    total: 0,
                    details: Vec::new(),
                };

                let mut system_failure = Score {
                    total: 0,
                    details: Vec::new(),
                };

                let mut app_warning = Score {
                    total: 0,
                    details: Vec::new(),
                };

                let mut system_warning = Score {
                    total: 0,
                    details: Vec::new(),
                };

                if has_coredump {
                    app_failure.add(100, "COREDUMP_* present");
                }

                for pattern in [
                    "segfault",
                    "core dumped",
                    "sigsegv",
                    "sigabrt",
                    "panic",
                    "assertion",
                    "assertion failed",
                    "fatal",
                    "stack smashing",
                    "stack overflow",
                    "abort",
                    "terminated by signal",
                ] {
                    if lower.contains(pattern) {
                        app_failure.add(25, format!("MESSAGE contains '{pattern}'"));
                    }
                }

                for pattern in [
                    "kernel panic",
                    "watchdog",
                    "soft lockup",
                    "hard lockup",
                    "rcu stall",
                    "hung task",
                    "oops",
                    "call trace",
                    "machine check",
                    "mce",
                    "i/o error",
                    "filesystem corruption",
                    "ext4-fs error",
                    "btrfs error",
                    "xfs error",
                    "oom killer",
                    "out of memory",
                    "memory corruption",
                    "nvme timeout",
                    "fatal",
                ] {
                    if lower.contains(pattern) {
                        system_failure.add(25, format!("MESSAGE contains '{pattern}'"));
                    }
                }

                if lower.contains("warning") {
                    app_warning.add(20, "MESSAGE contains 'warning'");
                    system_warning.add(20, "MESSAGE contains 'warning'");
                }

                if app_failure.total == 0 && system_failure.total == 0 && system_warning.total == 0 && app_warning.total == 0 {
                    continue;
                }

                if transport == "kernel" {
                    system_failure.add(100, "_TRANSPORT=kernel");
                    system_warning.add(100, "_TRANSPORT=kernel");
                }

                match comm.as_str() {
                    "kernel" => system_failure.add(80, "_COMM=kernel"),
                    "systemd"
                    | "systemd-oomd"
                    | "systemd-journald"
                    | "watchdog"
                    | "kworker" => {
                        system_failure.add(60, format!("_COMM={comm}"));
                        system_warning.add(60, format!("_COMM={comm}"));
                    }
                    _ => {}
                }

                if unit.starts_with("user@") {
                    app_failure.add(20, format!("_SYSTEMD_UNIT={unit}"));
                    app_warning.add(20, format!("_SYSTEMD_UNIT={unit}"));
                }

                let user_id = journal
                    .get_data("_UID")
                    .ok()
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(u32::MAX);

                if user_id == u32::MAX {
                    system_failure.add(20, "_UID not found");
                    system_warning.add(20, "_UID not found");
                } else {
                    app_failure.add(20, format!("_UID={user_id}"));
                    app_warning.add(20, format!("_UID={user_id}"));
                }

                if exe.starts_with("/home/")
                    || exe.starts_with("/opt/")
                    || exe.starts_with("/snap/")
                    || exe.starts_with("/usr/local/")
                {
                    app_failure.add(20, format!("executable={exe}"));
                    app_warning.add(20, format!("executable={exe}"));
                }

                if exe.starts_with("/usr/lib/systemd")
                    || exe.starts_with("/usr/lib/modules")
                    || exe.starts_with("/lib/modules")
                {
                    system_failure.add(40, format!("executable={exe}"));
                    system_warning.add(40, format!("executable={exe}"));
                }

                let candidates = [
                    (
                        EventKind::ApplicationFailure,
                        "ApplicationFailure",
                        &app_failure,
                    ),
                    (
                        EventKind::SystemFailure,
                        "SystemFailure",
                        &system_failure,
                    ),
                    (
                        EventKind::ApplicationWarning,
                        "ApplicationWarning",
                        &app_warning,
                    ),
                    (
                        EventKind::SystemWarning,
                        "SystemWarning",
                        &system_warning,
                    ),
                ];

                let Some((kind, name, score)) = candidates
                    .into_iter()
                    .max_by_key(|(_, _, score)| score.total)
                else {
                    continue;
                };

                if score.total == 0 {
                    continue;
                }

                let classifier_reason = format!(
                    "{name}\n\
                scores:\n\
                ApplicationFailure={}\n\
                SystemFailure={}\n\
                ApplicationWarning={}\n\
                SystemWarning={}\n\n{}",
                    app_failure.total,
                    system_failure.total,
                    app_warning.total,
                    system_warning.total,
                    score.details.join("\n"),
                );

                let naive = match journal.get_realtime() {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                let user = get_user_by_uid(user_id)
                    .map(|u| u.name().to_string_lossy().into_owned())
                    .unwrap_or_default();

                let timestamp = Local.from_utc_datetime(&naive);

                let application = if !comm.is_empty() {
                    comm.clone()
                } else if !exe.is_empty() {
                    exe.clone()
                } else {
                    "unknown".to_string()
                };

                let exec_path = if exe.is_empty() {
                    "unknown".to_string()
                } else {
                    exe
                };

                events.push(Event {
                    id: format!("journal-{}", events.len()),
                    timestamp,
                    application,
                    exec_path,
                    reason: message,
                    kind,
                    user,
                    user_id: user_id as i64,
                    classifier_reason: classifier_reason,
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