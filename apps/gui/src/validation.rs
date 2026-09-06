pub enum TriggerValidation<'a> {
    Valid,
    Empty,
    ExactMatch,
    Substring { conflicting_trigger: &'a str },
    Superstring { conflicting_trigger: &'a str },
}

pub fn validate_trigger<'a>(candidate: &str, existing: &'a [&str], current_trigger: Option<&str>) -> TriggerValidation<'a> {
    let cleaned_trigger = candidate.trim();

    if cleaned_trigger.is_empty() {
        return TriggerValidation::Empty;
    }

    for existing_trigger in existing {
        // Skip comparing against the trigger currently being edited
        if let Some(current_trigger) = current_trigger && existing_trigger == &current_trigger {
            continue;
        }
        if existing_trigger == &cleaned_trigger {
            return TriggerValidation::ExactMatch;
        }
        if existing_trigger.contains(cleaned_trigger) {
            return TriggerValidation::Substring { conflicting_trigger: existing_trigger };
        }
        if cleaned_trigger.contains(existing_trigger) {
            return TriggerValidation::Superstring { conflicting_trigger: existing_trigger };
        }
    }

    TriggerValidation::Valid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_unique_trigger() {
        let existing = vec![";email", ";phone"];
        let result = validate_trigger(";addr", &existing, None);
        assert!(matches!(result, TriggerValidation::Valid));
    }

    #[test]
    fn test_empty_or_whitespace_candidate() {
        let existing = vec![";email"];

        let result_empty = validate_trigger("", &existing, None);
        assert!(matches!(result_empty, TriggerValidation::Empty));

        let result_spaces = validate_trigger("   ", &existing, None);
        assert!(matches!(result_spaces, TriggerValidation::Empty));
    }

    #[test]
    fn test_exact_duplicate() {
        let existing = vec![";email", ";phone"];
        let result = validate_trigger(";email", &existing, None);

        assert!(matches!(result, TriggerValidation::ExactMatch));
    }

    #[test]
    fn test_candidate_is_substring_of_existing() {
        // Candidate ";mail" is inside existing ";mailto"
        let existing = vec![";mailto", ";phone"];
        let result = validate_trigger(";mail", &existing, None);

        match result {
            TriggerValidation::Substring { conflicting_trigger } => {
                assert_eq!(conflicting_trigger, ";mailto");
            }
            _ => panic!("Expected Substring variant"),
        }
    }

    #[test]
    fn test_candidate_is_superstring_of_existing() {
        // Candidate ";mailto" contains existing ";mail"
        let existing = vec![";mail", ";phone"];
        let result = validate_trigger(";mailto", &existing, None);

        match result {
            TriggerValidation::Superstring { conflicting_trigger } => {
                assert_eq!(conflicting_trigger, ";mail");
            }
            _ => panic!("Expected Superstring variant"),
        }
    }

    #[test]
    fn test_self_editing_should_ignore_current_trigger() {
        // When editing existing trigger ";email", typing ";email" back should be valid
        let existing = vec![";email", ";phone"];
        let result = validate_trigger(";email", &existing, Some(";email"));

        assert!(matches!(result, TriggerValidation::Valid));
    }

    #[test]
    fn test_self_editing_still_catches_other_conflicts() {
        // When editing ";email", changing it to ";phone" should still conflict with existing ";phone"
        let existing = vec![";email", ";phone"];
        let result = validate_trigger(";phone", &existing, Some(";email"));

        assert!(matches!(result, TriggerValidation::ExactMatch));
    }

    #[test]
    fn test_whitespace_trimmed_candidate_and_existing() {
        let existing = vec![";email"];
        let result = validate_trigger(" ;email ", &existing, None);

        assert!(matches!(result, TriggerValidation::ExactMatch));
    }
}