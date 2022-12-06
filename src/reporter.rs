use std::{error::Error, fs};

use serde::{Deserialize, Serialize};

use crate::models::JeopardyQuestion;

#[derive(Serialize, Deserialize)]
pub struct Reporter {
    json: String,
}

impl Reporter {
    pub fn new(questions: &Vec<JeopardyQuestion>) -> Self {
        Self {
            json: serde_json::to_string_pretty(&questions).expect("Could not serialize questions"),
        }
    }

    /// Writes json report to disk
    pub async fn write(self, loc: String) -> Result<(), Box<dyn Error>> {
        fs::write(loc, self.json).expect("Could not write results to disk");

        Ok(())
    }

    /// Writes json output to console
    pub fn echo(self) {
        println!("{}", self.json);
    }
}
