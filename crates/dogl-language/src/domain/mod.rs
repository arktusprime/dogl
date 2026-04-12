//! Transitional semantic-domain boundary.
//!
//! The current semantic types remain sourced from the legacy domain files while
//! the crate skeleton converges on the target multi-layer architecture.

mod error;
mod value_objects;
mod traits;
mod element;
mod flow;
mod dmn;
mod layout;
mod pool;
mod collab;
mod dogl_file;

pub use collab::{layout_from_grouped, layout_to_grouped, Collab};
pub use dmn::{Dmn, DmnRule};
pub use dogl_file::DoglFile;
pub use element::{
    Artifact, ArtifactCode, Element, Event, EventCode, Gateway, GatewayCode, Task, TaskCode,
};
pub use error::DomainError;
pub use flow::Flow;
pub use layout::{Layout, LayoutGroupedByPool, PoolLayoutData};
pub use pool::{Lane, Pool, Quadrant, Stage};
pub use traits::{ElementVariant, HasExpressions, Identifiable};
pub use value_objects::{
    name_from_id, Bounds, CollabId, ElementId, Expression, FlowType, LaneId, PoolId, Position,
    SchemaVersion, StageId, Uid,
};
