use std::collections::BTreeMap;

use chrono::{Datelike, Local};
use egui::{CentralPanel, FontData, FontDefinitions, FontFamily};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    todos: BTreeMap<u64, Task>,
    #[serde(skip)]
    task: String,
    daily: bool,
    id: u64,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
pub struct Task {
    pub name: String,
    pub done: bool,
    pub period: Interval,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub enum Interval {
    Daily(u8),
    Weekly(u8),
}

impl Interval {
    fn extract(&self) -> u8 {
        match self {
            Self::Daily(s) => *s,
            Self::Weekly(s) => *s,
        }
    }
}

impl Default for Task {
    fn default() -> Self {
        Self {
            name: "todo".to_owned(),
            done: false,
            period: Interval::Daily(1),
        }
    }
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            todos: BTreeMap::new(),
            task: "".to_owned(),
            id: 0,
            daily: false,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "my_font".to_owned(),
            FontData::from_static(include_bytes!("../assets/VictorMono-Medium.ttf")),
        ); // .ttf and .otf supported

        // Put my font first (highest priority):
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "my_font".to_owned());

        cc.egui_ctx.set_fonts(fonts);
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        let now = Local::now();
        for (&_id, task) in &mut self.todos {
            match task.period {
                Interval::Daily(_) => {
                    if task.period.extract() < now.day() as u8 {
                        task.done = false;
                        task.period = Interval::Daily(now.day() as u8);
                    }
                }
                Interval::Weekly(_) => {
                    if task.period.extract() < now.iso_week().week() as u8 {
                        task.done = false;
                        task.period = Interval::Weekly(now.iso_week().week() as u8);
                    }
                }
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Everyframe");

            let mut to_remove = Vec::new();
            ui.columns(2, |cols| {
                for (&id, task) in &mut self.todos {
                    match task.period {
                        Interval::Daily(_) => {
                            cols[0].horizontal(|ui| {
                                ui.checkbox(&mut task.done, "");
                                ui.label(task.name.clone() + " [D]");
                                if ui.button("Remove").clicked() {
                                    to_remove.push(id);
                                }
                            });
                        }
                        Interval::Weekly(_) => {
                            cols[1].horizontal(|ui| {
                                ui.checkbox(&mut task.done, "");
                                ui.label(task.name.clone() + " [W]");
                                if ui.button("Remove").clicked() {
                                    to_remove.push(id);
                                }
                            });
                        }
                    }
                }
            });
            for id in to_remove {
                self.todos.remove(&id);
            }
            ui.separator();

            ui.horizontal(|ui| {
                // ui.label("add new task: ");
                ui.checkbox(&mut self.daily, "daily?");
                ui.text_edit_singleline(&mut self.task);
                if ui.button("Add new").clicked() {
                    self.todos.insert(
                        self.id,
                        Task {
                            done: false,
                            name: self.task.clone(),
                            period: if self.daily {
                                Interval::Daily(1)
                            } else {
                                Interval::Weekly(1)
                            },
                        },
                    );
                    self.id += 1;
                }
            });

            print!("{:?}", self.todos);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}
