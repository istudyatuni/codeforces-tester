use eframe::egui::{Response, Ui, Widget};

use lib::{TaskID, Test};

#[derive(Debug, Default)]
pub(crate) struct EditTestsState {
    pub(crate) id: String,
    pub(crate) edited_test: Option<(usize, Test)>,
    // TODO: move to trait as .response()?
    pub(crate) response: EditTestsResponse,
    pub(crate) tests: Vec<Test>,
}

#[derive(Debug, Default)]
pub(crate) enum EditTestsResponse {
    SaveTest((usize, Test)),
    Cancel,
    #[default]
    None,
}

impl EditTestsState {
    pub(crate) fn new(id: &TaskID, tests: &[Test]) -> Self {
        Self {
            id: id.clone(),
            tests: tests.into(),
            ..Default::default()
        }
    }
}

fn edit_tests_ui(ui: &mut Ui, state: &mut EditTestsState) -> Response {
    ui.heading(format!("Edit tests for task {}:", state.id));

    for (i, t) in (&state.tests).iter().enumerate() {
        ui.horizontal(|ui| {
            if ui.button("edit").clicked() {
                state.edited_test = Some((i, t.clone()));
            }
            ui.label(format!("test {}", i + 1));
        });
    }

    let mut cancel_editing = false;
    if let Some((i, t)) = &mut state.edited_test {
        ui.heading(format!("Edit test {}:", *i + 1));
        let label = ui.label("Input:");
        ui.code_editor(&mut t.input).labelled_by(label.id);
        let label = ui.label("Expected:");
        ui.code_editor(&mut t.expected)
            .labelled_by(label.id);
        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                state.response = EditTestsResponse::SaveTest((*i, t.clone()));
            }
            if ui.button("Cancel").clicked() {
                cancel_editing = true;
            }
        });
    } else {
        if ui.button("Cancel").clicked() {
            state.response = EditTestsResponse::Cancel;
        }
    }
    if cancel_editing {
        state.edited_test = None;
    }

    ui.label("")
}

pub(crate) fn edit_tests(state: &mut EditTestsState) -> impl Widget + '_ {
    move |ui: &mut Ui| edit_tests_ui(ui, state)
}
