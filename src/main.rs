mod journal;
mod model;
mod reliability;
mod timeline;

use eframe::egui;

use model::DaySummary;

struct ReliabilityApp {
    days: Vec<DaySummary>,
    selected_day: usize,
}

impl ReliabilityApp {
    fn new() -> Self {
        let events = journal::collect_events();
        let days = reliability::build_days(&events);

        Self {
            days,
            selected_day: 0,
        }
    }
}

impl eframe::App for ReliabilityApp {
    fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        egui::TopBottomPanel::top("header").show(
            ctx,
            |ui| {
                ui.heading(
                    "Review your computer's reliability and problem history",
                );
            },
        );

        egui::CentralPanel::default().show(ctx, |ui| {
            timeline::draw(
                ui,
                &self.days,
                &mut self.selected_day,
            );

            ui.separator();

            if let Some(day) =
                self.days.get(self.selected_day)
            {
                ui.heading(format!(
                    "Reliability details for {}",
                    day.day
                ));

                egui::ScrollArea::vertical()
                    .max_height(260.0)
                    .show(ui, |ui| {
                        egui::Grid::new("details")
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("Source");
                                ui.label("Summary");
                                ui.label("Date");
                                ui.end_row();

                                for ev in &day.events {
                                    ui.label(&ev.application);

                                    ui.label(
                                        ev.reason
                                            .chars()
                                            .take(80)
                                            .collect::<String>(),
                                    );

                                    ui.label(
                                        ev.timestamp
                                            .format(
                                                "%Y-%m-%d %H:%M",
                                            )
                                            .to_string(),
                                    );

                                    ui.end_row();
                                }
                            });
                    });
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 850.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Reliability Monitor",
        options,
        Box::new(|_| Ok(Box::new(ReliabilityApp::new()))),
    )
}