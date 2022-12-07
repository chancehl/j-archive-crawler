mod errors;
mod models;
mod parser;
mod reporter;
mod scraper;

use crate::scraper::JArchiveScraper;
use clap::Parser;
use errors::ScrapingError;
use models::CliArgs;
use reporter::Reporter;

#[tokio::main]
async fn main() -> Result<(), ScrapingError> {
    let args = CliArgs::parse();

    let results = JArchiveScraper::new()
        .scrape(args.episode_no, args.iterations.into())
        .await;

    match results {
        Ok(questions) => {
            let reporter = Reporter::new(&questions);

            if let Some(out) = args.outfile {
                reporter
                    .write(out)
                    .await
                    .expect("Unable to write results to outfile");
            } else {
                // if we don't provide an outfile, assume the user wants results logged to the console
                reporter.echo();
            }

            println!("Run success!");
        }
        Err(err) => panic!("Encountered the following error: {0}", err),
    };

    Ok(())
}
