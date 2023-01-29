mod crawler;
mod models;
mod parser;
mod reporter;
mod sanitizer;
mod serializer;

use clap::Parser;
use crawler::{CrawlerError, JArchiveCrawler};
use models::cli_args::CliArgs;
use reporter::ReporterBuilder;

#[tokio::main]
async fn main() -> Result<(), CrawlerError> {
    let args = CliArgs::parse();

    let results = JArchiveCrawler::new()
        .crawl(args.episode_no, args.iterations.into())
        .await;

    match results {
        Ok(episodes) => {
            let reporter = ReporterBuilder::new()
                .set_outfile(args.outfile)
                .build()
                .expect("Could not build reporter");

            reporter
                .write(&episodes)
                .await
                .expect("Unable to write results to outfile");
        }
        Err(err) => panic!("Encountered the following error: {0}", err),
    };

    Ok(())
}
