#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error("Parse error at line {line}: {message}")]
    Parse { line: usize, message: String },

    #[error("No feed loaded. Use 'feed <path>' first (line {line})")]
    NoFeedLoaded { line: usize },

    #[error("Destructive operation in batch requires --confirm flag (line {line})")]
    MissingConfirm { line: usize },

    #[error("line {line}: {message}")]
    FeedLoad { line: usize, message: String },

    #[error("line {line}: validation found errors")]
    ValidationFailed { line: usize },

    #[error("line {line}: {message}")]
    Command { line: usize, message: String },

    #[error("line {line}: write error: {source}")]
    Write {
        line: usize,
        #[source]
        source: headway_core::writer::WriteError,
    },

    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("{0}")]
    Io(#[from] std::io::Error),
}
