#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod controller;
pub mod validation;

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

    let controller = Rc::new(
        RefCell::new(AppController::new(&ui, shared_config))
    );

    // Initial call to update the trigger list
    controller.borrow_mut().update_trigger_list();

    let c = controller.clone();
    ui.on_filter_changed(move || {
        c.borrow_mut().update_trigger_list();
    });

    let c = controller.clone();
    ui.on_selected_trigger_changed(move || {
        c.borrow().handle_selection_change();
    });

    let c = controller.clone();
    ui.on_new_trigger_changed(move |new_text| {
        c.borrow().handle_trigger_input_change(&new_text);
    });

    let c = controller.clone();
    ui.on_add_rule_clicked(move || {
        c.borrow_mut().handle_add_rule();
    });


    let c = controller.clone();
    ui.on_save_clicked(move || {
        c.borrow_mut().handle_save_rule();
    });

    let c = controller.clone();
    ui.on_delete_selected_rule(move || {
        c.borrow_mut().handle_delete_rule();
    });

    ui.run()
}