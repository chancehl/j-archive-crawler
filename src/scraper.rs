use crate::errors::ScrapingError;
use crate::models::JeopardyQuestion;
use crate::parser::JArchiveDocumentParser;
use std::error::Error;

#[derive(Default)]
pub struct JArchiveScraper;

impl JArchiveScraper {
    /// Creates a new instance of the scraper
    pub fn new() -> Self {
        Default::default()
    }

    /// Scrapes j-archive for jeopardy questions
    pub async fn scrape(
        self,
        episode_no: u32,
        iterations: u32,
    ) -> Result<Vec<JeopardyQuestion>, ScrapingError> {
        let mut results: Vec<JeopardyQuestion> = Vec::new();

        for i in episode_no..(episode_no + iterations) {
            println!("> Scraping jeopardy questions for episode {0}", i);

            let raw_html = JArchiveScraper::get_html(i).await.map_err(|_| {
                ScrapingError::new(format!("Failed to get HTML for episode {0}", i))
            })?;

            if raw_html.contains(&format!("ERROR: No game {0} in database.", episode_no)) {
                return Err(ScrapingError::new(format!(
                    "Missing episode {0} in JArchive database",
                    i
                )));
            }

            let document = scraper::Html::parse_document(&raw_html);

            let questions = JArchiveDocumentParser::new(document).parse_all_rounds();

            println!(">> Successfully scraped questions episode {0}", i);

            results.extend(questions);
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
