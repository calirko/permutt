mod app;
mod converter;
mod output_picker;
mod style;

use gtk::prelude::*;

fn main() {
    let application = app::build_app();
    application.connect_startup(|_| style::load());
    std::process::exit(application.run().value());
}
