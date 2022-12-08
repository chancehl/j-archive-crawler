#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Static error (remove this in favor of a more specific struct once your class matures)
    #[error("Static error: {0}")]
    Static(&'static str),
}
