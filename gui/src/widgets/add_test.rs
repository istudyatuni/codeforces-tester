use eframe::egui::{Response, Ui, Widget};

#[derive(Debug, Default, Clone)]
pub(crate) struct AddTestState {
    pub(crate) input: String,
    pub(crate) expected: String,
}

fn add_test_ui(ui: &mut Ui, state: &mut AddTestState, task_id: String) -> Response {
    ui.heading(format!("Add test for task {}:", task_id.to_uppercase()));
    let id_label = ui.label("Input:");
    ui.text_edit_multiline(&mut state.input)
        .labelled_by(id_label.id);
    let id_label = ui.label("Expected:");
    ui.text_edit_multiline(&mut state.expected)
        .labelled_by(id_label.id);
    ui.button("Submit")
}

pub(crate) fn add_test(state: &mut AddTestState, task_id: String) -> impl Widget + '_ {
    move |ui: &mut Ui| add_test_ui(ui, state, task_id)
}
