use parser::utils::{rule_to_string, rule_to_user_friendly, tokenize_expansion};
use parser::{Config, ExpansionRule};
use slint::{Model, ModelRc, SharedString, VecModel};
use std::cell::{Cell, RefCell};
use std::{fs, io};
use std::ops::Deref;
use std::rc::Rc;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    let config_path = "rules.toml";
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

    let active_idx: Rc<Cell<isize>> = Rc::new(Cell::new(-1));

    let ui_handle = ui.as_weak();
    let triggers_for_validation = triggers_model.clone();
    let active_idx_for_validation = active_idx.clone();
    ui.on_new_trigger_changed(move |trigger| {
        let ui = ui_handle.unwrap();
        let cleaned_input = trigger.trim();

        if cleaned_input.is_empty() {
            ui.set_validation_error("".into());
            return;
        }

        let current_selected = active_idx_for_validation.get();

        let is_duplicate = (0..triggers_for_validation.row_count()).any(|row| {
            // Skip checking the row we are actively editing
            if current_selected >= 0 && row == current_selected as usize {
                return false;
            }

            let t = triggers_for_validation.row_data(row);
            if t.is_none() {
                return false;
            }
            let t = t.unwrap();
            t.as_str() == cleaned_input
        });

        let err: SharedString = if is_duplicate {
            format!("⚠️ Trigger '{}' already exists!", cleaned_input).into()
        } else {
            "".into()
        };
        ui.set_validation_error(err);
    });

    let ui_handle_for_selection = ui.as_weak();
    let config_for_selection = shared_config.clone();
    let active_idx_for_selection = active_idx.clone();
    ui.on_selected_trigger_changed(move |idx| {
        let ui = ui_handle_for_selection.unwrap();
        active_idx_for_selection.set(idx as isize);

        if idx == -1 {
            ui.set_has_selection(false);
            ui.set_new_trigger("".into());
            ui.set_new_expansion("".into());
            ui.set_validation_error("".into());
        } else {
            let borrowed_config = config_for_selection.borrow();
            let matched = borrowed_config.rules.get(idx as usize);
            if let Some(rule) = matched {
                ui.set_new_trigger(rule.trigger.clone().into());
                ui.set_new_expansion(rule_to_user_friendly(&rule).into());
                ui.set_validation_error("".into());
                ui.set_has_selection(true);
            }
        }
    });

    // -- Add Rule Action Callback
    let ui_handle_for_add = ui.as_weak();
    let config_for_add = shared_config.clone();
    let triggers_model_for_add = triggers_model.clone();

    ui.on_add_rule_clicked(move || {
        let ui = ui_handle_for_add.unwrap();

        let raw_trigger = ui.get_new_trigger().to_string();
        let raw_expansion = ui.get_new_expansion().to_string();

        if raw_trigger.trim().is_empty() || raw_expansion.trim().is_empty() {
            return;
        }

        let snippets = tokenize_expansion(&raw_expansion);
        if snippets.is_err() {
            return;
        }
        let new_rule = ExpansionRule {
            trigger: raw_trigger.clone(),
            expansion: snippets.unwrap(),
        };

        config_for_add.borrow_mut().rules.push(new_rule);
        triggers_model_for_add.push(SharedString::from(&raw_trigger));

        let borrowed_config = config_for_add.borrow();
        // let toml_string = borrowed_config
        //     .rules
        //     .iter()
        //     .map(rule_to_string)
        //     .collect::<Vec<String>>()
        //     .join("\n\n");
        //
        // let _ = fs::write("rules.toml", toml_string);
        let _ = save_to_file(borrowed_config.deref());

        ui.set_new_trigger("".into());
        ui.set_new_expansion("".into());
    });

    let ui_handle_for_save = ui.as_weak();
    let config_for_save = shared_config.clone();
    let active_idx_for_save = active_idx.clone();
    let triggers_model_for_save = triggers_model.clone();

    ui.on_save_clicked(move || {
        let ui = ui_handle_for_save.unwrap();
        let current_idx = active_idx_for_save.get();

        if current_idx < 0 {
            return;
        }

        let raw_trigger = ui.get_new_trigger().to_string();
        let raw_expansion = ui.get_new_expansion().to_string();

        if raw_trigger.trim().is_empty() || raw_expansion.trim().is_empty() {
            return;
        }

        let snippets = tokenize_expansion(&raw_expansion);
        if snippets.is_err() {
            return;
        }
        let snippets = snippets.unwrap();
        let mut borrowed_config = config_for_save.borrow_mut();
        if let Some(existing_rule) = borrowed_config.rules.get_mut(current_idx as usize) {
            existing_rule.trigger = raw_trigger.clone();
            existing_rule.expansion = snippets;
            triggers_model_for_save
                .set_row_data(current_idx as usize, SharedString::from(&raw_trigger));

            // let toml_string = borrowed_config
            //     .rules
            //     .iter()
            //     .map(rule_to_string)
            //     .collect::<Vec<String>>()
            //     .join("\n\n");

            let _ = save_to_file(borrowed_config.deref());
        }
    });

    ui.run()
}

fn save_to_file(config: &Config) -> Result<(), io::Error> {
    let toml_string = config
        .rules
        .iter()
        .map(rule_to_string)
        .collect::<Vec<String>>()
        .join("\n\n");

    let target_path = "rules.toml";
    let tmp_path = &format!("{}.tmp", target_path);

    fs::write(tmp_path, toml_string)?;

    if let Err(e) = fs::rename(tmp_path, target_path) {
        let _ = fs::remove_file(tmp_path);
        return Err(e);
    }

    Ok(())
}
