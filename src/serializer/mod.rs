use serde_json::to_string_pretty;

use crate::models::{episode::JeopardyEpisode, error::Error};

pub struct Serializer {
    episodes: Vec<JeopardyEpisode>,
}

impl Serializer {
    pub fn stringify_all(&self) -> String {
        to_string_pretty(&self.episodes).expect("Could not serialize episode data")
    }
}

#[derive(Default)]
pub struct SerializerBuilder {
    episodes: Option<Vec<JeopardyEpisode>>,
}

impl SerializerBuilder {
    pub fn new() -> Self {
        SerializerBuilder::default()
    }

    pub fn set_episodes(&mut self, episodes: Vec<JeopardyEpisode>) -> &mut Self {
        self.episodes = Some(episodes);

        self
    }

    pub fn build(&mut self) -> Result<Serializer, Error> {
        let Some(episodes) = &self.episodes else {
            return Err(Error::Static("Missing episodes"));
        };

        Ok(Serializer {
            episodes: episodes.to_vec(),
        })
    }
}
