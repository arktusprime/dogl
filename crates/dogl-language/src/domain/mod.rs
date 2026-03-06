//! Transitional semantic-domain boundary.
//!
//! The current semantic types remain sourced from the legacy domain files while
//! the crate skeleton converges on the target multi-layer architecture.

#[path = "../../../../src/domain/error.rs"]
mod error;
#[path = "../../../../src/domain/value_objects.rs"]
mod value_objects;
#[path = "../../../../src/domain/traits.rs"]
mod traits;
#[path = "../../../../src/domain/element.rs"]
mod element;
#[path = "../../../../src/domain/flow.rs"]
mod flow;
#[path = "../../../../src/domain/dmn.rs"]
mod dmn;
#[path = "../../../../src/domain/layout.rs"]
mod layout;
#[path = "../../../../src/domain/pool.rs"]
mod pool;
#[path = "../../../../src/domain/collab.rs"]
mod collab;
#[path = "../../../../src/domain/dogl_file.rs"]
mod dogl_file;

pub use collab::Collab;
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
