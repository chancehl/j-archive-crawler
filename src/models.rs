use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Round {
    Jeopardy,
    DoubleJeopardy,
    FinalJeopardy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JeopardyQuestion {
    pub question: String,
    pub category: String,
    pub round: Round,
    pub value: u32,
    pub answer: Option<String>,
}

/// Program to scrape jeopardy question data from j-archive.com
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct CliArgs {
    /// The episode number to parse
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
