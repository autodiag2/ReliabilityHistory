mod journal;
mod model;
mod reliability;
use gio;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};

use serde_json::json;

use webkit2gtk::UserContentManager;
use webkit2gtk::{
    WebView
};

fn main() {
    let app = Application::builder()
        .application_id(
            "com.example.reliability"
        )
        .build();

    app.connect_activate(build_ui);

    app.run();
}

fn build_ui(app: &Application) {

    let events =
        journal::collect_events();

    let days =
        reliability::build_days(&events);

    let manager =
        UserContentManager::new();

    manager
        .register_script_message_handler(
            "app"
        )
        .unwrap();

    let webview =
        WebView::with_user_content_manager(
            &manager
        );

    let html =
        include_str!("ui.html");

    webview.load_html(
        html,
        Some("file:///"),
    );

    let days_json =
        serde_json::to_string(&days)
            .unwrap();

    let webview_clone =
        webview.clone();

    webview.connect_load_changed(
        move |_, _| {

            let js = format!(
                "updateDays({});",
                days_json
            );

            webview_clone
                .run_javascript(
                    &js,
                    None::<&gio::Cancellable>,
                    |_| {},
                );
        }
    );

    let days_clone =
        days.clone();

    let webview_clone =
        webview.clone();

    manager.connect_script_message_received(
        None,
        move |_, msg| {

            let Some(value) =
                msg.js_value()
            else {
                return;
            };

            let Some(text) =
                value.to_string()
            else {
                return;
            };

            let Ok(v) =
                serde_json::from_str::<serde_json::Value>(&text)
            else {
                return;
            };

            if v["action"] == "select_day" {

                let idx =
                    v["day"]
                        .as_u64()
                        .unwrap_or(0)
                        as usize;

                if let Some(day) =
                    days_clone.get(idx)
                {
                    let js = format!(
                        "showEvents({});",
                        serde_json::to_string(
                            &day.events
                        ).unwrap()
                    );

                    webview_clone
                        .run_javascript(
                            &js,
                            None::<&gio::Cancellable>,
                            |_| {},
                        );
                }
            }
        }
    );

    let window =
        ApplicationWindow::builder()
            .application(app)
            .title(
                "Reliability Monitor"
            )
            .default_width(1400)
            .default_height(850)
            .child(&webview)
            .build();

    window.show();
}