//! DMN decision table: uid, id, and list of rules (condition => target_uid); one default rule.
//! Invariant: at most one default rule (review1 §2.3).

use crate::domain::error::DomainError;
use crate::domain::value_objects::{ElementId, Uid};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DmnRule {
    /// Optional condition expression; None or empty for default (catch-all) rule.
    pub condition: Option<String>,
    /// Target element uid (gateway routes to this element when condition matches).
    pub target_uid: Uid,
    /// Target id for notation/display and id↔uid mapping at boundary.
    pub target: ElementId,
    /// True for the default (=>d) rule.
    pub is_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dmn {
    pub uid: Uid,
    pub id: String,
    pub rules: Vec<DmnRule>,
}

impl Dmn {
    pub fn new(uid: Uid, id: impl Into<String>) -> Self {
        Self {
            uid,
            id: id.into(),
            rules: Vec::new(),
        }
    }

    /// Adds a rule. Returns `Err(DmnMultipleDefaults)` if this rule is default and a default already exists.
    pub fn add_rule(
        &mut self,
        condition: Option<String>,
        target_uid: Uid,
        target: impl Into<ElementId>,
        is_default: bool,
    ) -> Result<(), DomainError> {
        if is_default && self.rules.iter().any(|r| r.is_default) {
            return Err(DomainError::DmnMultipleDefaults);
        }
        self.rules.push(DmnRule {
            condition,
            target_uid,
            target: target.into(),
            is_default,
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_rule_one_default_ok() {
        let mut d = Dmn::new(1, "Dec");
        assert!(d.add_rule(Some("a".into()), 10, "A", false).is_ok());
        assert!(d.add_rule(None, 20, "B", true).is_ok());
        assert_eq!(d.rules.len(), 2);
    }

    #[test]
    fn add_rule_second_default_rejects() {
        let mut d = Dmn::new(1, "Dec");
        assert!(d.add_rule(None, 20, "B", true).is_ok());
        assert!(matches!(
            d.add_rule(None, 30, "C", true),
            Err(DomainError::DmnMultipleDefaults)
        ));
        assert_eq!(d.rules.len(), 1);
    }
}
