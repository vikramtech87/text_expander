use std::cell::{Cell, RefCell};
use std::{fs, io};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};
use slint::{ComponentHandle, Model, SharedString, VecModel, Weak};
use parser::{Config, ExpansionRule};
use parser::utils::{rule_to_string, rule_to_user_friendly, tokenize_expansion};
use crate::AppWindow;

pub struct AppController {
    ui_weak: Weak<AppWindow>,
    config: Rc<RefCell<Config>>,
    current_ix: Rc<Cell<isize>>,
    triggers: Rc<VecModel<SharedString>>,
}

impl AppController {
    pub fn new(
        ui: &AppWindow,
        config: Rc<RefCell<Config>>,
        triggers: Rc<VecModel<SharedString>>
    ) -> Self {
        Self {
            ui_weak: ui.as_weak(),
            config,
            current_ix: Rc::new(Cell::new(-1)),
            triggers,
        }
    }

    pub fn handle_selection_change(&self, idx: i32) {
        let Some(ui) = self.ui_weak.upgrade() else { return; };
        self.current_ix.set(idx as isize);

        ui.set_validation_error("".into());

        if idx == -1 {
            ui.set_has_selection(false);
            ui.set_new_trigger("".into());
            ui.set_new_expansion("".into());
            return;
        }

        // idx != -1
        let borrowed_config = self.config.borrow();
        let Some(rule) = borrowed_config.rules.get(idx as usize) else { return; };
        let expansion = rule_to_user_friendly(rule);
        ui.set_has_selection(true);
        ui.set_new_trigger(SharedString::from(&rule.trigger).into());
        ui.set_new_expansion(expansion.into());
    }

    pub fn handle_trigger_input_change(&self, trigger: &str) {
        let Some(ui) = self.ui_weak.upgrade() else { return; };
        let borrowed_config = self.config.borrow();
        let cleaned = trigger.trim();

        if cleaned.is_empty() {
            ui.set_validation_error("".into());
            return;
        }

        let selected_ix = self.current_ix.get();
        let selected_trigger = match borrowed_config.rules.get(selected_ix as usize) {
            Some(rule) => &rule.trigger,
            None => "",
        };
        let is_duplicate = borrowed_config.rules
            .iter()
            .any(|rule| rule.trigger == cleaned && rule.trigger != selected_trigger);

        let err: SharedString = if is_duplicate {
            format!("⚠️ Trigger '{}' already exists!", cleaned).into()
        } else {
            "".into()
        };
        ui.set_validation_error(err);
    }

    pub fn handle_add_rule(&self) {
        let Some(ui) = self.ui_weak.upgrade() else { return; };
        let mut borrowed_config = self.config.borrow_mut();

        let raw_trigger = ui.get_new_trigger().to_string();
        let raw_expansion = ui.get_new_expansion().to_string();

        let Some(new_rule) = self.get_parsed_rule(&raw_trigger, raw_expansion) else { return; };

        borrowed_config.rules.push(new_rule);
        self.triggers.push(SharedString::from(&raw_trigger));

        let _ = self.save_to_file(&borrowed_config);

        ui.set_validation_error("".into());
        ui.set_new_trigger("".into());
        ui.set_new_expansion("".into());
    }

    pub fn handle_save_rule(&self) {
        let Some(ui) = self.ui_weak.upgrade() else { return; };
        let mut borrowed_config = self.config.borrow_mut();
        let curr_ix = self.current_ix.get();

        let raw_trigger = ui.get_new_trigger().to_string();
        let raw_expansion = ui.get_new_expansion().to_string();

        let Some(new_rule) = self.get_parsed_rule(&raw_trigger, raw_expansion) else { return; };
        if let Some(existing_rule) = borrowed_config.rules.get_mut(curr_ix as usize) {
            existing_rule.trigger = new_rule.trigger.clone();
            existing_rule.expansion = new_rule.expansion;
            self.triggers.set_row_data(
                curr_ix as usize,
                SharedString::from(&new_rule.trigger).into(),
            );

            let _ = self.save_to_file(&borrowed_config);
        }
    }
    fn get_parsed_rule(&self, raw_trig: &str, raw_expansion: String) -> Option<ExpansionRule> {
        if raw_trig.trim().is_empty() || raw_expansion.trim().is_empty() {
            return None;
        }

        let snippets = tokenize_expansion(&raw_expansion);
        if snippets.is_err() {
            return None;
        }
        let snippets = snippets.unwrap();

        Some(ExpansionRule {
            trigger: raw_trig.trim().into(),
            expansion: snippets,
        })
    }

    fn save_to_file(&self, config: &Config) -> Result<(), io::Error> {
        let target_path = workspace_config::get_rules_file()?;
        let config_dir = workspace_config::get_config_dir()?;

        // --- 1. Create the timpestamped backup
        if fs::metadata(&target_path).is_ok() {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_secs())
                .unwrap_or(0);
            let backup_filename = format!("rules_{}.toml.bak", timestamp);
            let backup_path = config_dir.join(&backup_filename);
            fs::copy(&target_path, &backup_path)?;
        }

        // --- 2. Write to file
        let toml_string = config
            .rules
            .iter()
            .map(rule_to_string)
            .collect::<Vec<String>>()
            .join("\n\n");

        fs::write(&target_path, toml_string)?;

        Ok(())
    }
}