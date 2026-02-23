//! Elements: Event, Task, Gateway, Artifact with optional codes and expressions.
//! All implement ElementVariant; Element delegates to the variant (review1 §2.1).

use crate::domain::traits::{ElementVariant, HasExpressions, Identifiable};
use crate::domain::value_objects::{ElementId, Expression, Uid};

/// Element in the process: event, task, gateway, or artifact.
#[derive(Debug, Clone, PartialEq)]
pub enum Element {
    Event(Event),
    Task(Task),
    Gateway(Gateway),
    Artifact(Artifact),
}

impl Identifiable for Element {
    fn uid(&self) -> Uid {
        match self {
            Element::Event(e) => e.uid(),
            Element::Task(e) => e.uid(),
            Element::Gateway(e) => e.uid(),
            Element::Artifact(e) => e.uid(),
        }
    }

    fn id(&self) -> &ElementId {
        match self {
            Element::Event(e) => e.id(),
            Element::Task(e) => e.id(),
            Element::Gateway(e) => e.id(),
            Element::Artifact(e) => e.id(),
        }
    }
}

impl HasExpressions for Element {
    fn expressions(&self) -> &[Expression] {
        match self {
            Element::Event(e) => e.expressions(),
            Element::Task(e) => e.expressions(),
            Element::Gateway(e) => e.expressions(),
            Element::Artifact(e) => e.expressions(),
        }
    }
}

impl ElementVariant for Element {}

impl Element {
    /// Display name (BPMN/UI). When omitted in .dogl, generated from id (PascalCase → sentence).
    pub fn name(&self) -> &str {
        match self {
            Element::Event(e) => &e.name,
            Element::Task(e) => &e.name,
            Element::Gateway(e) => &e.name,
            Element::Artifact(e) => &e.name,
        }
    }
}

/// Event code: start, intermediate, end (notation: (s), (i), (e)).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventCode {
    Start,
    Intermediate,
    End,
    /// Inferred from connectivity when no code given.
    Inferred,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub uid: Uid,
    pub id: ElementId,
    /// Display name; when omitted in .dogl, use `name_from_id(&id)` (review1 §2.5).
    pub name: String,
    pub code: EventCode,
    pub expressions: Vec<Expression>,
}

impl Identifiable for Event {
    fn uid(&self) -> Uid {
        self.uid
    }
    fn id(&self) -> &ElementId {
        &self.id
    }
}

impl HasExpressions for Event {
    fn expressions(&self) -> &[Expression] {
        &self.expressions
    }
}

impl ElementVariant for Event {}

/// Task code (notation 02: [], [m], [u], [st], etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskCode {
    Generic,
    Manual,
    User,
    Service,
    Receive,
    Send,
    Script,
    BusinessRule,
    SendMessage,
    ReceiveMessage,
    CallActivity,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub uid: Uid,
    pub id: ElementId,
    /// Display name; when omitted in .dogl, use `name_from_id(&id)` (review1 §2.5).
    pub name: String,
    pub code: TaskCode,
    pub expressions: Vec<Expression>,
}

impl Identifiable for Task {
    fn uid(&self) -> Uid {
        self.uid
    }
    fn id(&self) -> &ElementId {
        &self.id
    }
}

impl HasExpressions for Task {
    fn expressions(&self) -> &[Expression] {
        &self.expressions
    }
}

impl ElementVariant for Task {}

/// Gateway code (notation 02: <>, <x>, <p>, <i>, <c>, <eb>).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatewayCode {
    /// Inclusive (OR), default <>.
    Inclusive,
    Exclusive,
    Parallel,
    Complex,
    EventBased,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Gateway {
    pub uid: Uid,
    pub id: ElementId,
    /// Display name; when omitted in .dogl, use `name_from_id(&id)` (review1 §2.5).
    pub name: String,
    pub code: GatewayCode,
    /// Optional reference to standalone DMN block (e.g. @dmn: "DecisionName").
    pub dmn_ref: Option<String>,
    pub expressions: Vec<Expression>,
}

impl Identifiable for Gateway {
    fn uid(&self) -> Uid {
        self.uid
    }
    fn id(&self) -> &ElementId {
        &self.id
    }
}

impl HasExpressions for Gateway {
    fn expressions(&self) -> &[Expression] {
        &self.expressions
    }
}

impl ElementVariant for Gateway {}

/// Artifact code (notation 02: {}, {d}, {db}, {f}, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactCode {
    Default,
    Data,
    Database,
    File,
    Report,
    Document,
    Message,
    Email,
    Collection,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Artifact {
    pub uid: Uid,
    pub id: ElementId,
    /// Display name; when omitted in .dogl, use `name_from_id(&id)` (review1 §2.5).
    pub name: String,
    pub code: ArtifactCode,
    pub expressions: Vec<Expression>,
}

impl Identifiable for Artifact {
    fn uid(&self) -> Uid {
        self.uid
    }
    fn id(&self) -> &ElementId {
        &self.id
    }
}

impl HasExpressions for Artifact {
    fn expressions(&self) -> &[Expression] {
        &self.expressions
    }
}

impl ElementVariant for Artifact {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{name_from_id, Expression};

    #[test]
    fn element_delegates_uid_id_expressions_name() {
        let uid = 100u128;
        let id = "Start".to_string();
        let name = name_from_id("Start");
        let e = Element::Event(Event {
            uid,
            id: id.clone(),
            name: name.clone(),
            code: EventCode::Start,
            expressions: vec![],
        });
        assert_eq!(e.uid(), uid);
        assert_eq!(e.id(), &id);
        assert_eq!(e.expressions(), &[] as &[_]);
        assert_eq!(e.name(), name.as_str());
    }

    #[test]
    fn task_element_delegates() {
        let uid = 200u128;
        let el = Element::Task(Task {
            uid,
            id: "DoWork".to_string(),
            name: name_from_id("DoWork"),
            code: TaskCode::Generic,
            expressions: vec![Expression::new("do", "run")],
        });
        assert_eq!(el.uid(), uid);
        assert_eq!(el.id(), "DoWork");
        assert_eq!(el.expressions().len(), 1);
        assert_eq!(el.name(), "Do work");
    }
}
