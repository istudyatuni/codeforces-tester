use eframe::egui::{Response, Ui, Widget};

#[derive(Debug, Default, Clone)]
pub(crate) struct AddTaskState {
    pub(crate) id: String,
    pub(crate) name: String,
}

fn add_task_ui(ui: &mut Ui, state: &mut AddTaskState) -> Response {
    ui.heading("Add task:");
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
    ui.button("Submit")
}

pub(crate) fn add_task(state: &mut AddTaskState) -> impl Widget + '_ {
    move |ui: &mut Ui| add_task_ui(ui, state)
}
