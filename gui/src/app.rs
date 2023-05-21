use std::{fs::read_to_string, path::PathBuf};

use eframe::egui::{self, RichText};
use rfd::FileDialog;

use lib::{Config, TaskID};

use crate::errors::{Error, ErrorKind, ErrorsMap};
use crate::widgets::{add_task, add_test, edit_task, AddTaskState, AddTestState, EditTaskState};

pub(crate) const CONFIG_PATH_STORAGE_KEY: &str = "config_path";

#[derive(Debug, Default)]
pub(crate) struct App {
    config_path: Option<PathBuf>,
    config: Option<Config>,
    app_state: AppState,
    post_update: PostUpdate,
    errors: ErrorsMap,
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
    EditTask(TaskID, EditTaskState),
    AddTest(TaskID, AddTestState),
    Msg(String),
    #[default]
    None,
}

#[derive(Debug, Default)]
enum PostUpdate {
    SaveConfig,
    #[default]
    None,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.post_update = Default::default();

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
                            if ui.button("edit").clicked() {
                                self.app_state = AppState::EditTask(
                                    t.id.clone(),
                                    EditTaskState::new(t.id, t.name),
                                );
                            }
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
                            self.post_update = PostUpdate::SaveConfig;
                        }
                    }
                }
                AppState::EditTask(task_id, ref mut state) => {
                    if let Some(ref mut config) = self.config {
                        state.is_task_exists = config.is_task_exists(&state.id);
                        if ui.add(edit_task(state)).clicked() {
                            config.update_task(&task_id, &state.id, &state.name);
                            self.post_update = PostUpdate::SaveConfig;
                        }
                    }
                }
                AppState::AddTest(task_id, ref mut state) => {
                    if ui.add(add_test(state, task_id.clone())).clicked() {
                        if let Some(ref mut config) = self.config {
                            config.add_test_to_task(task_id, &state.input, &state.expected);
                            self.post_update = PostUpdate::SaveConfig;
                        }
                    }
                }
                AppState::Msg(msg) => {
                    ui.label(msg.clone());
                }
                AppState::None => (),
            }

            match self.post_update {
                PostUpdate::SaveConfig => self.save_config(),
                PostUpdate::None => (),
            }

            if !self.errors.is_empty() {
                ui.heading("An errors occured:");
                for (kind, e) in &self.errors {
                    ui.label(format!("{kind}: {e}"));
                }
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

        self.errors.delete(ErrorKind::CannotSelectConfig);
        let Some(path) = picker.pick_file() else {
            return self.errors.add(Error::CannotSelectConfig)
        };

        self.config_path = Some(path);
        self.config = None;
    }
    fn create_default_config(&mut self) {
        let mut saver = FileDialog::new().set_file_name("cdf.toml");
        if let Some(config_path) = &self.config_path {
            if let Some(config_dir) = config_path.parent() {
                saver = saver.set_directory(config_dir);
            }
        }

        self.errors
            .delete(ErrorKind::CannotSelectPathForSavingConfig);
        let Some(path) = saver.save_file() else {
            return self.errors.add(Error::CannotSelectPathForSavingConfig)
        };

        self.config_path = Some(path);
        self.config = Some(Config::default());
        self.save_config();
        // read again to make sure there are no new errors, and delete old errors
        self.read_config();
    }
    fn read_config(&mut self) {
        if let Some(path) = &self.config_path {
            self.errors.delete(ErrorKind::PathNotExists(path.clone()));
            if let Err(e) = path.try_exists() {
                return self
                    .errors
                    .add(Error::PathNotExists(e.to_string(), path.clone()));
            };

            self.errors.delete(ErrorKind::CannotReadConfig);
            let s = match read_to_string(path) {
                Ok(s) => s,
                Err(e) => return self.errors.add(Error::CannotReadConfig(e.to_string())),
            };

            self.errors.delete(ErrorKind::CannotParseConfig);
            let config = match Config::try_from(s.as_str()) {
                Ok(c) => c,
                Err(e) => {
                    return self
                        .errors
                        .add(Error::CannotParseConfig(e.to_string(), path.clone()))
                }
            };

            self.config = Some(config);
            self.app_state = AppState::default();
        }
    }
    fn save_config(&mut self) {
        self.errors
            .delete(ErrorKind::BugConfigPathEmptyWhenSavingConfig);
        let Some(config_path) = &self.config_path else {
            return self.errors.add(Error::BugConfigPathEmptyWhenSavingConfig)
        };

        self.errors
            .delete(ErrorKind::BugConfigEmptyWhenSavingConfig);
        let Some(config) = &self.config else {
            return self.errors.add(Error::BugConfigEmptyWhenSavingConfig)
        };

        self.errors.delete(ErrorKind::CannotSaveConfig);
        match config.save_config_to(config_path) {
            Ok(_) => self.app_state = AppState::Msg("Config saved".into()),
            Err(e) => self.errors.add(Error::CannotSaveConfig(e.to_string())),
        }
    }
}
