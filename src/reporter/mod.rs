use std::{fs, io};

use serde::{Deserialize, Serialize};

use crate::{
    models::{episode::JeopardyEpisode, error::Error},
    serializer::SerializerBuilder,
};

#[derive(Serialize, Deserialize)]
pub struct Reporter {
    outfile: Option<String>,
}

impl Reporter {
    /// Writes json report to disk
    pub async fn write(self, episodes: &Vec<JeopardyEpisode>) -> Result<(), io::Error> {
        let json = SerializerBuilder::new()
            .set_episodes(episodes.to_vec())
            .build()
            .expect("Could not build serializer from data")
            .stringify_all();

        if let Some(outfile) = self.outfile {
            fs::write(outfile, json)
        } else {
            print!("{}", json);

            Ok(())
        }
    }
}

#[derive(Default)]
pub struct ReporterBuilder {
    outfile: Option<String>,
}

impl ReporterBuilder {
    pub fn new() -> Self {
        ReporterBuilder::default()
    }

    pub fn set_outfile(&mut self, outfile: Option<String>) -> &mut Self {
        self.outfile = outfile;

        self
    }

    pub fn build(&mut self) -> Result<Reporter, Error> {
        Ok(Reporter {
            outfile: self.outfile.to_owned(),
        })
    }
}
