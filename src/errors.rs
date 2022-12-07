use std::fmt;

#[derive(Debug, Clone)]
pub struct ScrapingError {
    pub episode_no: u32,
}

impl fmt::Display for ScrapingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            format!("Could not scape data for episode {0}", self.episode_no)
        )
    }
}

impl ScrapingError {
    pub fn new(episode_no: u32) -> ScrapingError {
        ScrapingError { episode_no }
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
