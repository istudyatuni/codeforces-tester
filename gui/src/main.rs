use gtk::prelude::*;

use gtk::{glib, Application, ApplicationWindow, Builder};

fn main() -> glib::ExitCode {
    let application = Application::new(
        Some("com.github.istudyatuni.codeforces-tester"),
        Default::default(),
    );
    application.connect_activate(build_ui);
    application.run()
}

pub fn build_ui(application: &Application) {
    let ui_src = include_str!("../resources/window.ui");
    let builder = Builder::new();
    builder
        .add_from_string(ui_src)
        .expect("Couldn't add from string");

    let window: ApplicationWindow = builder.object("window").expect("Couldn't get window");
    window.set_application(Some(application));
    // let text_view: TextView = builder.object("text_view").expect("Couldn't get text_view");

    window.present();
}
