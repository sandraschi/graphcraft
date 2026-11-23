use thiserror::Error;

/// Typed domain errors for GraphCraft L1 core operations.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum GraphError {
    #[error("Node not found: '{0}'")]
    NodeNotFound(String),

    #[error("Edge not found: '{0}'")]
    EdgeNotFound(String),

    #[error("Duplicate node id: '{0}'")]
    DuplicateNode(String),

    #[error("Invalid input: {0}")]
    Validation(String),

    #[error("Serialization failed: {0}")]
    Serialization(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Feature not implemented yet: {0}")]
    NotImplemented(String),
}

impl From<rusqlite::Error> for GraphError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Database(err.to_string())
    }
}
