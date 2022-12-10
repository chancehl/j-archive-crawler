use crate::models::episode::JeopardyEpisode;
use crate::parser::JArchiveDocumentParser;
use crate::reporter::ReporterBuilder;
use std::error::Error;
use std::fmt;

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
    ) -> Result<Vec<JeopardyEpisode>, CrawlerError> {
        let mut results: Vec<JeopardyEpisode> = Vec::new();

        let episode_range = episode_no..(episode_no + iterations);
        let total = episode_range.len();
        let mut index = 0;

        let reporter = ReporterBuilder::new().build().unwrap();

        for episode in episode_range {
            // Write proress to stdout
            reporter.report_progress(episode, index, total).unwrap();

            // Parse raw html
            let raw_html = JArchiveCrawler::get_html(episode).await.map_err(|_| {
                CrawlerError::new(format!("Failed to get HTML for episode {0}", episode))
            })?;

            // See if the
            if raw_html.contains(&format!("ERROR: No game {0} in database.", episode_no)) {
                return Err(CrawlerError::new(format!(
                    "Missing episode {0} in JArchive database",
                    episode
                )));
            }

            let document = scraper::Html::parse_document(&raw_html);

            if let Ok(episode_data) = JArchiveDocumentParser::new(document, episode).parse() {
                results.push(episode_data);
            } else {
                println!(
                    "Failed to scrape j-archive.com for jeopardy episode {0}",
                    episode
                )
            };

            index = index + 1;
        }

        Ok(results)
    }

    /// Gets the raw html for a page
    pub async fn get_html(episode_no: u32) -> Result<String, Box<dyn Error>> {
        let url = format!("https://j-archive.com/showgame.php?game_id={0}", episode_no);

        let raw_html = reqwest::get(url).await?.text().await?;

        Ok(raw_html)
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
