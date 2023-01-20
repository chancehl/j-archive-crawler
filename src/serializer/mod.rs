use serde_json::to_string_pretty;

use crate::models::{episode::JeopardyEpisode, error::Error};

pub struct Serializer {
    episodes: Vec<JeopardyEpisode>,
}

impl Serializer {
    /// Converts episodes to jsons
    pub fn to_json(&self) -> String {
        to_string_pretty(&self.episodes).expect("Could not serialize episode data")
    }
}

#[derive(Default)]
pub struct SerializerBuilder {
    episodes: Option<Vec<JeopardyEpisode>>,
}

impl SerializerBuilder {
    /// Creates a new instance of the serializer struct
    pub fn new() -> Self {
        SerializerBuilder::default()
    }

    /// Sets the episodes
    pub fn set_episodes(&mut self, episodes: Vec<JeopardyEpisode>) -> &mut Self {
        self.episodes = Some(episodes);

        self
    }

    /// Builds the serializer object
    pub fn build(&mut self) -> Result<Serializer, Error> {
        let Some(episodes) = &self.episodes else {
            return Err(Error::Static("Missing episodes"));
        };

        Ok(Serializer {
            episodes: episodes.to_vec(),
        })
    }
}
