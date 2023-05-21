use eframe::egui::{Button, Response, Ui, Widget};

use lib::TaskID;

#[derive(Debug, Default)]
pub(crate) struct EditTaskState {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) is_task_exists: bool,
}

impl EditTaskState {
    pub(crate) fn new<S: Into<String>>(id: &TaskID, name: S) -> Self {
        Self {
            id: id.clone(),
            name: name.into(),
            is_task_exists: true,
        }
    }
}

fn edit_task_ui(ui: &mut Ui, state: &mut EditTaskState) -> Response {
    ui.heading("Edit task:");
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
    ui.add_enabled(!state.is_task_exists, Button::new("Submit"))
        .on_disabled_hover_text("Tasks with this ID already exists")
}

pub(crate) fn edit_task(state: &mut EditTaskState) -> impl Widget + '_ {
    move |ui: &mut Ui| edit_task_ui(ui, state)
}
