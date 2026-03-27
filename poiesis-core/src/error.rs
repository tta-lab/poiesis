use thiserror::Error;

#[derive(Debug, Error)]
pub enum PoiesisError {
    // API errors
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("WordPress API: {message} (code: {code}, status: {status})")]
    WpApi {
        code: String,
        message: String,
        status: u16,
    },

    #[error("Authentication error: {0}")]
    Auth(String),

    // Content errors
    #[error("unknown section ID {id} — run 'poi detail <post_id> --tree' to see current IDs")]
    SectionNotFound { id: String },

    #[error("block parse failed: {0}")]
    BlockParseFailed(String),

    #[error("no content available")]
    NoContent,

    // Input validation
    #[error("invalid post ID '{0}' — must be a positive integer")]
    InvalidPostId(String),

    // Config errors
    #[error("config file not found. Run 'poi --help' for setup instructions.")]
    ConfigNotFound,

    #[error("config parse failed: {0}. Run 'poi --help' for setup instructions.")]
    ConfigParseFailed(String),

    #[error(
        "POIESIS_PASSWORD environment variable not set. Run 'poi --help' for setup instructions."
    )]
    MissingPassword,

    #[error(
        "POIESIS_PASSWORD environment variable is empty. Run 'poi --help' for setup instructions."
    )]
    EmptyPassword,

    // I/O
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
