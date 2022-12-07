use std::fmt;

#[derive(Debug, Clone)]
pub struct ScrapingError {
    msg: String,
}

impl fmt::Display for ScrapingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl ScrapingError {
    pub fn new(msg: impl Into<String>) -> ScrapingError {
        ScrapingError { msg: msg.into() }
    }
}

#[derive(Debug, Clone)]
pub struct JeopardyQuestionBuilderError {
    pub msg: String,
}

impl fmt::Display for JeopardyQuestionBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            format!(
                "Could not build jeopardy question from data: err={0}",
                self.msg
            )
        )
    }
}

impl JeopardyQuestionBuilderError {
    pub fn new(msg: impl Into<String>) -> JeopardyQuestionBuilderError {
        JeopardyQuestionBuilderError { msg: msg.into() }
    }
}
