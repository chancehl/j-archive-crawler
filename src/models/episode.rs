use serde::{Deserialize, Serialize};

use super::{error::Error, round::JeopardyRound};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JeopardyEpisode {
    pub air_date: Option<String>,
    pub rounds: (JeopardyRound, JeopardyRound, JeopardyRound),
    pub id: u32,
}

#[derive(Default)]
pub struct JeopardyEpisodeBuilder {
    air_date: Option<String>,
    rounds: Option<(JeopardyRound, JeopardyRound, JeopardyRound)>,
    id: Option<u32>,
}

impl JeopardyEpisodeBuilder {
    // Creates a new instance of the episode builder
    pub fn new() -> Self {
        JeopardyEpisodeBuilder::default()
    }

    // Sets the air date
    pub fn set_air_date(&mut self, air_date: Option<String>) -> &mut Self {
        self.air_date = air_date;

        self
    }

    // Sets the rounds
    pub fn set_rounds(
        &mut self,
        rounds: (JeopardyRound, JeopardyRound, JeopardyRound),
    ) -> &mut Self {
        self.rounds = Some(rounds);

        self
    }

    // Sets the id
    pub fn set_id(&mut self, id: u32) -> &mut Self {
        self.id = Some(id);

        self
    }

    // Builds the episode
    pub fn build(&self) -> Result<JeopardyEpisode, Error> {
        let Some(id) = &self.id else {
            return Err(Error::Static("Missing id"));
        };

        let Some(rounds) = &self.rounds else {
            return Err(Error::Static("Missing rounds"));
        };

        Ok(JeopardyEpisode {
            id: id.to_owned(),
            air_date: self.air_date.to_owned(),
            rounds: rounds.to_owned(),
        })
    }
}
