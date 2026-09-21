mod crawler;
mod models;
mod parser;
mod reporter;
mod resume;
mod serializer;
mod utils;

use clap::Parser;
use crawler::{CrawlerError, JArchiveCrawler};
use models::cli_args::CliArgs;
use models::delay::CrawlDelay;
use reporter::ReporterBuilder;
use resume::ResumeLog;

#[tokio::main]
async fn main() -> Result<(), CrawlerError> {
    let args = CliArgs::parse();

    let iterations: u32 = args.iterations.into();
    let range = args.episode_no..(args.episode_no + iterations);

    // Only a crawl writing to a file can be resumed; there is no picking up
    // halfway through a stream to stdout
    let mut resume = match &args.outfile {
        Some(outfile) => ResumeLog::open(outfile, range)
            .map_err(|err| CrawlerError::new(format!("could not open resume log: {0}", err)))?,
        None => ResumeLog::disabled(),
    };

    let results = JArchiveCrawler::new()
        .crawl(
            args.episode_no,
            iterations,
            CrawlDelay::new(args.delay_ms, args.jitter_ms),
            &mut resume,
        )
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

            // Only safe once the real output is on disk
            resume
                .finish()
                .expect("Unable to remove resume log");
        }
        Err(err) => panic!("Encountered the following error: {0}", err),
    };

    Ok(())
}
