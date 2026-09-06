use std::cell::RefCell;
use std::{fs, io};
use std::rc::Rc;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel, Weak};
use rule_codec::{serialize_rules};
use rule_codec::models::{RulesConfig, ExpansionRule, Expansion};
use crate::AppWindow;
use crate::validation::{validate_trigger, TriggerValidation};

pub struct AppController {
    ui_weak: Weak<AppWindow>,
    config: Rc<RefCell<RulesConfig>>,
    filtered: Vec<String>,
}

impl AppController {
    pub fn new(
        ui: &AppWindow,
        config: Rc<RefCell<RulesConfig>>,
    ) -> Self {
        Self {
            ui_weak: ui.as_weak(),
            config,
            filtered: Vec::new(),
        }
    }

    fn clear_ui_selection(&self) {
        let Some(ui) = self.ui_weak.upgrade() else { return; };

        ui.set_has_selection(false);
        ui.set_new_trigger("".into());
        ui.set_new_expansion("".into());
        ui.set_selected_trigger_index(-1);
        ui.set_validation_error("".into());
    }

    fn fetch_selected_expansion(&self) -> Option<Expansion> {
        let Some(trigger) = self.get_selected_trigger() else { return None; };

        self.config
            .borrow()
            .rules
            .iter()
            .find(|rule| rule.trigger == trigger)
            .map(|rule| Expansion(rule.expansion.clone()))
    }

    fn get_selected_trigger(&self) -> Option<String> {
        let Some(ui) = self.ui_weak.upgrade() else { return None; };

        let selected_idx = ui.get_selected_trigger_index() as isize;

        if selected_idx == -1 {
            return None;
        }

        Some(self.filtered[selected_idx as usize].clone())
    }

    pub fn update_trigger_list(&mut self) {
        let Some(ui) = self.ui_weak.upgrade() else { return; };
        let query = ui.get_filter().to_string().trim().to_lowercase();

        self.clear_ui_selection();

        self.filtered = self.config
            .borrow()
            .rules
            .iter()
            .filter(|rule| {
                if query.is_empty() {
                    true
                } else {
                    rule.trigger.to_lowercase().contains(&query)
                }
            })
            .map(|rule| rule.trigger.clone())
            .collect::<Vec<_>>();

        let triggers = self.filtered
            .iter()
            .map(SharedString::from)
            .collect::<Vec<_>>();
        ui.set_current_triggers(ModelRc::from(Rc::new(VecModel::from(triggers))));
    }

    pub fn handle_selection_change(&self) {
        let Some(ui) = self.ui_weak.upgrade() else { return; };
        ui.set_validation_error("".into());

        let Some(selected_trigger) = self.get_selected_trigger() else {
            self.clear_ui_selection();
            return;
        };

        let Some(expansion) = self.fetch_selected_expansion() else {
            self.clear_ui_selection();
            return;
        };

        let expansion_str = expansion.to_string();
        ui.set_has_selection(true);
        ui.set_new_trigger(SharedString::from(&selected_trigger).into());
        ui.set_new_expansion(expansion_str.into());
    }

    pub fn handle_trigger_input_change(&self, trigger: &str) {
        let Some(ui) = self.ui_weak.upgrade() else { return; };
        let borrowed_config = self.config.borrow();
        let cleaned = trigger.trim();

        if cleaned.is_empty() {
            ui.set_validation_error("".into());
            return;
        }

        let selected_trigger = self.get_selected_trigger();
        let current_trigger_ref = selected_trigger.as_deref();

        let existing_triggers: Vec<&str> = borrowed_config.rules
            .iter()
            .map(|rule| rule.trigger.as_str())
            .collect();

        let validation = validate_trigger(
            trigger,
            &existing_triggers,
            current_trigger_ref
        );

        let err: SharedString = match validation {
            TriggerValidation::Empty => "Trigger cannot be empty".into(),
            TriggerValidation::ExactMatch => "Trigger already exists".into(),
            TriggerValidation::Substring { conflicting_trigger } =>
                format!("{} is substring of existing {}", trigger, conflicting_trigger).into(),
            TriggerValidation::Superstring { conflicting_trigger } =>
                format!("{} is superstring of existing {}", trigger, conflicting_trigger).into(),
            TriggerValidation::Valid => "".into(),
        };
        ui.set_validation_error(err);
    }

    pub fn handle_add_rule(&mut self) {
        let Some(ui) = self.ui_weak.upgrade() else { return; };


        let raw_trigger = ui.get_new_trigger().to_string();
        let raw_expansion = ui.get_new_expansion().to_string();

        let Some(new_rule) = self.get_parsed_rule(&raw_trigger, raw_expansion) else { return; };

        // --- Scoped mutable borrow block --- //
        {
            let mut borrowed_config = self.config.borrow_mut();
            borrowed_config.rules.push(new_rule);
            let _ = self.save_to_file(&borrowed_config);
        }

        // --- Safe to borrow immutable here --- //
        self.update_trigger_list();

        ui.set_validation_error("".into());
        ui.set_new_trigger("".into());
        ui.set_new_expansion("".into());
    }

    pub fn handle_save_rule(&mut self) {
        let Some(ui) = self.ui_weak.upgrade() else { return; };
        let Some(trigger) = self.get_selected_trigger() else { return; };

        let raw_trigger = ui.get_new_trigger().to_string();
        let raw_expansion = ui.get_new_expansion().to_string();
        let Some(new_rule) = self.get_parsed_rule(&raw_trigger, raw_expansion) else { return; };

        {
            let mut borrowed_config = self.config.borrow_mut();
            // Need to find the idx
            if let Some(existing_rule) = borrowed_config
                .rules
                .iter_mut()
                .find(|rule| rule.trigger == trigger) {
                    existing_rule.trigger = new_rule.trigger.clone();
                    existing_rule.expansion = new_rule.expansion;
                    let _ = self.save_to_file(&borrowed_config);
                }
        }

        self.update_trigger_list();
    }

    // TODO: Needed to be refactor below
    pub fn handle_delete_rule(&mut self) {
        let Some(selected_trigger) = self.get_selected_trigger() else { return; };

        {
            let mut borrowed_config = self.config.borrow_mut();
            borrowed_config.rules.retain(|rule| rule.trigger != selected_trigger);
            let _ = self.save_to_file(&borrowed_config);
        }
        self.update_trigger_list();
    }

    fn get_parsed_rule(&self, raw_trig: &str, raw_expansion: String) -> Option<ExpansionRule> {
        if raw_trig.trim().is_empty() || raw_expansion.trim().is_empty() {
            return None;
        }

        let snippets = Expansion::from_str(&raw_expansion);
        if snippets.is_err() {
            return None;
        }
        let snippets = snippets.unwrap();

        Some(ExpansionRule {
            trigger: raw_trig.trim().into(),
            expansion: snippets.0,
        })
    }

    fn save_to_file(&self, config: &RulesConfig) -> Result<(), io::Error> {
        let target_path = workspace_config::get_rules_file()?;
        let config_dir = workspace_config::get_config_dir()?;

        // --- 1. Create the timpestamped backup
        if fs::metadata(&target_path).is_ok() {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_secs())
                .unwrap_or(0);
            let backup_filename = format!("rules_{}.rtex.bak", timestamp);
            let backup_path = config_dir.join(&backup_filename);
            fs::copy(&target_path, &backup_path)?;
        }

        // --- 2. Write to file
        let bytes = serialize_rules(&config.rules);
        fs::write(&target_path, bytes)?;

        Ok(())
    }
}