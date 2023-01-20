use std::{
    fs,
    io::{self, stdout},
};

use crossterm::{
    cursor::{RestorePosition, SavePosition},
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use serde::{Deserialize, Serialize};

use crate::{models::episode::JeopardyEpisode, serializer::SerializerBuilder};

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
            .to_json();

        if let Some(outfile) = self.outfile {
            fs::write(outfile, json)
        } else {
            print!("{}", json);

            Ok(())
        }
    }

    /// Reports the progress of the current iteration
    pub fn report_progress(
        &self,
        episode_no: u32,
        curr: usize,
        total: usize,
    ) -> Result<(), io::Error> {
        let symbols = vec!["\\", "|", "/", "―"];
        let modulo = curr.rem_euclid(symbols.len());

        let output = format!(" {} ", symbols[modulo]);
        let formatted_episode_no = format!(" #{} ({} / {})", episode_no, (curr + 1), total);

        stdout()
            .execute(SavePosition)?
            .execute(Clear(ClearType::CurrentLine))?
            .execute(SetForegroundColor(Color::Green))?
            .execute(Print(output))?
            .execute(ResetColor)?
            .execute(Print("Crawling episode"))?
            .execute(SetForegroundColor(Color::Green))?
            .execute(Print(formatted_episode_no))?
            .execute(ResetColor)?
            .execute(RestorePosition)?;

        Ok(())
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

    pub fn build(&mut self) -> Result<Reporter, crate::models::error::Error> {
        Ok(Reporter {
            outfile: self.outfile.to_owned(),
        })
    }
}
