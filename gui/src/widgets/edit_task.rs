use eframe::egui::{Button, Response, Ui, Widget};

use lib::TaskID;

#[derive(Debug, Default)]
pub(crate) struct EditTaskState {
    pub(crate) id: String,
    pub(crate) name: String,
    orig_id: String,
    orig_name: String,
}

impl EditTaskState {
    pub(crate) fn new<S: Into<String>>(id: &TaskID, name: S) -> Self {
        let name = name.into();
        Self {
            id: id.clone(),
            name: name.clone(),
            orig_id: id.clone(),
            orig_name: name,
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
    let as_orig_state = state.id == state.orig_id && state.name == state.orig_name;
    ui.add_enabled(!as_orig_state, Button::new("Submit"))
}

pub(crate) fn edit_task(state: &mut EditTaskState) -> impl Widget + '_ {
    move |ui: &mut Ui| edit_task_ui(ui, state)
}
