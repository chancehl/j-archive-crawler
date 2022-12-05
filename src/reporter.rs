use std::{error::Error, fs};

use serde::{Deserialize, Serialize};

use crate::models::JeopardyQuestion;

#[derive(Serialize, Deserialize)]
pub struct Reporter {
    questions: Vec<JeopardyQuestion>,
}

impl Reporter {
    pub fn new(questions: Vec<JeopardyQuestion>) -> Self {
        Self { questions }
    }

    pub async fn write(self, loc: String) -> Result<(), Box<dyn Error>> {
        let json =
            serde_json::to_string_pretty(&self.questions).expect("Could not serialize questions");

        fs::write(loc, json).expect("Could not write results to disk");

        Ok(())
    }
}
