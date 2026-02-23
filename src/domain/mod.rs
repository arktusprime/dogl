//! **Domain** — core: types and value objects (DoglFile, Collab, Pool, Lane, Stage, Element, Flow, Dmn, Layout).
//! No dependencies on parser, JSON, BPMN. See rd/arch/architecture.md.

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

pub use value_objects::{
    name_from_id, Bounds, CollabId, ElementId, Expression, FlowType, LaneId, PoolId, Position,
    SchemaVersion, StageId, Uid,
};
pub use element::{
    Artifact, ArtifactCode, Element, Event, EventCode, Gateway, GatewayCode, Task, TaskCode,
};
pub use flow::Flow;
pub use dmn::{Dmn, DmnRule};
pub use layout::{Layout, LayoutGroupedByPool, PoolLayoutData};
pub use pool::{Lane, Pool, Quadrant, Stage};
pub use collab::Collab;
pub use dogl_file::DoglFile;
pub use error::DomainError;
pub use traits::{ElementVariant, HasExpressions, Identifiable};
