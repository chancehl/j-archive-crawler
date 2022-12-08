use serde::{Deserialize, Serialize};

use super::{
    error::Error,
    question::{JeopardyQuestion, Round},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JeopardyRound {
    pub questions: Vec<JeopardyQuestion>,
    pub round: Round,
}

#[derive(Default)]
pub struct JeopardyRoundBuilder {
    questions: Option<Vec<JeopardyQuestion>>,
    round: Option<Round>,
}

impl JeopardyRoundBuilder {
    /// Creates a new instance of the round builder
    pub fn new() -> Self {
        JeopardyRoundBuilder::default()
    }

    /// Sets the questions
    pub fn set_questions(&mut self, questions: Vec<JeopardyQuestion>) -> &mut Self {
        self.questions = Some(questions);

        self
    }

    /// Sets the round
    pub fn set_round(&mut self, round: Round) -> &mut Self {
        self.round = Some(round);

        self
    }

    /// Builds the round
    pub fn build(&mut self) -> Result<JeopardyRound, Error> {
        let Some(questions) = &self.questions else {
            return Err(Error::Static("Missing questions"));
        };

        let Some(round) = &self.round else {
            return Err(Error::Static("Missing round"));
        };

        Ok(JeopardyRound {
            questions: questions.to_owned(),
            round: round.to_owned(),
        })
    }
}
