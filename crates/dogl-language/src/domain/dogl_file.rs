//! DoglFile: file-level entity. One .dogl file → one DoglFile (parser result).
//! Carries schema version for AST/JSON compatibility (task 2.3).

use crate::domain::collab::Collab;
use crate::domain::value_objects::SchemaVersion;

/// Contents of one `.dogl` file. Parser returns `Result<DoglFile, ParseError>`.
#[derive(Debug, Clone, PartialEq)]
pub struct DoglFile {
    /// One or more collaborations (processes) in the file.
    pub collabs: Vec<Collab>,
    /// Schema version for AST/JSON compatibility. Set to current by default in `new()`.
    pub schema_version: Option<SchemaVersion>,
}

impl DoglFile {
    /// Creates a DoglFile with the current schema version so AST/JSON always has a version for compatibility.
    pub fn new(collabs: Vec<Collab>) -> Self {
        Self {
            collabs,
            schema_version: Some(SchemaVersion::current()),
        }
    }

    pub fn with_schema_version(mut self, v: SchemaVersion) -> Self {
        self.schema_version = Some(v);
        self
    }

    /// Returns the schema version, or the current one if not set (e.g. when loading legacy data).
    pub fn schema_version_or_current(&self) -> SchemaVersion {
        self.schema_version
            .clone()
            .unwrap_or_else(SchemaVersion::current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::SchemaVersion;

    #[test]
    fn new_sets_current_schema_version() {
        let df = DoglFile::new(vec![]);
        assert!(df.collabs.is_empty());
        assert_eq!(
            df.schema_version.as_ref().map(|v| v.as_str()),
            Some(SchemaVersion::CURRENT)
        );
    }

    #[test]
    fn schema_version_or_current() {
        let df = DoglFile::new(vec![]);
        assert_eq!(df.schema_version_or_current().as_str(), SchemaVersion::CURRENT);
    }

    #[test]
    fn with_schema_version() {
        let c = Collab::new(1, "C1");
        let df = DoglFile::new(vec![c]).with_schema_version(SchemaVersion::current());
        assert_eq!(df.collabs.len(), 1);
        assert!(df.schema_version.is_some());
    }
}
