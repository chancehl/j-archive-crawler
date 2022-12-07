mod models;
mod parser;
mod reporter;
mod scraper;

use crate::scraper::{JArchiveScraper, ScrapingError};
use clap::Parser;
use models::CliArgs;
use reporter::Reporter;

#[tokio::main]
async fn main() -> Result<(), ScrapingError> {
    let args = CliArgs::parse();

    let results = JArchiveScraper::new()
        .scrape(args.episode_no, args.iterations.into())
        .await
        .unwrap();

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
