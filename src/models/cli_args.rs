use clap::Parser;

/// Program to crawl j-archive.com and parse jeopardy question data into json
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct CliArgs {
    /// The episode number to parse (note: if iteratons are applied, this will be the starting episode)
    #[arg(short = 'e', long = "episode", default_value_t = 7515)]
    // pick a more reasonable default, 7515 is 12/01/22 episode
    pub episode_no: u32,

    /// The number of iterations
    #[arg(short = 'i', long = "iterations", default_value_t = 1)]
    pub iterations: u16,

    /// Where to write the results to
    #[arg(short = 'o', long = "outfile")]
    pub outfile: Option<String>,
}
