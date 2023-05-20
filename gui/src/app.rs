use std::{fs::read_to_string, path::PathBuf};

use anyhow::Result;
use eframe::egui::{self, RichText};
use rfd::FileDialog;

use lib::{Config, TaskID};

use crate::widgets::{add_task, add_test, AddTaskState, AddTestState};

pub(crate) const CONFIG_PATH_STORAGE_KEY: &str = "config_path";

#[derive(Debug, Default)]
pub(crate) struct App {
    config_path: Option<PathBuf>,
    config: Option<Config>,
    app_state: AppState,
    error_state: Option<String>,
}

impl App {
    pub(crate) fn new(config_path: Option<PathBuf>) -> Self {
        Self {
            config_path,
            ..Self::default()
        }
    }
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
            ui.horizontal(|ui| {
                if ui.button("Select config").clicked() {
                    self.select_config();
                }
                if self.config_path.is_some() && ui.button("Reload config").clicked() {
                    self.config = None;
                }
                if self.config_path.is_some()
                    && ui
                        .button("Create config")
                        .on_hover_text("Create and save a minimal config")
                        .clicked()
                {
                    self.create_default_config();
                }
            });

            if let Some(config_path) = &self.config_path {
                ui.label(format!("Config: {}", config_path.display()));
                if self.config.is_none() {
                    self.read_config();
                }

                if let Some(config) = &self.config {
                    ui.heading("Tasks");
                    for t in config.tasks() {
                        ui.horizontal(|ui| {
                            if ui.button("add test").clicked() {
                                self.app_state =
                                    AppState::AddTest(t.id.clone(), AddTestState::default());
                            }
                            ui.label(RichText::new(t.format()).strong());
                        });
                    }
                    if ui.button("Add task").clicked() {
                        self.app_state = AppState::AddTask(AddTaskState::default());
                    }
                }
            }

            match &mut self.app_state {
                AppState::AddTask(ref mut state) => {
                    if ui.add(add_task(state)).clicked() {
                        if let Some(ref mut config) = self.config {
                            config.add_task(&state.id, &state.name);
                        }
                        match self.save_config() {
                            Ok(app_state) => self.app_state = app_state,
                            Err(err_state) => self.error_state = Some(err_state),
                        };
                    }
                }
                AppState::AddTest(task_id, ref mut state) => {
                    if ui.add(add_test(state, task_id.clone())).clicked() {
                        if let Some(ref mut config) = self.config {
                            config.add_test_to_task(&task_id, &state.input, &state.expected);
                        }
                        match self.save_config() {
                            Ok(app_state) => self.app_state = app_state,
                            Err(err_state) => self.error_state = Some(err_state),
                        };
                    }
                }
                AppState::Msg(msg) => {
                    ui.label(msg.clone());
                }
                AppState::None => (),
            };
            if let Some(e) = &self.error_state {
                ui.label(format!("An error occured: {e}"));
            }
        });
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Some(config_path) = &self.config_path {
            storage.set_string(CONFIG_PATH_STORAGE_KEY, config_path.display().to_string())
        }
    }
}

impl App {
    fn select_config(&mut self) {
        let mut picker = FileDialog::new().add_filter("config", &["toml"]);
        if let Some(config_path) = &self.config_path {
            if let Some(config_dir) = config_path.parent() {
                picker = picker.set_directory(config_dir);
            }
        }
        if let Some(path) = picker.pick_file() {
            if let Some(config_path) = &self.config_path {
                if path != *config_path {
                    self.config_path = Some(path);
                    self.config = None;
                }
            } else {
                self.config_path = Some(path);
            }
        } else {
            self.error_state = Some("cannot select config".into());
        }
    }
    fn create_default_config(&mut self) {
        let mut saver = FileDialog::new().set_file_name("cdf.toml");
        if let Some(config_path) = &self.config_path {
            if let Some(config_dir) = config_path.parent() {
                saver = saver.set_directory(config_dir);
            }
        }
        if let Some(path) = saver.save_file() {
            self.config_path = Some(path);
            self.config = Some(Config::default());
            match self.save_config() {
                Ok(app_state) => self.app_state = app_state,
                Err(err_state) => self.error_state = Some(err_state),
            };
        } else {
            self.error_state = Some("cannot save config".into());
        }
    }
    fn read_config(&mut self) {
        if let Some(config_path) = &self.config_path {
            match read_config(config_path) {
                Ok(c) => {
                    self.config = Some(c);
                    self.app_state = AppState::default();
                    self.error_state = None;
                }
                Err(e) => {
                    self.error_state = Some(e);
                }
            }
        }
    }
    fn save_config(&self) -> Result<AppState, String> {
        if let Some(config_path) = &self.config_path {
            if let Some(ref config) = self.config {
                match config.save_config_to(config_path.into()) {
                    Ok(_) => return Ok(AppState::Msg("Config saved".into())),
                    Err(e) => {
                        return Err(format!(
                            "cannot save config to {}: {e}",
                            config_path.display()
                        ))
                    }
                }
            } else {
                return Err("self.config is empty when saving config. THIS IS A BUG!".into());
            }
        } else {
            return Err("self.config_path is empty when saving config. THIS IS A BUG!".into());
        }
    }
}

fn read_config(path: &PathBuf) -> Result<Config, String> {
    if let Err(e) = path.try_exists() {
        return Err(format!("{} does not exists: {e}", path.display()));
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
