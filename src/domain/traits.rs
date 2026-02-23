//! Element traits: Identifiable, HasExpressions, ElementVariant (review1 §2.1).
//! Event, Task, Gateway, Artifact implement ElementVariant; Element delegates to the variant.

use crate::domain::value_objects::{ElementId, Expression, Uid};

/// Entity with stable uid and notation id. All in-code references use uid.
pub trait Identifiable {
    fn uid(&self) -> Uid;
    fn id(&self) -> &ElementId;
}

/// Entity with an ordered chain of expressions (e.g. @do, @dmn, @call).
pub trait HasExpressions {
    fn expressions(&self) -> &[Expression];
}

/// Element variant: Identifiable + HasExpressions. Implemented by Event, Task, Gateway, Artifact.
pub trait ElementVariant: Identifiable + HasExpressions {}
