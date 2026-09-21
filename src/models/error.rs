#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Static error (remove this in favor of a more specific struct once your class matures)
    #[error("Static error: {0}")]
    Static(&'static str),

    /// Error carrying context about the page that failed to parse
    #[error("{0}")]
    Message(String),
}

impl Error {
    /// Builds a message error from anything string-like
    pub fn message(msg: impl Into<String>) -> Self {
        Error::Message(msg.into())
    }
}
