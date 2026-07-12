#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod controller;

use slint::{ModelRc, SharedString, VecModel};
use std::cell::{RefCell};
use std::{fs};
use std::rc::Rc;
use rule_codec::deserialize_rules;
use rule_codec::models::RulesConfig;
use crate::controller::AppController;



slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    let rules = workspace_config::get_rules_file()
        .map_err(|e| format!("Error reading file {}", e))
        .and_then(|p| {
            fs::read(&p).map_err(|e| format!("Error reading file {}", e))
        })
        .and_then(|bytes| {
            deserialize_rules(&bytes)
                .map_err(|e| format!("Error deserializing file {}", e))
        }).unwrap_or(Vec::new());

    let config = RulesConfig { rules };
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
        c.handle_save_rule();
    });

    let c = controller.clone();
    ui.on_delete_selected_rule(move || {
        c.handle_delete_rule();
    });

    ui.run()
}