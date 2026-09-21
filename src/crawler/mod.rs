use crate::models::delay::CrawlDelay;
use crate::models::episode::JeopardyEpisode;
use crate::parser::JArchiveDocumentParser;
use crate::reporter::ReporterBuilder;
use std::error::Error;
use std::fmt;
use std::time::Duration;

/// Sent with every request; j-archive rejects requests with no User-Agent.
const USER_AGENT: &str = concat!("j-archive-crawler/", env!("CARGO_PKG_VERSION"));

/// Ceiling on a single request. reqwest applies no timeout of its own, so
/// without this a connection that opens and then goes quiet hangs forever.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Ceiling on establishing the connection alone
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Attempts per episode before it is recorded as failed
const MAX_ATTEMPTS: u32 = 3;

#[derive(Default)]
pub struct JArchiveCrawler;

impl JArchiveCrawler {
    /// Creates a new instance of the scraper
    pub fn new() -> Self {
        Default::default()
    }

    /// Crawls j-archive for jeopardy questions
    pub async fn crawl(
        self,
        episode_no: u32,
        iterations: u32,
        delay: CrawlDelay,
    ) -> Result<Vec<JeopardyEpisode>, CrawlerError> {
        let mut results: Vec<JeopardyEpisode> = Vec::new();
        let mut failures: Vec<(u32, String)> = Vec::new();

        let episode_range = episode_no..(episode_no + iterations);
        let total = episode_range.len();

        let reporter = ReporterBuilder::new()
            .build()
            .expect("Could not build reporter with given data");

        // One client for the whole crawl so connections are pooled and reused
        let client = JArchiveCrawler::build_client()
            .map_err(|err| CrawlerError::new(format!("could not build HTTP client: {0}", err)))?;

        for (index, episode) in episode_range.enumerate() {
            // Wait between requests so a long crawl does not hammer j-archive.
            // Skipped before the first episode and after the last.
            if index > 0 && !delay.is_zero() {
                tokio::time::sleep(delay.sample()).await;
            }

            // Write proress to stdout
            reporter.report_progress(episode, index, total).unwrap();

            // A single bad episode must not abandon the rest of the crawl, so
            // every failure below is recorded and skipped rather than returned.

            // Parse raw html
            let raw_html = match JArchiveCrawler::get_html(&client, episode).await {
                Ok(raw_html) => raw_html,
                Err(err) => {
                    failures.push((episode, format!("request failed: {0}", err)));
                    continue;
                }
            };

            // See if the episode exists in the archive at all
            if raw_html.contains(&format!("ERROR: No game {0} in database.", episode)) {
                failures.push((episode, "not in the j-archive database".to_string()));
                continue;
            }

            let document = scraper::Html::parse_document(&raw_html);

            match JArchiveDocumentParser::new(document, episode).parse() {
                Ok(episode_data) => results.push(episode_data),
                Err(err) => failures.push((episode, format!("parse failed: {0}", err))),
            };
        }

        // Report skips on stderr so they do not corrupt JSON written to stdout
        if !failures.is_empty() {
            eprintln!();
            eprintln!("Skipped {0} of {1} episodes:", failures.len(), total);

            for (episode, reason) in &failures {
                eprintln!("  {0}: {1}", episode, reason);
            }
        }

        // Nothing at all came back: treat that as a hard failure so a bulk run
        // exits non-zero instead of quietly writing an empty array
        if results.is_empty() && !failures.is_empty() {
            return Err(CrawlerError::new(format!(
                "All {0} episode(s) failed; first error: {1}",
                failures.len(),
                failures[0].1
            )));
        }

        Ok(results)
    }

    /// Builds the shared HTTP client
    ///
    /// j-archive returns 403 for requests that send no User-Agent header,
    /// which reqwest omits by default.
    fn build_client() -> Result<reqwest::Client, reqwest::Error> {
        reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(REQUEST_TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .build()
    }

    /// Gets the raw html for a page, retrying transient failures
    pub async fn get_html(
        client: &reqwest::Client,
        episode_no: u32,
    ) -> Result<String, Box<dyn Error>> {
        let url = format!("https://j-archive.com/showgame.php?game_id={0}", episode_no);

        let mut last_error: Option<Box<dyn Error>> = None;

        for attempt in 1..=MAX_ATTEMPTS {
            let result = match client.get(url.as_str()).send().await {
                Ok(response) => match response.error_for_status() {
                    Ok(response) => response.text().await.map_err(Box::<dyn Error>::from),
                    Err(err) => Err(Box::<dyn Error>::from(err)),
                },
                Err(err) => Err(Box::<dyn Error>::from(err)),
            };

            match result {
                Ok(raw_html) => return Ok(raw_html),
                Err(err) => last_error = Some(err),
            }

            // Back off before another attempt: 2s, then 4s
            if attempt < MAX_ATTEMPTS {
                tokio::time::sleep(Duration::from_secs(1 << attempt)).await;
            }
        }

        Err(last_error
            .unwrap_or_else(|| Box::<dyn Error>::from("request failed for an unknown reason")))
    }
}

#[derive(Debug, Clone)]
pub struct CrawlerError {
    msg: String,
}

impl fmt::Display for CrawlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl CrawlerError {
    pub fn new(msg: impl Into<String>) -> CrawlerError {
        CrawlerError { msg: msg.into() }
    }
}
