use std::{fs::read_to_string, path::PathBuf};

use anyhow::Result;
use eframe::egui::{self, RichText};

use lib::{Config, TaskID, TaskInfo};

#[derive(Debug, Default)]
pub(crate) struct App {
    config_path: Option<PathBuf>,
    config: Option<Config>,
    bottom_state: BottomState,
}

#[derive(Debug, Default)]
enum BottomState {
    AddTask(AddTask),
    AddTest(TaskID, AddTest),
    Msg(String),
    Error(String),
    #[default]
    None,
}

#[derive(Debug, Default, Clone)]
struct AddTask {
    id: String,
    name: String,
}

#[derive(Debug, Default, Clone)]
struct AddTest {
    input: String,
    expected: String,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Select config").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    if let Some(config_path) = &self.config_path {
                        if path != *config_path {
                            self.config_path = Some(path);
                            self.config = None;
                        }
                    } else {
                        self.config_path = Some(path);
                    }
                }
            }

            'config: {
                if let Some(config_path) = &self.config_path {
                    ui.label(format!("Config: {}", config_path.display()));
                    if self.config.is_none() {
                        match read_config(config_path) {
                            Ok(c) => {
                                self.config = Some(c);
                                self.bottom_state = BottomState::default()
                            }
                            Err(e) => {
                                self.bottom_state = BottomState::Error(e);
                                break 'config;
                            }
                        }
                    }

                    if let Some(config) = &self.config {
                        ui.heading(format!("Tasks"));
                        for t in config.tasks() {
                            ui.horizontal(|ui| {
                                if ui.button("add test").clicked() {
                                    self.bottom_state =
                                        BottomState::AddTest(t.id.clone(), AddTest::default());
                                }
                                ui.label(RichText::new(format_task_info(&t)).strong());
                            });
                        }
                        if ui.button("Add task").clicked() {
                            self.bottom_state = BottomState::AddTask(AddTask::default());
                        }
                    }
                }
            }

            match &mut self.bottom_state {
                BottomState::AddTask(ref mut state) => {
                    ui.heading(format!("Add task:"));
                    ui.horizontal(|ui| {
                        let id_label = ui.label("ID:");
                        ui.text_edit_singleline(&mut state.id)
                            .labelled_by(id_label.id);
                    });
                    ui.horizontal(|ui| {
                        let name_label = ui.label("Name:");
                        ui.text_edit_singleline(&mut state.name)
                            .labelled_by(name_label.id);
                    });
                    if ui.button("Submit").clicked() {
                        if let Some(config_path) = &self.config_path {
                            if let Some(ref mut config) = self.config {
                                config.add_task(state.id.clone(), state.name.clone());
                                match config.save_config_to(config_path.into()) {
                                    Ok(_) => {
                                        self.bottom_state = BottomState::Msg("Config saved".into())
                                    }
                                    Err(e) => {
                                        self.bottom_state = BottomState::Error(format!(
                                            "cannot save config to {}: {e}",
                                            config_path.display()
                                        ))
                                    }
                                }
                            }
                        }
                    }
                }
                BottomState::AddTest(task_id, ref mut state) => {
                    ui.heading(format!("Add test for task {}:", task_id.to_uppercase()));
                    let id_label = ui.label("Input:");
                    ui.text_edit_multiline(&mut state.input)
                        .labelled_by(id_label.id);
                    let id_label = ui.label("Expected:");
                    ui.text_edit_multiline(&mut state.expected)
                        .labelled_by(id_label.id);
                    if ui.button("Submit").clicked() {
                        if let Some(config_path) = &self.config_path {
                            if let Some(ref mut config) = self.config {
                                config.add_test_to_task(
                                    task_id.clone(),
                                    state.input.clone(),
                                    state.expected.clone(),
                                );
                                match config.save_config_to(config_path.into()) {
                                    Ok(_) => {
                                        self.bottom_state = BottomState::Msg("Config saved".into())
                                    }
                                    Err(e) => {
                                        self.bottom_state = BottomState::Error(format!(
                                            "cannot save config to {}: {e}",
                                            config_path.display()
                                        ))
                                    }
                                }
                            }
                        }
                    }
                }
                BottomState::Msg(msg) => {
                    ui.label(format!("{msg}"));
                }
                BottomState::Error(e) => {
                    ui.label(format!("An error occured: {e}"));
                }
                BottomState::None => (),
            };
        });
    }
}

fn read_config(path: &PathBuf) -> Result<Config, String> {
    match path.try_exists() {
        Err(e) => return Err(format!("{} does not exists: {e}", path.display())),
        _ => (),
    };
    let s = match read_to_string(path) {
        Ok(s) => s,
        Err(e) => return Err(format!("cannot read config: {e}")),
    };
    match Config::try_from(s.as_str()) {
        Ok(c) => Ok(c),
        Err(e) => Err(format!("cannot parse config: {e}")),
    }
}

fn format_task_info(t: &TaskInfo) -> String {
    format!(
        "{} - {}, {} tests",
        t.id.to_uppercase(),
        t.name,
        t.tests_count
    )
}
