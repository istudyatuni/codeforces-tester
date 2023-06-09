use std::{fs::read_to_string, path::PathBuf};

use eframe::egui::{self, Link, RichText, Ui};
use rfd::FileDialog;

use lib::{Config, Error as LibError, TaskID, TestResult};

use crate::errors::{Error, ErrorKind, ErrorsMap};
use crate::widgets::{
    add_task, add_test, edit_task, edit_tests, AddTaskState, AddTestState, EditTaskState,
    EditTestsResponse, EditTestsState,
};

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
            ..Default::default()
        }
    }
}

#[derive(Debug, Default)]
enum AppState {
    AddTask(AddTaskState),
    EditTask(TaskID, EditTaskState),
    AddTest(TaskID, AddTestState),
    EditTests(TaskID, EditTestsState),

    ShowTestsResults(Vec<TestResult>),
    Msg(String),
    #[default]
    None,
}

#[derive(Debug, Default)]
enum PostUpdate {
    SaveConfig,
    OpenConfigInEditor,
    CancelOperation,
    RunTests(TaskID),
    #[default]
    None,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    self.config_select_ui(ui);
                    self.config_content_ui(ui);
                    self.app_state_ui(ui);
                    self.errors_ui(ui);

                    self.handle_post_update();
                });
        });
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Some(config_path) = &self.config_path {
            storage.set_string(CONFIG_PATH_STORAGE_KEY, config_path.display().to_string())
        }
    }
}

/// UI
impl App {
    fn config_select_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("Open config").clicked() {
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
            ui.horizontal(|ui| {
                ui.label("Config:");
                let config_path_str = config_path.display().to_string();
                if ui
                    .add(Link::new(RichText::new(config_path_str).monospace()))
                    .on_hover_text("Open config in text editor")
                    .clicked()
                {
                    self.post_update = PostUpdate::OpenConfigInEditor;
                }
            });
        }
    }
    fn config_content_ui(&mut self, ui: &mut Ui) {
        if self.config.is_none() {
            self.read_config();
        }

        if let Some(config) = &self.config {
            ui.heading("Tasks");
            for t in config.tasks() {
                ui.horizontal(|ui| {
                    if ui.button("edit").clicked() {
                        self.app_state =
                            AppState::EditTask(t.id.clone(), EditTaskState::new(t.id, t.name));
                    }
                    if ui.button("add test").clicked() {
                        self.app_state = AppState::AddTest(t.id.clone(), AddTestState::default());
                    }
                    if ui.button("edit tests").clicked() {
                        self.app_state =
                            AppState::EditTests(t.id.clone(), EditTestsState::new(t.id, t.tests));
                    }
                    if ui.button(RichText::new("run tests").strong()).clicked() {
                        self.post_update = PostUpdate::RunTests(t.id.clone());
                    }
                    ui.label(RichText::new(t.format()).strong());
                });
            }
            if ui.button("Add task").clicked() {
                self.app_state = AppState::AddTask(AddTaskState::default());
            }
        }
    }
    fn app_state_ui(&mut self, ui: &mut Ui) {
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
                    if ui.add(edit_task(state)).clicked() {
                        config.update_task(task_id, &state.id, &state.name);
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
            AppState::EditTests(task_id, ref mut state) => {
                ui.add(edit_tests(state));
                match &state.response {
                    EditTestsResponse::SaveTest((i, test)) => {
                        if let Some(ref mut config) = self.config {
                            config.update_test(task_id, *i, test.clone());
                            self.post_update = PostUpdate::SaveConfig;
                        }
                    }
                    EditTestsResponse::Cancel => self.post_update = PostUpdate::CancelOperation,
                    EditTestsResponse::None => (),
                }
            }
            AppState::ShowTestsResults(results) => {
                for (i, res) in results.iter().enumerate() {
                    match res {
                        TestResult::Ok => {
                            ui.label(format!("test {} ok", i + 1));
                        }
                        TestResult::Failed(f) => {
                            ui.collapsing(format!("test {} failed:", f.index + 1), |ui| {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.strong("Expected output:");
                                        ui.monospace(&f.expected);
                                    });
                                    ui.separator();
                                    ui.vertical(|ui| {
                                        ui.strong("Actual output:");
                                        ui.monospace(f.cmd_output.stdout.trim());
                                    });
                                });
                                if !f.cmd_output.stderr.is_empty() {
                                    ui.strong("Stderr:");
                                    ui.monospace(f.cmd_output.stderr.trim());
                                }
                            });
                        }
                        TestResult::Err(e) => {
                            self.errors.add(Error::CannotRunTests(e.to_string()));
                            break;
                        }
                    };
                }
            }
            AppState::Msg(msg) => {
                ui.label(msg.clone());
            }
            AppState::None => (),
        }
    }
    fn errors_ui(&mut self, ui: &mut Ui) {
        if !self.errors.is_empty() {
            let mut skip_show_errors = false;
            ui.horizontal(|ui| {
                ui.heading("An errors occured:");
                if ui.button("Clear").clicked() {
                    self.errors.clear();
                    skip_show_errors = true;
                }
            });
            if skip_show_errors {
                return;
            }
            for (kind, e) in &self.errors {
                match (kind.to_string().as_str(), e.to_string().as_str()) {
                    ("", "") => ui.label("an error occured"),
                    (e, "") | ("", e) => ui.label(format!("an error occured: {e}")),
                    (kind, e) => ui.label(format!("{kind}: {e}")),
                };
            }
        }
    }
}

/// Logic
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
            self.clear_app_state();
        }
    }
    fn save_config(&mut self) {
        self.errors.delete(ErrorKind::BugConfigPathEmpty);
        let Some(config_path) = &self.config_path else {
            return self.errors.add(Error::BugConfigPathEmpty)
        };

        self.errors.delete(ErrorKind::BugConfigEmpty);
        let Some(config) = &self.config else {
            return self.errors.add(Error::BugConfigEmpty)
        };

        self.errors.delete(ErrorKind::CannotSaveConfig);
        match config.save_config_to(config_path) {
            Ok(_) => self.app_state = AppState::Msg("Config saved".into()),
            Err(e) => self.errors.add(Error::CannotSaveConfig(e.to_string())),
        }
    }
    fn open_config_in_editor(&mut self) {
        self.errors.delete(ErrorKind::BugConfigEmpty);
        let Some(config_path) = &self.config_path else {
            return self.errors.add(Error::BugConfigEmpty);
        };

        self.errors.delete(ErrorKind::CannotOpenConfigInEditor);
        match open::that_in_background(config_path).join() {
            Ok(r) => match r {
                Ok(_) => (),
                Err(e) => self
                    .errors
                    .add(Error::CannotOpenConfigInEditor(e.to_string())),
            },
            Err(_) => self.errors.add(Error::CannotOpenConfigInEditor(
                "error join on thread".into(),
            )),
        }
    }

    fn run_tests(&mut self, id: TaskID) {
        self.errors.delete(ErrorKind::BugConfigEmpty);
        let Some(config) = &self.config else {
            return self.errors.add(Error::BugConfigEmpty);
        };

        self.errors.delete(ErrorKind::BugConfigPathEmpty);
        let Some(config_path) = &self.config_path else {
            return self.errors.add(Error::BugConfigPathEmpty);
        };

        match config.check_task(&id) {
            Ok(_) => (),
            Err(e) => {
                if let LibError::TaskHasNoTests(_) = e {
                    return self.app_state = AppState::Msg("No tests".into());
                }
            }
        }

        let dir = config_path.parent().map(|p| p.into());

        self.errors.delete(ErrorKind::CannotBuildTask);
        if config.should_build() {
            match config.build_from_dir(&id, &dir) {
                Ok(output) => {
                    if let Some(output) = output {
                        if !output.success {
                            return self
                                .errors
                                .add(Error::CannotBuildTask("\n".to_owned() + &output.stderr));
                        }
                    }
                }
                Err(e) => return self.errors.add(Error::CannotBuildTask(e.to_string())),
            }
        }

        let results: Vec<TestResult> = config.run_tests_from_dir(&id, &dir).into_iter().collect();
        self.app_state = AppState::ShowTestsResults(results);
    }

    fn handle_post_update(&mut self) {
        match &self.post_update {
            PostUpdate::SaveConfig => self.save_config(),
            PostUpdate::OpenConfigInEditor => self.open_config_in_editor(),
            PostUpdate::CancelOperation => self.clear_app_state(),
            PostUpdate::RunTests(id) => self.run_tests(id.clone()),
            PostUpdate::None => (),
        }
        self.post_update = Default::default();
    }
    fn clear_app_state(&mut self) {
        self.app_state = Default::default()
    }
}
