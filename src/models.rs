use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::errors::JeopardyQuestionBuilderError;

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Round {
    Jeopardy,
    DoubleJeopardy,
    FinalJeopardy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JeopardyQuestion {
    pub prompt: String,
    pub category: String,
    pub round: Round,
    pub value: Option<u32>,
    pub answer: Option<String>,
}

#[derive(Default)]
pub struct JeopardyQuestionBuilder {
    prompt: Option<String>,
    category: Option<String>,
    round: Option<Round>,
    value: Option<u32>,
    answer: Option<String>,
}

/// Builder pattern for Jeopardy question object
impl JeopardyQuestionBuilder {
    /// Creates a new instance of the builder
    pub fn new() -> Self {
        JeopardyQuestionBuilder::default()
    }

    /// Sets the prompt
    pub fn set_prompt(&mut self, prompt: impl Into<String>) -> &mut Self {
        self.prompt = Some(prompt.into());

        self
    }

    /// Sets the category
    pub fn set_category(&mut self, category: impl Into<String>) -> &mut Self {
        self.category = Some(category.into());

        self
    }

    /// Sets the round
    pub fn set_round(&mut self, round: Round) -> &mut Self {
        self.round = Some(round);

        self
    }

    /// Sets the value
    pub fn set_value(&mut self, value: Option<u32>) -> &mut Self {
        self.value = value;

        self
    }

    /// Sets the answer
    pub fn set_answer(&mut self, answer: Option<String>) -> &mut Self {
        self.answer = answer;

        self
    }

    /// Builds the object and returns it
    pub fn build(&self) -> Result<JeopardyQuestion, JeopardyQuestionBuilderError> {
        let Some(prompt) = self.prompt.as_ref() else {
            return Err(JeopardyQuestionBuilderError::new("Missing prompt"));
        };

        let Some(category) = self.category.as_ref() else {
            return Err(JeopardyQuestionBuilderError::new("Missing category"));
        };

        let Some(round) = self.round else {
            return Err(JeopardyQuestionBuilderError::new("Missing round"));
        };

        Ok(JeopardyQuestion {
            answer: self.answer.to_owned(),
            category: category.to_string(),
            prompt: prompt.to_string(),
            round,
            value: self.value,
        })
    }
}

/// Program to scrape jeopardy question data from j-archive.com
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct CliArgs {
    /// The episode number to parse (note: if iteratons are applied, this will be the starting episode)
    #[arg(short = 'e', long = "episode", default_value_t = 7515)]
    // pick a more reasonable default, 7515 is 12/01/22 episode
    pub episode_no: u32,

    /// The number of iterations
    #[arg(short = 'i', long = "iterations", default_value_t = 1)]
    pub iterations: u8,

    /// Where to write the results to
    #[arg(short = 'o', long = "outfile")]
    pub outfile: Option<String>,
}
