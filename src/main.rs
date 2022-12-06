mod models;
mod parser;
mod reporter;
mod scraper;

use crate::scraper::scrape;
use clap::Parser;
use models::{CliArgs, JeopardyQuestion};
use reporter::Reporter;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = CliArgs::parse();

    let mut results: Vec<JeopardyQuestion> = Vec::new();

    for i in args.episode_no..(args.episode_no + u32::from(args.iterations)) {
        println!("> Scraping jeopardy questions for episode {0}", i);

        match scrape(i).await? {
            Some(questions) => {
                println!(">> Successfully scraped questions episode {0}", i);

                results.extend(questions.iter().cloned());
            }
            None => {
                println!(
                    ">> Failed to scrape questions for episode {0}. Skipping.",
                    i
                );

                continue;
            }
        };
    }

    if let Some(out) = args.outfile {
        Reporter::new(&results)
            .write(out)
            .await
            .expect("Unable to write results to outfile");
    } else {
        // if we don't provide an outfile, assume the user wants results logged to the console
        Reporter::new(&results).echo();
    }

    println!("Run complete!");

    Ok(())
}
