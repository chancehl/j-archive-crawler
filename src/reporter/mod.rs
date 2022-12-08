use std::{fs, io};

use serde::{Deserialize, Serialize};

use crate::models::episode::JeopardyEpisode;

#[derive(Serialize, Deserialize)]
pub struct Reporter {
    json: String,
}

impl Reporter {
    pub fn new(episode: &Vec<JeopardyEpisode>) -> Self {
        Self {
            json: serde_json::to_string_pretty(&episode).expect("Could not serialize episode data"),
        }
    }

    /// Writes json report to disk
    pub async fn write(self, loc: String) -> Result<(), io::Error> {
        fs::write(loc, self.json)
    }

    /// Writes json output to console
    pub fn echo(self) {
        println!("{}", self.json);
    }
}
