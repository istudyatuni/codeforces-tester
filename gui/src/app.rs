use std::{fs::read_to_string, path::PathBuf};

use anyhow::Result;
use eframe::egui::{self, RichText};

use lib::{Config, TaskID, TaskInfo};

use crate::widgets::{add_task, add_test, AddTaskState, AddTestState};

#[derive(Debug, Default)]
pub(crate) struct App {
    config_path: Option<PathBuf>,
    config: Option<Config>,
    app_state: AppState,
    error_state: Option<String>,
}

#[derive(Debug, Default)]
enum AppState {
    AddTask(AddTaskState),
    AddTest(TaskID, AddTestState),
    Msg(String),
    #[default]
    None,
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
                                self.app_state = AppState::default()
                            }
                            Err(e) => {
                                self.error_state = Some(e);
                                break 'config;
                            }
                        }
                    }

                    if let Some(config) = &self.config {
                        ui.heading(format!("Tasks"));
                        for t in config.tasks() {
                            ui.horizontal(|ui| {
                                if ui.button("add test").clicked() {
                                    self.app_state =
                                        AppState::AddTest(t.id.clone(), AddTestState::default());
                                }
                                ui.label(RichText::new(format_task_info(&t)).strong());
                            });
                        }
                        if ui.button("Add task").clicked() {
                            self.app_state = AppState::AddTask(AddTaskState::default());
                        }
                    }
                }
            }

            match &mut self.app_state {
                AppState::AddTask(ref mut state) => {
                    if ui.add(add_task(state)).clicked() {
                        if let Some(config_path) = &self.config_path {
                            if let Some(ref mut config) = self.config {
                                config.add_task(state.id.clone(), state.name.clone());
                                match config.save_config_to(config_path.into()) {
                                    Ok(_) => self.app_state = AppState::Msg("Config saved".into()),
                                    Err(e) => {
                                        self.error_state = Some(format!(
                                            "cannot save config to {}: {e}",
                                            config_path.display()
                                        ))
                                    }
                                }
                            }
                        }
                    }
                }
                AppState::AddTest(task_id, ref mut state) => {
                    if ui.add(add_test(state, task_id.clone())).clicked() {
                        if let Some(config_path) = &self.config_path {
                            if let Some(ref mut config) = self.config {
                                config.add_test_to_task(
                                    task_id.clone(),
                                    state.input.clone(),
                                    state.expected.clone(),
                                );
                                match config.save_config_to(config_path.into()) {
                                    Ok(_) => self.app_state = AppState::Msg("Config saved".into()),
                                    Err(e) => {
                                        self.error_state = Some(format!(
                                            "cannot save config to {}: {e}",
                                            config_path.display()
                                        ))
                                    }
                                }
                            }
                        }
                    }
                }
                AppState::Msg(msg) => {
                    ui.label(format!("{msg}"));
                }
                AppState::None => (),
            };
            if let Some(e) = &self.error_state {
                ui.label(format!("An error occured: {e}"));
            }
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
