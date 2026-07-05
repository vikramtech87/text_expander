mod controller;

use parser::{Config};
use slint::{ModelRc, SharedString, VecModel};
use std::cell::{RefCell};
use std::{fs};
use std::rc::Rc;
use crate::controller::AppController;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    let config_path = workspace_config::get_rules_file()
        .unwrap_or("./rules.toml".into());

    let config = fs::read_to_string(config_path)
        .ok()
        .and_then(|content| toml::from_str::<Config>(&content).ok())
        .unwrap_or_else(|| Config { rules: Vec::new() });
    let shared_config = Rc::new(RefCell::new(config));

    let trigger_strings: Vec<SharedString> = shared_config
        .borrow()
        .rules
        .iter()
        .map(|rule| SharedString::from(&rule.trigger))
        .collect();

    let triggers_model = Rc::new(VecModel::from(trigger_strings));
    ui.set_current_triggers(ModelRc::from(triggers_model.clone()));

    let controller = Rc::new(
        AppController::new(&ui, shared_config, triggers_model.clone())
    );

    let c = controller.clone();
    ui.on_selected_trigger_changed(move |idx| {
        c.handle_selection_change(idx);
    });

    let c = controller.clone();
    ui.on_new_trigger_changed(move |new_text| {
        c.handle_trigger_input_change(&new_text);
    });

    let c = controller.clone();
    ui.on_add_rule_clicked(move || {
       c.handle_add_rule();
    });


    let c = controller.clone();
    ui.on_save_clicked(move || {
        c.handle_save_rule()
    });

    ui.run()
}


