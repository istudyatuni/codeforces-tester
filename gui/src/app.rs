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
    AddTest(TaskID),
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
                            // ui.horizontal(|ui| {
                            ui.label(RichText::new(format_task_info(&t)).strong());
                            // });
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
                    let id_label = ui.label("ID: ");
                    ui.text_edit_singleline(&mut state.id)
                        .labelled_by(id_label.id);
                    let name_label = ui.label("Name: ");
                    ui.text_edit_singleline(&mut state.name)
                        .labelled_by(name_label.id);
                    if ui.button("Submit").clicked() {
                        if let Some(config_path) = &self.config_path {
                            match save_config(&mut self.config, config_path, state.clone()) {
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
                BottomState::AddTest(_task_id) => todo!(),
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

fn save_config(config: &mut Option<Config>, config_path: &PathBuf, task: AddTask) -> Result<()> {
    if let Some(ref mut config) = config {
        config.add_task(task.id.clone(), task.name.clone());
        config.save_config_to(config_path.into())?;
    }
    Ok(())
}
